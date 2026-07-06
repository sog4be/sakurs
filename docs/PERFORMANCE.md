# Performance Tuning Guide

## Table of Contents

- [Known Issues (v0.1.1)](#known-issues-v011)
- [Automatic Performance Tuning](#automatic-performance-tuning)
- [Manual Thread Control](#manual-thread-control)
- [Chunk Size Tuning](#chunk-size-tuning)
  - [Chunk Size Guidelines](#chunk-size-guidelines)
  - [Combining Thread Count and Chunk Size](#combining-thread-count-and-chunk-size)
- [Performance Profiles](#performance-profiles)
  - [Small Files (< 256KB)](#small-files--256kb)
  - [Medium Files (256KB - 10MB)](#medium-files-256kb---10mb)
  - [Large Files (> 10MB)](#large-files--10mb)
- [Understanding Thread Count Selection](#understanding-thread-count-selection)
- [Benchmarking](#benchmarking)
- [When to Override Automatic Selection](#when-to-override-automatic-selection)
- [Memory Considerations](#memory-considerations)
- [Streaming Mode](#streaming-mode)
- [Performance Tips](#performance-tips)
- [Profiling](#profiling)
- [Common Performance Patterns](#common-performance-patterns)
  - [Fast Processing (News Articles, Logs)](#fast-processing-news-articles-logs)
  - [Balanced Processing (Books, Documents)](#balanced-processing-books-documents)
  - [Heavy Processing (Large Corpora)](#heavy-processing-large-corpora)

## Known Issues (v0.1.1)

> **This section documents measured v0.1.1 behavior.** It contradicts some of
> the tuning advice below; the advice is kept as-is because it describes the
> intended design, which the v0.2.0 rework restores. Numbers were measured on
> Apple Silicon (release build, 1MB synthetic English text) and are
> reproducible with `cargo bench --bench throughput_baseline` plus the tests
> referenced below.
>
> **Status update (v0.1.2 development):** the correctness issues below are
> fixed except for one narrow class (decisions whose lookahead is cut
> exactly at a chunk edge, e.g. an abbreviation split as `Dr.`|`Smith` —
> pinned by the ignored `abbreviation_decision_at_exact_chunk_edge` test and
> planned for the v0.2.0 scanner redesign). Throughput improved 30–110×
> (plain 0.18 → 12.4 MB/s, quote-heavy 0.07 → 7.8 MB/s, abbreviation-heavy
> 0.09 → 3.3 MB/s at default settings, threads=1); the remaining
> chunk-size-proportional cost is the full-chunk copy into
> `BoundaryContext::text` per terminator, also scheduled for v0.2.0.

### Throughput is far below design targets, and larger chunks are *slower*

Several per-terminator and per-enclosure code paths currently perform
O(chunk_size) work (full-chunk copies and UTF-8 re-decodes), so total cost
grows with chunk size instead of shrinking:

| Configuration (threads=1) | plain prose | quote-heavy | abbreviation-heavy |
|---|---|---|---|
| naive 1-pass scan (reference) | 400 MB/s | 894 MB/s | 875 MB/s |
| chunk=16KB | 0.57 MB/s | 0.83 MB/s | 0.44 MB/s |
| chunk=64KB | 0.42 MB/s | 0.26 MB/s | 0.25 MB/s |
| chunk=256KB (default) | 0.18 MB/s | 0.07 MB/s | 0.09 MB/s |
| chunk=1MB (single chunk) | 0.05 MB/s | 0.02 MB/s | 0.02 MB/s |

Until this is fixed, prefer *smaller* `--chunk-kb` values for throughput —
but see the correctness caveat below before doing so.

### Chunked results can diverge from sequential results

Boundary decisions are finalized during the scan phase using context that is
truncated at chunk edges, and overlapping chunk regions double-count
enclosure state. Measured consequences (512KB synthetic corpora, compared
against a single-chunk reference):

- Japanese text with 「」/『』: with default 256KB chunks, ~50% of boundaries
  are lost (everything after the first chunk boundary).
- Quote-heavy English: 87% of boundaries lost at 64KB chunks; a handful of
  spurious boundaries at 256KB chunks.
- Text ending exactly at a terminator loses its final boundary in
  multi-chunk mode.

These failures are pinned by `sakurs-core/tests/chunk_invariance.rs` and
`sakurs-core/tests/chunking_regressions.rs` (marked `#[ignore]` until fixed;
run with `cargo test -- --ignored`). Fixes are planned for v0.1.2
(contiguity, final boundary, abbreviation index bugs) and v0.2.0 (full
chunk-invariance via the scanner redesign).

## Automatic Performance Tuning

Sakurs automatically optimizes performance based on:
- Input text size
- Available CPU cores
- System resources

For most use cases, the default automatic mode provides optimal performance.

## Manual Thread Control

Use the `--threads` option when you need explicit control:

```bash
# Force sequential processing
sakurs process -i input.txt --threads 1

# Use 4 threads regardless of file size
sakurs process -i input.txt --threads 4

# Use all available cores
sakurs process -i input.txt --threads $(nproc)
```

## Chunk Size Tuning

The chunk size determines how text is split for parallel processing. The default is 256KB, which works well for most cases.

Use the `--chunk-kb` option to customize:

```bash
# Small chunks for files with many short sentences
sakurs process -i short_sentences.txt --chunk-kb 64

# Large chunks for files with long sentences or few boundaries
sakurs process -i long_text.txt --chunk-kb 512

# Very large chunks for maximum throughput on large files
sakurs process -i huge_file.txt --chunk-kb 1024
```

### Chunk Size Guidelines

- **64-128KB**: Good for texts with many short sentences
- **256KB** (default): Balanced for most use cases
- **512KB-1MB**: Better for large files with fewer sentence boundaries
- **>1MB**: Maximum throughput but may reduce parallelism benefits

### Combining Thread Count and Chunk Size

For optimal performance on large files, tune both parameters:

```bash
# Maximum performance for very large files
sakurs process -i huge.txt --threads 8 --chunk-kb 1024

# Balanced approach for medium files
sakurs process -i medium.txt --threads 4 --chunk-kb 256
```

## Performance Profiles

### Small Files (< 256KB)
- Automatically uses sequential processing
- Overhead of parallelization exceeds benefits
- No action needed

### Medium Files (256KB - 10MB)
- Automatically uses 2-4 threads
- Balanced performance and resource usage
- Consider `--threads 2` for consistent behavior

### Large Files (> 10MB)
- Automatically scales to available cores
- Maximum parallelization benefit
- Use `--parallel` flag to force parallel mode

## Understanding Thread Count Selection

The automatic thread calculation follows this formula:
```
threads = min(text_size / 256KB, available_CPU_cores)
```

Examples:
- 100KB file on 8-core machine → 1 thread (sequential)
- 1MB file on 8-core machine → 4 threads
- 10MB file on 4-core machine → 4 threads (CPU limited)
- 10MB file on 16-core machine → 40 threads → capped at 16

## Benchmarking

Compare different configurations:

```bash
# Benchmark sequential vs parallel
time sakurs process -i large.txt --threads 1 -o /dev/null
time sakurs process -i large.txt --parallel -o /dev/null

# Find optimal thread count
for t in 1 2 4 8; do
    echo "Threads: $t"
    time sakurs process -i large.txt --threads $t -o /dev/null
done
```

## When to Override Automatic Selection

1. **Batch Processing**: Use consistent thread count across files
   ```bash
   find . -name "*.txt" -exec sakurs process -i {} --threads 4 \;
   ```

2. **Resource-Constrained Systems**: Limit threads on shared servers
   ```bash
   sakurs process -i large.txt --threads 2  # Leave cores for other processes
   ```

3. **Debugging**: Force sequential for easier troubleshooting
   ```bash
   sakurs process -i problematic.txt --threads 1 -vv
   ```

4. **CI/CD Pipelines**: Predictable resource usage
   ```bash
   sakurs process -i docs.txt --threads "${CI_MAX_THREADS:-2}"
   ```

## Memory Considerations

Each thread requires memory for:
- Text chunk processing (~256KB per thread)
- State tracking overhead
- Output buffer

Approximate memory usage:
```
memory = base_memory + (threads * chunk_overhead)
```

For memory-constrained systems, reduce thread count:
```bash
# Limit to 2 threads on low-memory systems
sakurs process -i large.txt --threads 2
```

## Streaming Mode

For very large files or continuous streams, use streaming mode:
```bash
# Process in chunks with limited memory usage
sakurs process -i huge.txt --stream

# Custom chunk size for streaming
sakurs process -i huge.txt --stream --stream-chunk-mb 50
```

## Performance Tips

1. **SSD vs HDD**: Parallel processing benefits more from SSDs
2. **File Format**: Plain text processes faster than complex encodings
3. **Output Format**: Text output is fastest, JSON adds ~10% overhead
4. **Language**: English processing is slightly faster than Japanese

## Profiling

To understand performance bottlenecks:

```bash
# Use verbose mode to see timing information
sakurs process -i large.txt -vv

# Profile with system tools
time -v sakurs process -i large.txt --threads 4
```

## Common Performance Patterns

### Fast Processing (News Articles, Logs)
```bash
# Many small files - limit parallelism overhead
find . -name "*.log" | xargs -P 4 -I {} sakurs process -i {} --threads 1
```

### Balanced Processing (Books, Documents)
```bash
# Let auto-detection handle it
sakurs process -i book.txt
```

### Heavy Processing (Large Corpora)
```bash
# Maximum parallelism with progress tracking
sakurs process -i corpus.txt --parallel -v
```