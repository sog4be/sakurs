# Performance Tuning Guide

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