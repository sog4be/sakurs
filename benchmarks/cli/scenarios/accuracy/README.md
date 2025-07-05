# Accuracy Benchmarks

This directory contains scripts for measuring the accuracy of Sakurs sentence segmentation on standard datasets.

## Overview

We provide two types of accuracy benchmark scripts:

1. **Basic scripts** (`*_ewt.sh`, `*_gsd.sh`) - Simple accuracy measurement
2. **Hyperfine scripts** (`*_hyperfine.sh`) - Comprehensive benchmarking with performance metrics

## Datasets

- **English**: UD English EWT (Universal Dependencies English Web Treebank)
- **Japanese**: UD Japanese-GSD (Universal Dependencies Japanese GSD corpus)

## Scripts

### Enhanced Hyperfine Benchmarks (Recommended)

#### `english_ewt_hyperfine.sh`
Comprehensive English accuracy benchmark with:
- Accuracy validation with minimum F1 threshold
- Performance measurement using Hyperfine (10 runs, 3 warmups)
- Statistical analysis of timing results
- Combined JSON report with both accuracy and performance metrics
- Colored output and progress tracking

#### `japanese_gsd_hyperfine.sh`
Comprehensive Japanese accuracy benchmark with:
- Special handling for Japanese text characteristics
- Character-based performance metrics (chars/second)
- Full text available for accurate benchmarking
- Text analysis capabilities
- Suitable for accurate evaluation

#### `run_all_hyperfine.sh`
Run all accuracy benchmarks and generate:
- Summary JSON with all results
- Markdown report for human reading
- LaTeX table for academic papers
- Comparison across languages

### Basic Scripts (Legacy)

- `english_ewt.sh` - Basic English accuracy test
- `japanese_gsd.sh` - Basic Japanese accuracy test
- `run_all.sh` - Run all basic benchmarks

## Usage

### Prerequisites

```bash
# Required tools
- sakurs (built and in PATH)
- hyperfine
- python3 with numpy, sklearn
- jq (for JSON processing)
- bc (for calculations)

# Prepare data first
cd ../../scripts
python prepare_data.py
```

### Running Individual Benchmarks

```bash
# English with Hyperfine
./english_ewt_hyperfine.sh

# Japanese with Hyperfine
./japanese_gsd_hyperfine.sh

# Basic versions (no performance metrics)
./english_ewt.sh
./japanese_gsd.sh
```

### Running All Benchmarks

```bash
# Run all Hyperfine benchmarks with summary
./run_all_hyperfine.sh

# Run quietly (less output)
./run_all_hyperfine.sh --quiet
```

## Output

Results are saved to `../../results/accuracy/` with timestamps:

```
results/accuracy/
├── perf_english_ewt_*.json         # Hyperfine performance data
├── accuracy_english_ewt_*.json     # Accuracy metrics
├── combined_english_ewt_*.json     # Combined report
├── summary_*/
│   ├── benchmark_summary.json      # All results
│   ├── benchmark_summary.md        # Markdown report
│   └── benchmark_table.tex         # LaTeX table
```

## Metrics

### Accuracy Metrics
- **Precision**: Correctly predicted boundaries / All predicted boundaries
- **Recall**: Correctly predicted boundaries / All reference boundaries
- **F1 Score**: Harmonic mean of precision and recall
- **Pk**: Probability of segmentation error (simplified)
- **WindowDiff**: Window-based error metric (simplified)

### Performance Metrics
- **Mean Time**: Average processing time
- **Std Dev**: Standard deviation of times
- **Min/Max**: Range of processing times
- **Throughput**: Characters per second (Japanese only)

## Configuration

### Accuracy Thresholds
- English: Minimum F1 = 0.85
- Japanese: Minimum F1 = 0.80 (due to complexity)

### Hyperfine Settings
- Warmup runs: 3
- Benchmark runs: 10
- Shell: Default (with overhead correction)

## Troubleshooting

### "sakurs not found"
```bash
cd ../../../../
cargo build --release --bin sakurs
export PATH=$PATH:$(pwd)/target/release
```

### "hyperfine not found"
```bash
# macOS
brew install hyperfine

# Other platforms
cargo install hyperfine
```

### Low Accuracy for Japanese
- Check if original text is available (licensing issues)
- Text may be reconstructed from tokens
- Consider adjusting threshold in script

## Analysis Tools

Use the analysis script to generate reports and plots:

```bash
cd ../../scripts
python analyze_results.py \
    -i ../results/accuracy/combined_*.json \
    -o ../results/analysis \
    -f all
```

This generates:
- Performance comparison plots
- Accuracy comparison plots
- LaTeX tables for papers
- Markdown reports