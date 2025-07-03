# Sakurs Benchmarks

This directory contains comprehensive benchmarks for the sakurs sentence boundary detection system.

## Structure

```
benchmarks/
├── core/          # Core library benchmarks (Rust/Criterion)
├── cli/           # CLI benchmarks (Hyperfine)
├── python/        # Python binding benchmarks (pytest-benchmark)
├── baselines/     # Comparison systems (NLTK Punkt, etc.)
├── data/          # Shared test data and Brown Corpus
└── results/       # Benchmark results and reports
```

## Quick Start

### Running Core Benchmarks

```bash
cd benchmarks/core

# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench accuracy
cargo bench performance
cargo bench scalability

# Run with specific features
cargo bench --features parallel

# Quick benchmarks for development
CRITERION_FAST=1 cargo bench
```

### Viewing Results

Benchmark results are saved in:
- HTML reports: `benchmarks/core/target/criterion/`
- Raw data: `benchmarks/core/target/criterion/*/base/`

Open the HTML reports in your browser:
```bash
open target/criterion/report/index.html
```

## Benchmark Suites

### 1. Accuracy Benchmarks (`accuracy.rs`)

Measures the precision, recall, and F1 score of sentence boundary detection.

- **Metrics**: F1, Precision, Recall, Pk score, WindowDiff
- **Test cases**: Simple sentences, abbreviations, quotations, numbers, complex mixed text
- **Brown Corpus**: Evaluation on real-world text samples

Run accuracy report:
```bash
cargo test --benches test_accuracy_report -- --nocapture
```

### 2. Performance Benchmarks (`performance.rs`)

Measures throughput and latency characteristics.

- **Throughput**: Characters/second for various text sizes
- **Latency**: Response time for small inputs
- **Complexity**: Performance vs text complexity
- **Memory**: Allocation patterns

Run performance metrics:
```bash
cargo test --benches test_performance_metrics -- --nocapture
```

### 3. Scalability Benchmarks (`scalability.rs`)

Evaluates parallel processing efficiency.

- **Thread scaling**: Performance with 1, 2, 4, 8 threads
- **Chunk size impact**: Optimal chunk sizes for parallelization
- **Efficiency**: Speedup and parallel efficiency metrics

Run scalability report:
```bash
cargo test --benches test_scalability_report --features parallel -- --nocapture
```

## Benchmark Configuration

### Criterion Configuration

The benchmarks use custom Criterion configurations optimized for different scenarios:

- **Accuracy**: Higher sample size (50) for statistical significance
- **Performance**: Longer measurement time (10s) for stable results
- **Scalability**: Fewer samples (10) but longer runtime (15s)

### Environment Variables

- `CRITERION_FAST=1`: Run quick benchmarks (reduced samples/time)
- `RAYON_NUM_THREADS`: Control maximum parallel threads

## Adding New Benchmarks

1. Add test data to `core/src/data.rs`
2. Create new benchmark file in `core/benches/`
3. Add benchmark entry to `core/Cargo.toml`
4. Update this README

## Brown Corpus Integration

The benchmarks can use the real Brown Corpus dataset for evaluation:

### Setup
```bash
# Download and process Brown Corpus
cd benchmarks/data/brown_corpus
make download
```

### Running Brown Corpus Benchmarks
```bash
# Run detailed Brown Corpus accuracy report
cargo run --example brown_corpus_report

# Run benchmarks with Brown Corpus data
cargo bench brown_corpus
```

## Comparing with Baselines

NLTK Punkt comparison will be added in a future PR. The infrastructure is designed to support:

1. Unified interface for different segmenters
2. Fair comparison with same test data
3. Statistical significance testing

## Performance Targets

Based on initial benchmarks:

- **Accuracy**: >99% F1 score on Brown Corpus
- **Throughput**: >100K sentences/second on modern hardware
- **Scalability**: >3x speedup with 4 cores
- **Memory**: <100MB for 1M character texts

## Troubleshooting

### "No such file or directory" errors
Make sure you're in the `benchmarks/core` directory before running cargo commands.

### Parallel benchmarks not running
Ensure the parallel feature is enabled: `cargo bench --features parallel`

### Results vary significantly
- Ensure no other CPU-intensive processes are running
- Use `CRITERION_FAST=1` for development, full runs for final results
- Consider using performance governor on Linux

## Future Work

- [ ] NLTK Punkt integration and comparison
- [ ] Brown Corpus full dataset integration  
- [ ] CI/CD benchmark regression detection
- [ ] Memory profiling with valgrind/heaptrack
- [ ] Flame graphs for performance analysis