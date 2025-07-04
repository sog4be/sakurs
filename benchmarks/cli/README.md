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
│   ├── prepare_data.py      # Data preparation utilities
│   ├── evaluate_accuracy.py  # Accuracy evaluation
│   └── format_results.py     # Result formatting
└── results/           # Benchmark results (git-ignored)
```

## Requirements

- Hyperfine (for benchmarking)
- Python 3.8+ (for evaluation scripts)
- sakurs-cli (built and in PATH)
- Baseline tools (NLTK, ja_sentence_segmenter)

## Installation

```bash
# Install Hyperfine
brew install hyperfine  # macOS
# or see https://github.com/sharkdp/hyperfine for other platforms

# Install Python dependencies
pip install -r requirements.txt

# Build sakurs
cd ../../
cargo build --release --bin sakurs
export PATH=$PATH:$(pwd)/target/release
```

## Quick Start

```bash
# Prepare benchmark data
python scripts/prepare_data.py

# Run all accuracy benchmarks
bash scenarios/accuracy/run_all.sh

# Run all performance benchmarks
bash scenarios/performance/run_all.sh

# Generate comparison report
python scenarios/comparison/generate_report.py
```

## Benchmark Types

### 1. Accuracy Benchmarks

Measure segmentation accuracy using annotated corpora:
- **English**: UD English EWT
- **Japanese**: UD Japanese-BCCWJ

Metrics: Precision, Recall, F1, Pk, WindowDiff

### 2. Performance Benchmarks

Measure throughput and latency using large text samples:
- **English**: Wikipedia dump (500MB sample)
- **Japanese**: Wikipedia dump (500MB sample)

Metrics: Throughput (MB/s), Latency, Memory usage

### 3. Comparison Benchmarks

Fair comparison against established baselines:
- **English**: vs NLTK Punkt
- **Japanese**: vs ja_sentence_segmenter

## Data Preparation

Benchmark data is managed by the parent `benchmarks/data/` directory:

```bash
# Download all required corpora
cd ../data
make download-all

# Or download specific datasets
python ud_english_ewt/download.py
python ud_japanese_bccwj/download.py
python wikipedia/download.py --language en --size 500MB
python wikipedia/download.py --language ja --size 500MB
```

## Running Benchmarks

### Individual Scenarios

```bash
# English accuracy
bash scenarios/accuracy/english_ewt.sh

# Japanese performance
bash scenarios/performance/japanese_wikipedia.sh

# Comparison with NLTK
bash scenarios/comparison/english_vs_punkt.sh
```

### Batch Execution

```bash
# Run all benchmarks and generate report
make benchmark-all
```

### Custom Hyperfine Parameters

```bash
# More runs for stability
hyperfine --runs 20 --warmup 5 'sakurs-cli segment --input data.txt'

# Export results
hyperfine --export-json results.json 'sakurs-cli segment --input data.txt'
```

## Output Format

Results are saved in `results/` with timestamp:
```
results/
├── 2024-01-15_10-30-00_english_accuracy.json
├── 2024-01-15_10-35-00_english_performance.json
└── 2024-01-15_10-40-00_comparison_report.html
```

## Reproducibility

For academic reproducibility:

1. **Environment**: Document system specs in results
2. **Data**: Use versioned corpora (UD r2.16)
3. **Seeds**: Set random seeds where applicable
4. **Isolation**: Run with minimal background processes

## Integration with Paper

Results can be formatted for academic papers:

```bash
# Generate LaTeX tables
python scripts/format_results.py --format latex

# Generate plots
python scripts/format_results.py --format plots
```

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