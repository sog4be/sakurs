# Performance Guide

## Table of Contents

- [Measured Performance](#measured-performance)
- [Determinism](#determinism)
- [Thread Control](#thread-control)
- [Chunk Size](#chunk-size)
- [Memory](#memory)
- [Benchmarking](#benchmarking)
- [Profiling](#profiling)

## Measured Performance

Numbers below were measured at v0.2.0 on Apple Silicon (M2 Max), release builds. Absolute throughput varies by machine; the shapes (scaling, insensitivity to chunk size) are properties of the algorithm.

Single thread, 1MB synthetic English (criterion, `cargo bench --bench throughput_baseline`):

| Text profile | Throughput |
|---|---|
| plain prose | ~190 MB/s |
| quote-heavy | ~72 MB/s |
| abbreviation-heavy | ~69 MB/s |

Quote- and abbreviation-heavy text costs more because every suppressible enclosure character runs the suppression oracle and every period runs the abbreviation matcher; both are bounded-window, allocation-free operations.

Multithread scaling, 50MB plain English through the core pipeline:

| Threads | Throughput | Efficiency |
|---|---|---|
| 1 | 252 MB/s | 100% |
| 2 | 489 MB/s | 97% |
| 4 | 929 MB/s | 92% |
| 8 | 1.44 GB/s | 71% |

Through the public `SentenceProcessor` API, which additionally computes a character offset for every boundary, the same measurement gives 225 MB/s single-threaded and 1.11 GB/s at 8 threads: the character-offset pass is sequential and becomes visible once the parallel phases are fast.

The scan phase parallelizes over chunks; the prefix fold touches only per-chunk aggregates and pending items (O(chunks)); the reduce phase is embarrassingly parallel. Throughput is flat across chunk sizes because per-character work is constant — no code path does O(chunk) work per terminator.

## Determinism

Output is bit-identical across chunk sizes, thread counts, and runs: boundary decisions are pure functions of a bounded text window, and decisions whose window crosses a chunk edge are deferred and resolved with the neighboring chunk's context. There is no model, no randomness, and no execution-order dependence. This is enforced by chunk-invariance property tests.

## Thread Control

By default (`Adaptive`), the thread count is chosen from the text size (one thread per ~256KB, capped at available cores), so small inputs stay single-threaded and large inputs use the machine.

```bash
# CLI
sakurs process -i large.txt --threads 8
sakurs process -i small.txt --threads 1
```

```rust
// Rust API
let config = Config::builder().language("en")?.threads(Some(8)).build()?;
```

```python
# Python API
sakurs.split(text, execution_mode="parallel", threads=8)
```

Efficiency is highest when the text is large enough to give every thread multiple chunks; below ~1MB the parallel setup cost usually outweighs the gain, which is what the adaptive default encodes.

## Chunk Size

The default chunk size is 256KB and there is rarely a reason to change it. Correctness never depends on it (see [Determinism](#determinism)), and throughput is flat across a wide range; the only effects are second-order: chunks should be small enough that `threads` chunks exist (parallelism) and large enough that per-chunk fixed costs stay negligible (roughly ≥64KB).

```bash
sakurs process -i huge.txt --chunk-kb 512 --threads 16
```

## Memory

Processing holds the input text plus O(threads) scan states and the collected boundaries. Per-chunk state is small (context buffers of ≤256 bytes, per-enclosure-type counters, pending items); candidate storage is proportional to the number of sentences. There is no per-character allocation on the hot path.

## Benchmarking

```bash
# Criterion micro-benchmarks (text profiles × chunk sizes)
cargo bench --bench throughput_baseline

# Save and compare baselines around a change
cargo bench --bench throughput_baseline -- --save-baseline before
cargo bench --bench throughput_baseline -- --baseline before
```

Benchmarks are not CI-gated (machine variance makes hard thresholds flaky); compare saved baselines locally instead.

## Profiling

```bash
# Timing summary from the CLI
sakurs process -i file.txt -v

# Flame graph of the core
cargo install flamegraph
cargo flamegraph --bench throughput_baseline
```
