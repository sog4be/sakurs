# Sakurs CLI Benchmarks

This directory contains Hyperfine-based benchmarks for the Sakurs CLI, measuring both accuracy and performance across multiple languages and datasets.

## Overview

The CLI benchmarks are designed to:
- Measure real-world performance using Hyperfine
- Compare against established baselines (NLTK Punkt, ja_sentence_segmenter)
- Support academic publication requirements
- Ensure reproducibility across different environments

## Structure

```
cli/
├── scenarios/          # Benchmark scenarios
│   ├── accuracy/      # Accuracy measurement scripts
│   ├── performance/   # Performance measurement scripts
│   └── comparison/    # Baseline comparison scripts
├── scripts/           # Helper scripts
│   ├── prepare_data.py       # Data preparation utilities
│   ├── metrics.py            # Unified metrics measurement
│   ├── aggregate_results.py  # Results aggregation
│   ├── evaluate_accuracy.py  # Accuracy evaluation
│   └── format_results.py     # Result formatting
├── run_experiments.sh  # Master experiment script
└── results/           # Benchmark results (git-ignored)
```

## Requirements

- Hyperfine (for benchmarking)
- Python 3.12 (managed with uv)
- sakurs-cli (built and in PATH)
- Baseline tools (NLTK, ja_sentence_segmenter)

## Installation

```bash
# Install Hyperfine
brew install hyperfine  # macOS
# or see https://github.com/sharkdp/hyperfine for other platforms

# Install Python dependencies (from benchmarks/ directory)
cd ..
uv sync --all-extras

# Build sakurs
cd ../../
cargo build --release --bin sakurs
export PATH=$PATH:$(pwd)/target/release
```

## Data Preparation

Before running benchmarks, prepare the required datasets:

```bash
# From benchmarks directory
cd benchmarks
uv run python cli/scripts/prepare_data.py
```

This will:
1. Download and prepare Wikipedia samples (500MB each for EN/JA)
   - Uses June 2024 dumps (20240601) from Hugging Face
   - Tracks version metadata and download timestamps
2. Verify UD Treebanks are available (r2.16)
   - UD English-EWT: ~25K sentences
   - UD Japanese-BCCWJ: ~57K sentences  
3. Check Brown Corpus availability
4. Create CLI-formatted versions for benchmarking

Dataset locations after preparation:
- Wikipedia: `benchmarks/data/wikipedia/cache/`
- UD Treebanks: `benchmarks/data/ud_*/cli_format/`
- Brown Corpus: `benchmarks/data/brown_corpus/`

## Quick Start

### Using the Master Experiment Script (Recommended)

```bash
# From the cli directory
cd cli

# Run all experiments with default settings
./run_experiments.sh

# Prepare data and run experiments
./run_experiments.sh --prepare-data

# Run specific benchmarks
./run_experiments.sh --skip-memory --skip-accuracy  # Only throughput
./run_experiments.sh --threads 1,4,8 --test-runs 5  # Custom settings

# View help
./run_experiments.sh --help
```

The master script will:
- Run throughput benchmarks with multiple thread counts (1, 2, 4, 8)
- Measure memory usage for both single and multi-threaded execution
- Evaluate accuracy on UD treebanks
- Compare against baselines (NLTK, ja_sentence_segmenter)
- Aggregate results into formatted tables

#### Master Script Options

| Option | Description | Default |
|--------|-------------|---------|  
| `-o, --output-dir` | Output directory for results | `results/YYYYMMDD_HHMMSS` |
| `-t, --threads` | Comma-separated list of thread counts | `1,2,4,8` |
| `-w, --warmup-runs` | Number of warmup runs | `1` |
| `-r, --test-runs` | Number of test runs to average | `3` |
| `-p, --prepare-data` | Prepare/download datasets before running | `false` |
| `--skip-throughput` | Skip throughput benchmarks | `false` |
| `--skip-memory` | Skip memory benchmarks | `false` |
| `--skip-accuracy` | Skip accuracy benchmarks | `false` |

### Traditional Scripts

```bash
# From the benchmarks directory, prepare benchmark data
cd ..
uv run python cli/scripts/prepare_data.py

# Return to cli directory
cd cli

# Run all accuracy benchmarks
bash scenarios/accuracy/run_all.sh

# Run all performance benchmarks (Wikipedia-based with Hyperfine)
bash scenarios/performance/run_all_hyperfine.sh

# Or run basic performance test
bash scenarios/performance/run_all.sh

# Run all baseline comparisons (Hyperfine-based)
bash scenarios/comparison/run_all_comparisons.sh

# Or run complete benchmark suite (accuracy + performance + comparison)
bash scenarios/comparison/full_benchmark_suite.sh
```

## Benchmark Types

### 1. Accuracy Benchmarks

Measure segmentation accuracy using annotated corpora:
- **English**: UD English EWT
- **Japanese**: UD Japanese-BCCWJ

Metrics: Precision, Recall, F1, Pk, WindowDiff

### 2. Performance Benchmarks

Measure throughput and latency using large text samples:
- **English**: Wikipedia (500MB sample, HF dataset 20240601.en)
- **Japanese**: Wikipedia (500MB sample, HF dataset 20240601.ja)

Metrics:
- **Throughput**: MB/s processed with various thread counts
- **Memory**: Peak RSS (Resident Set Size) in MiB
- **Latency**: Processing time for different file sizes

### 3. Comparison Benchmarks

Fair comparison against established baselines:
- **English**: vs NLTK Punkt
- **Japanese**: vs ja_sentence_segmenter

## Data Preparation

Benchmark data is managed by the parent `benchmarks/data/` directory:

```bash
# Prepare all benchmark data (recommended) - from benchmarks/ directory
cd ..
uv run python cli/scripts/prepare_data.py

# Or download specific datasets manually
cd data
uv run python ud_english_ewt/download.py
uv run python ud_japanese_bccwj/download.py

# Wikipedia samples for performance benchmarks (prepared automatically)
# Manual preparation (if needed):
cd ../cli
bash scenarios/performance/prepare_wikipedia_data.sh
```

## Running Benchmarks

### Individual Scenarios

```bash
# English accuracy
bash scenarios/accuracy/english_ewt.sh

# English performance (Wikipedia)
bash scenarios/performance/english_wikipedia_hyperfine.sh

# Japanese performance (Wikipedia)
bash scenarios/performance/japanese_wikipedia_hyperfine.sh

# English vs NLTK Punkt comparison (comprehensive)
bash scenarios/comparison/english_vs_punkt_hyperfine.sh

# Japanese vs ja_sentence_segmenter comparison (comprehensive)
bash scenarios/comparison/japanese_vs_jaseg_hyperfine.sh
```

### Batch Execution

```bash
# Run complete benchmark suite (accuracy + performance + comparison)
bash scenarios/comparison/full_benchmark_suite.sh

# Run with advanced statistical analysis
bash scenarios/comparison/full_benchmark_suite.sh --with-analysis
```

### Custom Hyperfine Parameters

```bash
# More runs for stability
hyperfine --runs 20 --warmup 5 'sakurs-cli segment --input data.txt'

# Export results
hyperfine --export-json results.json 'sakurs-cli segment --input data.txt'
```

## Output Format

### Master Experiment Script Output

When using `run_experiments.sh`, results are organized by timestamp:
```
results/
├── 20250704_143000/
│   ├── metadata.json                    # Experiment metadata
│   ├── throughput_ja_sakurs_1t.json    # Individual result files
│   ├── throughput_ja_sakurs_2t.json
│   ├── throughput_ja_sakurs_4t.json
│   ├── throughput_ja_sakurs_8t.json
│   ├── throughput_ja_jaseg.json        # Baseline comparisons
│   ├── throughput_en_nltk.json
│   ├── memory_ja_sakurs_1t.json
│   ├── memory_ja_sakurs_8t.json
│   ├── accuracy_ja_sakurs.json
│   ├── accuracy_en_sakurs.json
│   ├── aggregated_results.json         # All results aggregated
│   └── results_tables.md               # Formatted markdown tables
```

The `results_tables.md` file contains pre-formatted tables ready for academic papers:
- **Table 1**: Throughput (MB/s) comparison across tools and thread counts
- **Table 2**: Memory usage (MiB) for single and multi-threaded execution  
- **Table 3**: Accuracy metrics (Precision, Recall, F1, Pk, WindowDiff)

#### Example Results Table Format

```markdown
### Table 1: Throughput on 500 MiB Wikipedia (MB/s)

| Lang | Tool | 1 T | 2 T | 4 T | 8 T |
|------|------|-----|-----|-----|-----|
| JA | ja_sentence_segmenter | 45.2 | — | — | — |
| | **Δ-Stack (Ours)** | **120.5** | **235.8** | **412.3** | **485.7** |
| EN | NLTK Punkt | 38.9 | — | — | — |
| | **Δ-Stack (Ours)** | **145.3** | **282.1** | **498.2** | **567.9** |
```

### Traditional Scripts Output

Results from individual scenario scripts:
```
results/
├── 2024-01-15_10-30-00_english_accuracy.json
├── 2024-01-15_10-35-00_english_performance.json
└── 2024-01-15_10-40-00_comparison_report.html
```

## Reproducibility

For academic reproducibility:

1. **Environment**: System specs are automatically captured in `metadata.json`
2. **Data**: Use versioned corpora
   - UD Treebanks: r2.16 (verified during data preparation)
   - Wikipedia: 20240601 dumps with tracked metadata
3. **Seeds**: Deterministic processing (no randomness)
4. **Isolation**: Run with minimal background processes
5. **Version tracking**: All dataset versions and timestamps recorded

The experiment system automatically captures:
- Dataset versions and download timestamps
- Tool versions (sakurs, NLTK, ja_sentence_segmenter)
- System specifications (CPU, memory, OS)
- Exact command-line arguments used
- Number of warmup and test runs

## Integration with Paper

Results are automatically formatted for academic papers:

### Using Master Script Results

The `run_experiments.sh` script generates `results_tables.md` with:
- Pre-formatted markdown tables
- Statistical summaries (mean, std dev)
- Baseline comparisons
- Ready-to-copy format for papers

### Manual Formatting

```bash
# Generate LaTeX tables (from benchmarks/ directory)
cd ..
uv run python cli/scripts/format_results.py --format latex --results-dir cli/results

# Generate plots
uv run python cli/scripts/format_results.py --format plots --results-dir cli/results

# Aggregate multiple experiments
uv run python cli/scripts/aggregate_results.py \
  --input-dir cli/results/20250704_143000 \
  --output results_summary.json
```

### Academic Paper Template Integration

The results match the Δ-Stack Monoid paper template:
- Hardware/Software specifications table
- Dataset information with versions
- Three main results tables (throughput, memory, accuracy)
- Baseline comparison format

## Troubleshooting

### Hyperfine not found
```bash
# Check installation
which hyperfine

# Install if missing
brew install hyperfine  # macOS
cargo install hyperfine # Cross-platform
```

### sakurs not in PATH
```bash
# Add to PATH
export PATH=$PATH:../../target/release

# Or use full path in scripts
/path/to/sakurs process --input data.txt
```

### Memory issues with large files
- Use streaming mode: `--stream` flag
- Reduce sample size in performance benchmarks
- Increase system memory or use swap

## Contributing

When adding new benchmarks:
1. Create scenario script in appropriate directory
2. Update `run_all.sh` to include new scenario
3. Document expected results
4. Ensure reproducibility