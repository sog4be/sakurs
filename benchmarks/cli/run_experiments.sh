#!/usr/bin/env bash
set -euo pipefail

# Master experiment script for Δ-Stack Monoid benchmarks
# This script runs all benchmarks in a unified manner and collects results

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default values
OUTPUT_DIR="results/$(date +%Y%m%d_%H%M%S)"
THREADS=(1 2 4 8)
WARMUP_RUNS=1
TEST_RUNS=3
PREPARE_DATA=false
SKIP_THROUGHPUT=false
SKIP_MEMORY=false
SKIP_ACCURACY=false

# Function to print colored messages
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
usage() {
    cat << EOF
Usage: $0 [options]

Options:
  -o, --output-dir DIR      Output directory for results (default: results/YYYYMMDD_HHMMSS)
  -t, --threads LIST        Comma-separated list of thread counts (default: 1,2,4,8)
  -w, --warmup-runs NUM     Number of warmup runs (default: 1)
  -r, --test-runs NUM       Number of test runs to average (default: 3)
  -p, --prepare-data        Prepare/download datasets before running
  --skip-throughput         Skip throughput benchmarks
  --skip-memory            Skip memory benchmarks
  --skip-accuracy          Skip accuracy benchmarks
  -h, --help               Show this help message

Examples:
  # Run all benchmarks with default settings
  ./run_experiments.sh

  # Run only throughput tests with specific threads
  ./run_experiments.sh --skip-memory --skip-accuracy -t 1,4,8

  # Prepare data and run all experiments
  ./run_experiments.sh --prepare-data -o results/my_experiment
EOF
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -t|--threads)
            IFS=',' read -ra THREADS <<< "$2"
            shift 2
            ;;
        -w|--warmup-runs)
            WARMUP_RUNS="$2"
            shift 2
            ;;
        -r|--test-runs)
            TEST_RUNS="$2"
            shift 2
            ;;
        -p|--prepare-data)
            PREPARE_DATA=true
            shift
            ;;
        --skip-throughput)
            SKIP_THROUGHPUT=true
            shift
            ;;
        --skip-memory)
            SKIP_MEMORY=true
            shift
            ;;
        --skip-accuracy)
            SKIP_ACCURACY=true
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            ;;
    esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"
print_info "Results will be saved to: $OUTPUT_DIR"

# Prepare datasets if requested
if [ "$PREPARE_DATA" = true ]; then
    print_info "Preparing datasets..."
    uv run python scripts/prepare_data.py
    print_success "Datasets prepared"
fi

# Check if sakurs exists
if ! command -v sakurs &> /dev/null; then
    print_error "sakurs not found. Please build and install it first:"
    print_error "  cd ../.. && cargo build --release"
    print_error "  export PATH=\$PATH:$(pwd)/../../target/release"
    exit 1
fi

# Get sakurs version
SAKURS_VERSION=$(sakurs --version 2>/dev/null || echo "unknown")
print_info "Using sakurs version: $SAKURS_VERSION"

# Function to run throughput benchmarks
run_throughput_benchmarks() {
    local lang=$1
    local dataset=$2
    local tool=$3
    local input_file=$4
    
    print_info "Running throughput benchmark: $tool on $dataset ($lang)"
    
    if [ "$tool" = "sakurs" ]; then
        for threads in "${THREADS[@]}"; do
            print_info "  Testing with $threads threads..."
            
            # Build command
            lang_flag=""
            if [ "$lang" = "ja" ]; then
                lang_flag="--language japanese"
            fi
            
            # Run benchmark
            uv run python -c "
import sys
import time
import subprocess
import os
sys.path.insert(0, 'scripts')
from metrics import MetricsMeasurer, BenchmarkResult, ThroughputMetrics

measurer = MetricsMeasurer()
try:
    # Get file size
    file_size_bytes = os.path.getsize('$input_file')
    file_size_mb = file_size_bytes / (1024 * 1024)
    
    # Warmup runs
    for _ in range($WARMUP_RUNS):
        subprocess.run(
            ['sakurs', 'process', '--input', '$input_file'] + '$lang_flag'.split(),
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )
    
    # Test runs
    durations = []
    for _ in range($TEST_RUNS):
        start_time = time.time()
        result = subprocess.run(
            ['sakurs', 'process', '--input', '$input_file'] + '$lang_flag'.split(),
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
        )
        end_time = time.time()
        
        if result.returncode == 0:
            durations.append(end_time - start_time)
    
    # Calculate average
    if durations:
        avg_duration = sum(durations) / len(durations)
        throughput = file_size_mb / avg_duration if avg_duration > 0 else 0
        result = ThroughputMetrics(
            duration_seconds=avg_duration,
            data_size_mb=file_size_mb,
            throughput_mbps=throughput,
            num_threads=$threads
        )
    else:
        raise RuntimeError('All benchmark runs failed')
    
    br = BenchmarkResult(
        tool='sakurs',
        language='$lang',
        dataset='$dataset',
        num_threads=$threads,
        throughput=result
    )
    
    # Save individual result
    measurer.save_results([br], '$OUTPUT_DIR/throughput_${lang}_${tool}_${threads}t.json')
    
    print(f'  Throughput: {result.throughput_mbps:.2f} MB/s')
except Exception as e:
    print(f'  Error: {e}')
"
        done
    else
        # Baseline tools (single-threaded only)
        print_info "  Testing baseline tool (single-threaded)..."
        
        # Determine command based on tool
        case "$tool" in
            "nltk")
                cmd_module="baselines.nltk_punkt.cli"
                ;;
            "ja_seg")
                cmd_module="baselines.ja_sentence_segmenter.cli"
                ;;
            *)
                print_error "Unknown tool: $tool"
                return
                ;;
        esac
        
        # Run benchmark
        uv run python -c "
import sys
import subprocess
sys.path.insert(0, 'scripts')
from metrics import MetricsMeasurer, BenchmarkResult

measurer = MetricsMeasurer()
try:
    result = measurer.run_throughput_benchmark(
        command=['uv', 'run', 'python', '-m', '$cmd_module', '--input', '-'],
        input_file='$input_file',
        num_threads=1,
        warmup_runs=$WARMUP_RUNS,
        test_runs=$TEST_RUNS
    )
    
    br = BenchmarkResult(
        tool='$tool',
        language='$lang',
        dataset='$dataset',
        num_threads=1,
        throughput=result
    )
    
    # Save individual result
    measurer.save_results([br], '$OUTPUT_DIR/throughput_${lang}_${tool}_1t.json')
    
    print(f'  Throughput: {result.throughput_mbps:.2f} MB/s')
except Exception as e:
    print(f'  Error: {e}')
"
    fi
}

# Function to run memory benchmarks
run_memory_benchmarks() {
    local lang=$1
    local dataset=$2
    local tool=$3
    local input_file=$4
    
    print_info "Running memory benchmark: $tool on $dataset ($lang)"
    
    if [ "$tool" = "sakurs" ]; then
        # Test with 1 and 8 threads for memory
        for threads in 1 8; do
            print_info "  Testing with $threads threads..."
            
            # Build command
            lang_flag=""
            if [ "$lang" = "ja" ]; then
                lang_flag="--language japanese"
            fi
            
            # Run benchmark
            uv run python -c "
import sys
sys.path.insert(0, 'scripts')
from metrics import MetricsMeasurer, BenchmarkResult

measurer = MetricsMeasurer()
try:
    # For sakurs, we need to pass the file path directly
    cmd = ['sakurs', 'process', '--input', '$input_file']
    if '$lang_flag':
        cmd.extend('$lang_flag'.split())
    
    result = measurer.enhanced_measure_memory_peak(
        command=cmd,
        input_file=None  # Don't use stdin for sakurs
    )
    
    br = BenchmarkResult(
        tool='sakurs',
        language='$lang',
        dataset='$dataset',
        num_threads=$threads,
        memory=result
    )
    
    # Save individual result
    measurer.save_results([br], '$OUTPUT_DIR/memory_${lang}_${tool}_${threads}t.json')
    
    print(f'  Peak memory: {result.peak_memory_mb:.2f} MB')
except Exception as e:
    print(f'  Error: {e}')
"
        done
    else
        # Baseline tools
        print_info "  Testing baseline tool..."
        
        # Determine command based on tool
        case "$tool" in
            "nltk")
                cmd_module="baselines.nltk_punkt.cli"
                ;;
            "ja_seg")
                cmd_module="baselines.ja_sentence_segmenter.cli"
                ;;
            *)
                print_error "Unknown tool: $tool"
                return
                ;;
        esac
        
        # Run benchmark
        uv run python -c "
import sys
sys.path.insert(0, 'scripts')
from metrics import MetricsMeasurer, BenchmarkResult

measurer = MetricsMeasurer()
try:
    result = measurer.enhanced_measure_memory_peak(
        command=['uv', 'run', 'python', '-m', '$cmd_module', '--input', '-'],
        input_file='$input_file'
    )
    
    br = BenchmarkResult(
        tool='$tool',
        language='$lang',
        dataset='$dataset',
        num_threads=1,
        memory=result
    )
    
    # Save individual result
    measurer.save_results([br], '$OUTPUT_DIR/memory_${lang}_${tool}_1t.json')
    
    print(f'  Peak memory: {result.peak_memory_mb:.2f} MB')
except Exception as e:
    print(f'  Error: {e}')
"
    fi
}

# Function to run accuracy benchmarks
run_accuracy_benchmarks() {
    local lang=$1
    local dataset=$2
    local tool=$3
    local test_file=$4
    
    print_info "Running accuracy benchmark: $tool on $dataset ($lang)"
    
    # Run accuracy evaluation script
    uv run python scripts/evaluate_tool_accuracy.py \
        --language "$lang" \
        --tool "$tool" \
        --test-file "$test_file" \
        --output "$OUTPUT_DIR/accuracy_${lang}_${tool}.json"
}

# Main experiment execution
print_info "Starting experiments..."

# Create experiment metadata
cat > "$OUTPUT_DIR/metadata.json" << EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "sakurs_version": "$SAKURS_VERSION",
    "platform": "$(uname -s)",
    "arch": "$(uname -m)", 
    "threads_tested": [${THREADS[@]}],
    "warmup_runs": $WARMUP_RUNS,
    "test_runs": $TEST_RUNS
}
EOF

# Throughput benchmarks
if [ "$SKIP_THROUGHPUT" = false ]; then
    print_info "Running throughput benchmarks..."
    
    # Japanese Wikipedia
    if [ -f "../data/cache/wikipedia_ja_500mb_20231101.txt" ]; then
        run_throughput_benchmarks "ja" "wikipedia" "sakurs" "../data/cache/wikipedia_ja_500mb_20231101.txt"
        run_throughput_benchmarks "ja" "wikipedia" "ja_seg" "../data/cache/wikipedia_ja_500mb_20231101.txt"
    else
        print_warning "Japanese Wikipedia sample not found, skipping..."
    fi
    
    # English Wikipedia
    if [ -f "../data/cache/wikipedia_en_500mb_20231101.txt" ]; then
        run_throughput_benchmarks "en" "wikipedia" "sakurs" "../data/cache/wikipedia_en_500mb_20231101.txt"
        run_throughput_benchmarks "en" "wikipedia" "nltk" "../data/cache/wikipedia_en_500mb_20231101.txt"
    else
        print_warning "English Wikipedia sample not found, skipping..."
    fi
fi

# Memory benchmarks
if [ "$SKIP_MEMORY" = false ]; then
    print_info "Running memory benchmarks..."
    
    # Japanese Wikipedia
    if [ -f "../data/cache/wikipedia_ja_500mb_20231101.txt" ]; then
        run_memory_benchmarks "ja" "wikipedia" "sakurs" "../data/cache/wikipedia_ja_500mb_20231101.txt"
        run_memory_benchmarks "ja" "wikipedia" "ja_seg" "../data/cache/wikipedia_ja_500mb_20231101.txt"
    else
        print_warning "Japanese Wikipedia sample not found, skipping..."
    fi
    
    # English Wikipedia
    if [ -f "../data/cache/wikipedia_en_500mb_20231101.txt" ]; then
        run_memory_benchmarks "en" "wikipedia" "sakurs" "../data/cache/wikipedia_en_500mb_20231101.txt"
        run_memory_benchmarks "en" "wikipedia" "nltk" "../data/cache/wikipedia_en_500mb_20231101.txt"
    else
        print_warning "English Wikipedia sample not found, skipping..."
    fi
fi

# Accuracy benchmarks
if [ "$SKIP_ACCURACY" = false ]; then
    print_info "Running accuracy benchmarks..."
    
    # UD Japanese-GSD
    if [ -f "../data/ud_japanese_gsd/cli_format/gsd_plain.txt" ]; then
        run_accuracy_benchmarks "ja" "ud_gsd" "sakurs" "../data/ud_japanese_gsd/cli_format/gsd_plain.txt"
        run_accuracy_benchmarks "ja" "ud_gsd" "ja_seg" "../data/ud_japanese_gsd/cli_format/gsd_plain.txt"
    else
        print_warning "UD Japanese-GSD test set not found, skipping..."
    fi
    
    # UD English-EWT
    if [ -f "../data/ud_english_ewt/cli_format/ewt_plain.txt" ]; then
        run_accuracy_benchmarks "en" "ud_ewt" "sakurs" "../data/ud_english_ewt/cli_format/ewt_plain.txt"
        run_accuracy_benchmarks "en" "ud_ewt" "nltk" "../data/ud_english_ewt/cli_format/ewt_plain.txt"
    else
        print_warning "UD English-EWT test set not found, skipping..."
    fi
fi

# Aggregate results
print_info "Aggregating results..."
uv run python scripts/aggregate_results.py \
    --input-dir "$OUTPUT_DIR" \
    --output "$OUTPUT_DIR/aggregated_results.json" \
    --template "$OUTPUT_DIR/results_tables.md"

print_success "All experiments completed!"
print_success "Results saved to: $OUTPUT_DIR"
print_info "View aggregated results: $OUTPUT_DIR/results_tables.md"