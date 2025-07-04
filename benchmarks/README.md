# Sakurs Benchmarks

This directory contains comprehensive benchmarks for the sakurs sentence boundary detection system.

## Structure

```
benchmarks/
├── core/          # Core library benchmarks (Rust/Criterion)
├── cli/           # CLI benchmarks (Hyperfine)
├── python/        # Python binding benchmarks (pytest-benchmark)
├── baselines/     # Comparison systems (NLTK Punkt, ja_sentence_segmenter)
├── data/          # Shared test data and corpora
├── results/       # Benchmark results and reports
├── pyproject.toml # Python dependencies (managed with uv)
└── uv.lock        # Locked Python dependencies
```

## Python Environment Setup

All Python benchmarks and tools use Python 3.12 and are managed with `uv`:

```bash
# Install uv if not already installed
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install all dependencies (including optional)
cd benchmarks
uv sync --all-extras

# Run any Python script
uv run python <script.py>
```

## Quick Start

### Running All Experiments

The easiest way to run comprehensive benchmarks is using the integrated experiment script:

```bash
cd benchmarks/cli

# Run all experiments (throughput, memory, accuracy)
./run_experiments.sh

# Prepare data and run experiments
./run_experiments.sh --prepare-data

# Custom thread configurations
./run_experiments.sh --threads 1,4,8 --test-runs 5

# Run specific benchmark types
./run_experiments.sh --skip-memory --skip-accuracy  # Only throughput
./run_experiments.sh --skip-throughput --skip-memory  # Only accuracy
```

Results are saved to timestamped directories with:
- Individual JSON results for each test
- Aggregated results in JSON format
- Formatted markdown tables ready for papers

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
- Integrated experiments: `benchmarks/cli/results/<timestamp>/`
- Core benchmarks: `benchmarks/core/target/criterion/`
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

## Dataset Preparation

The benchmarks use various corpora for evaluation. All datasets are managed through a unified preparation system:

### Quick Setup
```bash
# Prepare all benchmark datasets
cd benchmarks
uv run python cli/scripts/prepare_data.py

# Force re-download (if needed)
uv run python cli/scripts/prepare_data.py --force
```

### Available Datasets

#### 1. Brown Corpus
- **Size**: ~1M words of American English
- **Usage**: Accuracy benchmarks, baseline comparisons

```bash
cd benchmarks/data/brown_corpus
make download
```

#### 2. Wikipedia Datasets
- **Languages**: English and Japanese
- **Size**: 500MB samples per language
- **Version**: June 2024 dumps (20240601)
- **Features**: Version tracking, metadata management

The Wikipedia datasets are automatically downloaded from Hugging Face and include:
- Automatic 500MB sample creation
- Article boundary preservation
- Metadata tracking (download date, article count, etc.)

#### 3. UD Treebanks
- **UD English-EWT**: r2.16 (~25K sentences)
- **UD Japanese-BCCWJ**: r2.16 (~57K sentences)
- **Usage**: Gold standard for accuracy evaluation

Each UD dataset includes:
- Automatic test set size extraction
- Version verification (r2.16)
- Split information (train/dev/test)

### Dataset Statistics

When running data preparation, you'll see statistics like:
```
UD English EWT prepared: /path/to/ewt_plain.txt
  Version: 2.16
  Total sentences: 25,112
  Test set: 2,077 sentences, 25,148 words

UD Japanese-BCCWJ prepared: /path/to/bccwj_plain.txt  
  Version: 2.16
  Total documents: 2,291
  Total sentences: 57,147
  Test set: 4,442 sentences, 105,834 characters

Wikipedia-EN prepared: /path/to/wikipedia_sample_en.txt
  Version: 20240601 dump
  Size: 500.0 MB
  Articles: ~3,000
  
Wikipedia-JA prepared: /path/to/wikipedia_sample_ja.txt
  Version: 20240601 dump  
  Size: 500.0 MB
  Articles: ~8,000
```

### Running Dataset-Specific Benchmarks

#### Individual Benchmark Scripts
```bash
# Brown Corpus accuracy report
cargo run --example brown_corpus_report

# Wikipedia throughput benchmarks
cd benchmarks/cli
uv run python scenarios/performance/wikipedia_throughput.py

# UD Treebank accuracy evaluation
cd benchmarks/cli
uv run python scenarios/accuracy/ud_accuracy.py
```

#### Integrated Experiment Results

The `run_experiments.sh` script automatically generates formatted tables suitable for academic papers:

```bash
# Example output structure
results/20250704_120000/
├── wikipedia_ja_throughput_1t.json
├── wikipedia_ja_throughput_8t.json
├── memory_usage_results.json
├── ud_accuracy_results.json
├── aggregated_results.json
└── experiment_tables.md  # Ready-to-use tables
```

## Comparing with Baselines

The benchmark suite includes comparisons with established baselines:

### English: NLTK Punkt
- Data-driven sentence segmenter
- Single-threaded implementation
- Requires pre-trained models

### Japanese: ja_sentence_segmenter
- Regex-based segmenter
- Single-threaded implementation
- Optimized for Japanese text

### Running Comparisons
```bash
# Run all comparisons (included in run_experiments.sh)
cd benchmarks/cli
./run_experiments.sh

# Individual comparison scripts
uv run python scenarios/comparison/baseline_comparison.py
```

The integrated experiment system automatically:
1. Runs baselines with identical test data
2. Measures throughput, memory, and accuracy
3. Generates comparative tables
4. Calculates statistical significance

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