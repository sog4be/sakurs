# Performance Benchmarks

This directory contains scripts for measuring the performance of Sakurs sentence segmentation on large-scale text corpora.

## Overview

Performance benchmarks measure throughput, latency, and resource usage on real-world datasets:

- **English**: Wikipedia 500MB sample (HF dataset: 20231101.en)
- **Japanese**: Wikipedia 500MB sample (HF dataset: 20231101.ja)

## Scripts

### Data Preparation

#### `prepare_wikipedia_data.sh`
Prepares Wikipedia samples for benchmarking:
- Downloads samples via Hugging Face datasets
- Creates 500MB samples for each language
- Ensures reproducibility with fixed versions

### Benchmark Scripts

#### `english_wikipedia_hyperfine.sh`
Performance benchmark for English Wikipedia:
- Measures throughput (MB/s)
- Tracks memory usage
- Reports latency statistics (mean, p50, p90, p99)
- Supports streaming mode for large files

#### `japanese_wikipedia_hyperfine.sh`
Performance benchmark for Japanese Wikipedia:
- Character-based throughput (chars/s)
- Japanese-specific text analysis
- Memory-efficient processing
- Handles multi-script text (hiragana, katakana, kanji)

#### `run_all_hyperfine.sh`
Executes all performance benchmarks:
- Runs benchmarks for all languages
- Generates combined performance report
- Creates comparison visualizations
- Produces academic-ready tables

### Helper Scripts

#### `measure_resources.sh`
Resource monitoring during benchmarks:
- CPU usage tracking
- Memory consumption
- I/O statistics
- System load metrics

## Usage

### Prerequisites

```bash
# Required tools
- sakurs (built and in PATH)
- hyperfine
- python3 with required packages
- datasets (pip install datasets)

# Prepare Wikipedia data first
./prepare_wikipedia_data.sh
```

### Running Individual Benchmarks

```bash
# English Wikipedia performance
./english_wikipedia_hyperfine.sh

# Japanese Wikipedia performance
./japanese_wikipedia_hyperfine.sh

# With custom parameters
BENCHMARK_RUNS=20 ./english_wikipedia_hyperfine.sh
```

### Running All Benchmarks

```bash
# Run complete performance suite
./run_all_hyperfine.sh

# Generate detailed report
./run_all_hyperfine.sh --detailed
```

## Output

Results are saved to `../../results/performance/` with timestamps:

```
results/performance/
├── perf_english_wikipedia_*.json    # Hyperfine raw data
├── throughput_english_*.json        # Throughput analysis
├── memory_english_*.json            # Memory usage data
├── combined_performance_*.json      # Combined metrics
├── summary_*/
│   ├── performance_summary.json     # All results
│   ├── performance_report.md        # Markdown report
│   ├── throughput_comparison.png    # Visualization
│   └── performance_table.tex        # LaTeX table
```

## Metrics

### Throughput Metrics
- **MB/s**: Megabytes processed per second
- **chars/s**: Characters processed per second (Japanese)
- **sentences/s**: Sentences identified per second

### Latency Metrics
- **Mean**: Average processing time
- **Std Dev**: Standard deviation
- **Percentiles**: p50, p90, p95, p99
- **Min/Max**: Range of processing times

### Resource Metrics
- **Peak Memory**: Maximum memory usage (MB)
- **Average Memory**: Mean memory usage during run
- **CPU Usage**: Processor utilization percentage

## Configuration

### Environment Variables
- `SAMPLE_SIZE_MB`: Wikipedia sample size (default: 500)
- `BENCHMARK_RUNS`: Number of Hyperfine runs (default: 10)
- `WARMUP_RUNS`: Number of warmup iterations (default: 3)
- `STREAMING_MODE`: Enable streaming for large files (default: auto)

### Performance Tuning
```bash
# Increase runs for more stable results
export BENCHMARK_RUNS=20

# Use streaming for memory-constrained systems
export STREAMING_MODE=true

# Custom sample size
export SAMPLE_SIZE_MB=1000
```

## Troubleshooting

### Out of Memory Errors
```bash
# Enable streaming mode
export STREAMING_MODE=true

# Reduce sample size
export SAMPLE_SIZE_MB=100
```

### Slow Download Speeds
- Check internet connection
- Use cached samples if available
- Consider using smaller sample size

### Inconsistent Results
- Ensure minimal background processes
- Increase warmup runs
- Use CPU performance mode if available

## Best Practices

1. **System Preparation**
   - Close unnecessary applications
   - Disable system updates during benchmarks
   - Use consistent power settings

2. **Reproducibility**
   - Document system specifications
   - Use fixed dataset versions
   - Record environment variables

3. **Statistical Validity**
   - Run multiple iterations (10+)
   - Include warmup runs
   - Report confidence intervals

## Analysis Tools

Generate detailed analysis:

```bash
cd ../../scripts
python analyze_results.py \
    -i ../results/performance/combined_*.json \
    -o ../results/analysis \
    -f all \
    --performance-focus
```

This generates:
- Throughput comparison charts
- Memory usage graphs
- Statistical analysis reports
- Academic-ready figures