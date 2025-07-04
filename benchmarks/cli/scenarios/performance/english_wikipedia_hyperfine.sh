#!/bin/bash
# Performance benchmark for English Wikipedia using Hyperfine
# Measures throughput, latency, and resource usage on 500MB sample

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/wikipedia/cli_format"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results/performance"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
SAMPLE_SIZE_MB=${SAMPLE_SIZE_MB:-500}
BENCHMARK_RUNS=${BENCHMARK_RUNS:-10}
WARMUP_RUNS=${WARMUP_RUNS:-3}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Functions
print_status() {
    echo -e "${GREEN}[$(date +%H:%M:%S)]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}===============================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===============================================${NC}"
    echo ""
}

check_prerequisites() {
    local missing=0
    
    # Check if data is prepared
    local data_file="$DATA_DIR/wikipedia_en_${SAMPLE_SIZE_MB}mb.txt"
    if [ ! -f "$data_file" ]; then
        print_error "Wikipedia English data not found: $data_file"
        echo "Please run prepare_wikipedia_data.sh first"
        missing=1
    fi
    
    # Check if sakurs is available
    if ! command -v sakurs &> /dev/null; then
        print_error "sakurs not found in PATH"
        echo "Please build and add to PATH:"
        echo "  cd $ROOT_DIR && cargo build --release --bin sakurs"
        echo "  export PATH=\$PATH:$ROOT_DIR/target/release"
        missing=1
    fi
    
    # Check if hyperfine is available
    if ! command -v hyperfine &> /dev/null; then
        print_error "hyperfine not found. Please install it:"
        echo "  brew install hyperfine  # macOS"
        echo "  cargo install hyperfine # Cross-platform"
        missing=1
    fi
    
    return $missing
}

measure_baseline_metrics() {
    local input_file=$1
    
    print_status "Measuring baseline metrics..."
    
    # File size in bytes and MB
    local size_bytes=$(wc -c < "$input_file" | tr -d ' ')
    local size_mb=$(echo "scale=2; $size_bytes / 1024 / 1024" | bc)
    
    # Character and line count
    local char_count=$(wc -m < "$input_file" | tr -d ' ')
    local line_count=$(wc -l < "$input_file" | tr -d ' ')
    
    echo "  File size: ${size_mb}MB (${size_bytes} bytes)"
    echo "  Characters: ${char_count}"
    echo "  Lines: ${line_count}"
    
    # Export for later use
    export PERF_SIZE_BYTES=$size_bytes
    export PERF_SIZE_MB=$size_mb
    export PERF_CHAR_COUNT=$char_count
}

run_memory_profiling() {
    local input_file=$1
    local output_file=$2
    
    print_status "Running memory profiling..."
    
    # Use time command to measure memory (macOS compatible)
    if command -v gtime &> /dev/null; then
        TIME_CMD="gtime"
    else
        TIME_CMD="time"
    fi
    
    # Run with memory tracking
    local mem_output="$RESULTS_DIR/memory_english_${TIMESTAMP}.txt"
    
    $TIME_CMD -v sakurs process \
        --input "$input_file" \
        --output "$output_file" \
        --format sentences \
        --language english \
        2>&1 | tee "$mem_output" | grep -E "(Maximum resident|User time|System time)" || true
    
    # Extract memory usage if available
    if grep -q "Maximum resident" "$mem_output" 2>/dev/null; then
        local max_memory=$(grep "Maximum resident" "$mem_output" | awk '{print $NF}')
        echo "  Maximum memory: $max_memory"
    fi
}

# Main execution
main() {
    print_header "English Wikipedia Performance Benchmark"
    echo "Timestamp: $TIMESTAMP"
    echo "Sample size: ${SAMPLE_SIZE_MB}MB"
    echo "Benchmark runs: $BENCHMARK_RUNS (with $WARMUP_RUNS warmup)"
    echo "Results directory: $RESULTS_DIR"
    
    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi
    
    # Input/output files
    local input_file="$DATA_DIR/wikipedia_en_${SAMPLE_SIZE_MB}mb.txt"
    local output_file="$RESULTS_DIR/temp_output_en_${TIMESTAMP}.txt"
    
    # Step 1: Baseline metrics
    print_header "Step 1: Baseline Metrics"
    measure_baseline_metrics "$input_file"
    
    # Step 2: Memory profiling (single run)
    print_header "Step 2: Memory Profiling"
    run_memory_profiling "$input_file" "$output_file"
    
    # Step 3: Performance benchmark with Hyperfine
    print_header "Step 3: Performance Benchmark"
    print_status "Running Hyperfine benchmark..."
    
    hyperfine \
        --warmup "$WARMUP_RUNS" \
        --runs "$BENCHMARK_RUNS" \
        --export-json "$RESULTS_DIR/perf_english_wikipedia_${TIMESTAMP}.json" \
        --export-markdown "$RESULTS_DIR/perf_english_wikipedia_${TIMESTAMP}.md" \
        --show-output \
        --command-name "sakurs-english-wikipedia" \
        "sakurs process --input '$input_file' --output '$output_file' --format sentences --language english"
    
    # Step 4: Calculate throughput metrics
    print_header "Step 4: Throughput Analysis"
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json
import os

# Load Hyperfine results
with open("$RESULTS_DIR/perf_english_wikipedia_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

# Get metrics from environment
size_bytes = int(os.environ['PERF_SIZE_BYTES'])
size_mb = float(os.environ['PERF_SIZE_MB'])
char_count = int(os.environ['PERF_CHAR_COUNT'])

# Calculate throughput
mean_time = perf_data["results"][0]["mean"]
throughput_mb_s = size_mb / mean_time
throughput_chars_s = char_count / mean_time

# Count output sentences
try:
    with open("$output_file", 'r') as f:
        sentence_count = sum(1 for line in f if line.strip())
    sentences_per_sec = sentence_count / mean_time
except:
    sentence_count = 0
    sentences_per_sec = 0

print(f"Performance Metrics:")
print(f"  Mean processing time: {mean_time:.3f}s")
print(f"  Throughput: {throughput_mb_s:.2f} MB/s")
print(f"  Character rate: {throughput_chars_s:,.0f} chars/s")
print(f"  Sentence rate: {sentences_per_sec:,.0f} sentences/s")
print(f"  Total sentences: {sentence_count:,}")

# Calculate percentiles
times = perf_data["results"][0]["times"]
times.sort()
p50 = times[len(times)//2]
p90 = times[int(len(times)*0.9)]
p99 = times[int(len(times)*0.99)] if len(times) > 10 else times[-1]

print(f"\nLatency Percentiles:")
print(f"  p50: {p50:.3f}s")
print(f"  p90: {p90:.3f}s") 
print(f"  p99: {p99:.3f}s")

# Save throughput analysis
throughput_data = {
    "benchmark": "english_wikipedia_performance",
    "timestamp": "$TIMESTAMP",
    "input": {
        "file": "$input_file",
        "size_mb": size_mb,
        "size_bytes": size_bytes,
        "characters": char_count
    },
    "performance": {
        "mean_time": mean_time,
        "throughput_mb_s": throughput_mb_s,
        "throughput_chars_s": throughput_chars_s,
        "sentences_per_sec": sentences_per_sec,
        "total_sentences": sentence_count
    },
    "latency_percentiles": {
        "p50": p50,
        "p90": p90,
        "p99": p99
    }
}

with open("$RESULTS_DIR/throughput_english_wikipedia_${TIMESTAMP}.json", "w") as f:
    json.dump(throughput_data, f, indent=2)
EOF
    
    # Step 5: Generate combined report
    print_header "Step 5: Combined Report Generation"
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json

# Load all data
with open("$RESULTS_DIR/perf_english_wikipedia_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

with open("$RESULTS_DIR/throughput_english_wikipedia_${TIMESTAMP}.json") as f:
    throughput_data = json.load(f)

# Create combined report
combined = {
    "benchmark": "english_wikipedia_performance",
    "timestamp": "$TIMESTAMP",
    "configuration": {
        "sample_size_mb": $SAMPLE_SIZE_MB,
        "benchmark_runs": $BENCHMARK_RUNS,
        "warmup_runs": $WARMUP_RUNS
    },
    "dataset": {
        "language": "English",
        "source": "Wikipedia (HF 20231101.en)",
        "size_mb": throughput_data["input"]["size_mb"],
        "characters": throughput_data["input"]["characters"]
    },
    "performance": {
        "mean_time": perf_data["results"][0]["mean"],
        "stddev": perf_data["results"][0]["stddev"],
        "min": perf_data["results"][0]["min"],
        "max": perf_data["results"][0]["max"],
        "runs": len(perf_data["results"][0]["times"])
    },
    "throughput": throughput_data["performance"],
    "latency_percentiles": throughput_data["latency_percentiles"],
    "system": {
        "command": perf_data["results"][0]["command"],
        "hyperfine_version": perf_data.get("hyperfine_version", "unknown")
    }
}

# Save combined report
with open("$RESULTS_DIR/combined_english_wikipedia_${TIMESTAMP}.json", "w") as f:
    json.dump(combined, f, indent=2)

print("Combined report saved successfully")
EOF
    
    # Cleanup
    rm -f "$output_file"
    
    print_header "Benchmark Completed Successfully!"
    echo "Results saved to:"
    echo "  - Performance: $RESULTS_DIR/perf_english_wikipedia_${TIMESTAMP}.json"
    echo "  - Throughput: $RESULTS_DIR/throughput_english_wikipedia_${TIMESTAMP}.json"
    echo "  - Combined: $RESULTS_DIR/combined_english_wikipedia_${TIMESTAMP}.json"
}

# Run main function
main "$@"