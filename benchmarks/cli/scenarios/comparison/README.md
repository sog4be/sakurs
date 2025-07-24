# Comparison Benchmarks

This directory contains scripts for comparing Sakurs against established baseline tools using statistical benchmarking with Hyperfine.

## Overview

Comparison benchmarks provide fair, head-to-head evaluation against industry-standard baselines:

- **English**: Sakurs vs NLTK Punkt Tokenizer
- **Japanese**: Sakurs vs ja_sentence_segmenter

## Baseline Tools

### English: NLTK Punkt
- **What it is**: Statistical sentence boundary detection from NLTK
- **Installation**: `uv pip install nltk` + data download
- **Strengths**: Well-established, widely used, good English performance
- **Use case**: Standard baseline for English sentence segmentation

### Japanese: ja_sentence_segmenter
- **What it is**: Rule-based Japanese sentence segmenter
- **Installation**: `uv pip install ja-sentence-segmenter`
- **Strengths**: Designed specifically for Japanese, handles multiple scripts
- **Use case**: Specialized baseline for Japanese text processing

## Scripts

### Enhanced Hyperfine Comparisons

#### `english_vs_punkt_hyperfine.sh`
Comprehensive English comparison:
- Statistical benchmarking with Hyperfine (10 runs, 3 warmups)
- Both accuracy and performance comparison
- Memory usage profiling for both tools
- Cross-dataset validation (UD EWT + Wikipedia)
- Academic-ready reports

#### `japanese_vs_jaseg_hyperfine.sh`
Comprehensive Japanese comparison:
- Character-based metrics comparison
- Multi-script analysis (hiragana, katakana, kanji)
- Memory and resource usage tracking
- Statistical significance testing
- Handles text reconstruction complexities

### Cross-Dataset Scenarios

#### `cross_dataset_english.sh`
Test English tools across multiple datasets:
- UD English EWT (accuracy focus)
- Wikipedia English (performance focus)
- Consistency analysis across data types
- Performance vs accuracy trade-offs

#### `cross_dataset_japanese.sh`
Test Japanese tools across datasets:
- UD Japanese-GSD (accuracy focus)
- Wikipedia Japanese (performance focus)
- Script distribution analysis
- Robustness testing

### Integration Scenarios

#### `full_benchmark_suite.sh`
Complete benchmarking pipeline:
- Run all accuracy benchmarks
- Run all performance benchmarks
- Run all comparison benchmarks
- Generate unified report
- Statistical analysis across all results

#### `run_all_comparisons.sh`
Execute all comparison scenarios:
- Enhanced baseline comparisons
- Cross-dataset validation
- Summary reporting
- Visualization generation

## Usage

### Prerequisites

```bash
# Install baseline tools
uv pip install nltk ja-sentence-segmenter

# Download NLTK data
python -c "import nltk; nltk.download('punkt')"

# Ensure all datasets are prepared
python ../../scripts/prepare_data.py
bash ../performance/prepare_wikipedia_data.sh
```

### Running Individual Comparisons

```bash
# English vs NLTK Punkt (comprehensive)
./english_vs_punkt_hyperfine.sh

# Japanese vs ja_sentence_segmenter (comprehensive)
./japanese_vs_jaseg_hyperfine.sh

# Cross-dataset English comparison
./cross_dataset_english.sh

# Cross-dataset Japanese comparison
./cross_dataset_japanese.sh
```

### Running Complete Suite

```bash
# All comparison benchmarks
./run_all_comparisons.sh

# Complete benchmark suite (accuracy + performance + comparison)
./full_benchmark_suite.sh

# Generate detailed analysis
./full_benchmark_suite.sh --with-analysis
```

## Output

Results are saved to `../../results/comparison/` with timestamps:

```
results/comparison/
├── english_vs_punkt_*.json           # English comparison results
├── japanese_vs_jaseg_*.json          # Japanese comparison results
├── cross_dataset_*.json              # Cross-dataset analysis
├── statistical_analysis_*.json       # Significance testing
├── summary_*/
│   ├── comparison_summary.json       # All comparison results
│   ├── comparison_report.md          # Human-readable report
│   ├── baseline_comparison.tex       # LaTeX table
│   ├── performance_comparison.png    # Performance charts
│   └── accuracy_comparison.png       # Accuracy charts
```

## Metrics

### Performance Comparison
- **Throughput**: MB/s, chars/s processing rates
- **Latency**: Processing time statistics (mean, p50, p90, p99)
- **Memory**: Peak and average memory usage
- **Speedup**: Relative performance improvement

### Accuracy Comparison
- **Precision/Recall/F1**: Standard classification metrics
- **Boundary Metrics**: Pk and WindowDiff scores
- **Statistical Significance**: p-values and confidence intervals
- **Agreement Analysis**: Inter-tool agreement rates

### Resource Comparison
- **Memory Efficiency**: Peak memory usage comparison
- **CPU Utilization**: Processor usage patterns
- **Scalability**: Performance across different data sizes
- **Stability**: Variance in processing times

## Configuration

### Environment Variables
- `COMPARISON_RUNS`: Number of benchmark runs (default: 10)
- `COMPARISON_WARMUP`: Number of warmup runs (default: 3)
- `MEMORY_PROFILING`: Enable memory tracking (default: true)
- `STATISTICAL_TESTING`: Enable significance tests (default: true)

### Baseline Configuration
```bash
# Test with different baseline configurations
export PUNKT_LANGUAGE="english"  # or other NLTK language models
export JASEG_PIPELINE="standard"  # or "strict", "relaxed"
```

## Interpreting Results

### Performance Results
- **Speedup > 1.0**: Sakurs is faster than baseline
- **Throughput**: Higher values indicate better performance
- **Memory**: Lower peak usage indicates better efficiency
- **Latency p99**: Important for worst-case performance

### Accuracy Results
- **F1 Score**: Higher is better (0.0-1.0 scale)
- **Pk Score**: Lower is better (error rate)
- **Statistical Significance**: p < 0.05 indicates significant difference
- **Confidence Intervals**: Overlap indicates similar performance

### Academic Reporting
All comparison scripts generate:
- **LaTeX Tables**: Ready for academic papers
- **Statistical Tests**: P-values and confidence intervals
- **Effect Sizes**: Practical significance measures
- **Reproducibility Info**: Versions, seeds, environment details

## Troubleshooting

### Baseline Installation Issues
```bash
# NLTK punkt data missing
python -c "import nltk; nltk.download('punkt', quiet=False)"

# ja_sentence_segmenter not found
uv pip install ja-sentence-segmenter

# Version conflicts
uv pip list | grep -E "(nltk|ja-sentence-segmenter)"
```

### Memory Issues
```bash
# Reduce sample sizes for memory-constrained systems
export SAMPLE_SIZE_MB=100

# Skip memory profiling if causing issues
export MEMORY_PROFILING=false
```

### Statistical Analysis
```bash
# Increase runs for more stable statistical tests
export COMPARISON_RUNS=20

# Generate detailed statistical report
python statistical_analysis.py --input results/comparison/*.json
```

## Best Practices

1. **Fair Comparison**
   - Use identical input data for all tools
   - Same hardware and system conditions
   - Document all tool versions
   - Account for warmup effects

2. **Statistical Rigor**
   - Multiple runs (10+) for significance testing
   - Report confidence intervals
   - Test for normality before t-tests
   - Consider effect sizes, not just p-values

3. **Reproducibility**
   - Fix random seeds where applicable
   - Document system specifications
   - Record baseline tool versions
   - Use containerized environments when possible

## Analysis Tools

Generate detailed statistical analysis:

```bash
cd ../../scripts
python analyze_results.py \
    --input ../results/comparison/*.json \
    --baseline-analysis \
    --statistical-tests \
    --output ../results/analysis
```

This produces:
- Statistical significance tests
- Effect size calculations
- Performance regression analysis
- Academic-ready figures and tables