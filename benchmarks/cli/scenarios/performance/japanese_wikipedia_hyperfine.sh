#!/bin/bash
# Performance benchmark for Japanese Wikipedia using Hyperfine
# Measures throughput with character-based metrics for Japanese text

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
    local data_file="$DATA_DIR/wikipedia_ja_${SAMPLE_SIZE_MB}mb.txt"
    if [ ! -f "$data_file" ]; then
        print_error "Wikipedia Japanese data not found: $data_file"
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

analyze_japanese_text() {
    local input_file=$1
    
    print_status "Analyzing Japanese text characteristics..."
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import sys

def analyze_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        text = f.read()
    
    # Count character types
    hiragana = sum(1 for c in text if '\u3040' <= c <= '\u309f')
    katakana = sum(1 for c in text if '\u30a0' <= c <= '\u30ff')
    kanji = sum(1 for c in text if '\u4e00' <= c <= '\u9fff')
    romaji = sum(1 for c in text if c.isascii() and c.isalpha())
    total_chars = len(text)
    
    # File metrics
    size_bytes = len(text.encode('utf-8'))
    size_mb = size_bytes / 1024 / 1024
    
    print(f"  File size: {size_mb:.2f}MB ({size_bytes:,} bytes)")
    print(f"  Total characters: {total_chars:,}")
    print(f"  Hiragana: {hiragana:,} ({hiragana/total_chars*100:.1f}%)")
    print(f"  Katakana: {katakana:,} ({katakana/total_chars*100:.1f}%)")
    print(f"  Kanji: {kanji:,} ({kanji/total_chars*100:.1f}%)")
    print(f"  Romaji: {romaji:,} ({romaji/total_chars*100:.1f}%)")
    
    # Export metrics
    print(f"export PERF_SIZE_BYTES={size_bytes}")
    print(f"export PERF_SIZE_MB={size_mb}")
    print(f"export PERF_CHAR_COUNT={total_chars}")
    print(f"export PERF_HIRAGANA={hiragana}")
    print(f"export PERF_KATAKANA={katakana}")
    print(f"export PERF_KANJI={kanji}")

analyze_file("$input_file")
EOF
}

measure_baseline_metrics() {
    local input_file=$1
    
    print_status "Measuring baseline metrics..."
    
    # Run analysis and capture exports
    local metrics_output=$(analyze_japanese_text "$input_file")
    echo "$metrics_output" | grep -v "^export"
    
    # Source the exports
    eval "$(echo "$metrics_output" | grep "^export")"
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
    local mem_output="$RESULTS_DIR/memory_japanese_${TIMESTAMP}.txt"
    
    $TIME_CMD -v sakurs process \
        --input "$input_file" \
        --output "$output_file" \
        --format sentences \
        --language japanese \
        2>&1 | tee "$mem_output" | grep -E "(Maximum resident|User time|System time)" || true
    
    # Extract memory usage if available
    if grep -q "Maximum resident" "$mem_output" 2>/dev/null; then
        local max_memory=$(grep "Maximum resident" "$mem_output" | awk '{print $NF}')
        echo "  Maximum memory: $max_memory"
    fi
}

# Main execution
main() {
    print_header "Japanese Wikipedia Performance Benchmark"
    echo "Timestamp: $TIMESTAMP"
    echo "Sample size: ${SAMPLE_SIZE_MB}MB"
    echo "Benchmark runs: $BENCHMARK_RUNS (with $WARMUP_RUNS warmup)"
    echo "Results directory: $RESULTS_DIR"
    
    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi
    
    # Input/output files
    local input_file="$DATA_DIR/wikipedia_ja_${SAMPLE_SIZE_MB}mb.txt"
    local output_file="$RESULTS_DIR/temp_output_ja_${TIMESTAMP}.txt"
    
    # Step 1: Text analysis and baseline metrics
    print_header "Step 1: Japanese Text Analysis"
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
        --export-json "$RESULTS_DIR/perf_japanese_wikipedia_${TIMESTAMP}.json" \
        --export-markdown "$RESULTS_DIR/perf_japanese_wikipedia_${TIMESTAMP}.md" \
        --show-output \
        --command-name "sakurs-japanese-wikipedia" \
        "sakurs process --input '$input_file' --output '$output_file' --format sentences --language japanese"
    
    # Step 4: Calculate throughput metrics (character-based for Japanese)
    print_header "Step 4: Throughput Analysis"
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json
import os

# Load Hyperfine results
with open("$RESULTS_DIR/perf_japanese_wikipedia_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

# Get metrics from environment
size_bytes = int(os.environ['PERF_SIZE_BYTES'])
size_mb = float(os.environ['PERF_SIZE_MB'])
char_count = int(os.environ['PERF_CHAR_COUNT'])
hiragana_count = int(os.environ['PERF_HIRAGANA'])
katakana_count = int(os.environ['PERF_KATAKANA'])
kanji_count = int(os.environ['PERF_KANJI'])

# Calculate throughput
mean_time = perf_data["results"][0]["mean"]
throughput_mb_s = size_mb / mean_time
throughput_chars_s = char_count / mean_time

# Character-specific rates
hiragana_per_sec = hiragana_count / mean_time
katakana_per_sec = katakana_count / mean_time
kanji_per_sec = kanji_count / mean_time

# Count output sentences
try:
    with open("$output_file", 'r', encoding='utf-8') as f:
        sentence_count = sum(1 for line in f if line.strip())
    sentences_per_sec = sentence_count / mean_time
except:
    sentence_count = 0
    sentences_per_sec = 0

print(f"Performance Metrics:")
print(f"  Mean processing time: {mean_time:.3f}s")
print(f"  Throughput: {throughput_mb_s:.2f} MB/s")
print(f"  Character rate: {throughput_chars_s:,.0f} chars/s")
print(f"    - Hiragana: {hiragana_per_sec:,.0f} chars/s")
print(f"    - Katakana: {katakana_per_sec:,.0f} chars/s")
print(f"    - Kanji: {kanji_per_sec:,.0f} chars/s")
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
    "benchmark": "japanese_wikipedia_performance",
    "timestamp": "$TIMESTAMP",
    "input": {
        "file": "$input_file",
        "size_mb": size_mb,
        "size_bytes": size_bytes,
        "characters": char_count,
        "character_distribution": {
            "hiragana": hiragana_count,
            "katakana": katakana_count,
            "kanji": kanji_count
        }
    },
    "performance": {
        "mean_time": mean_time,
        "throughput_mb_s": throughput_mb_s,
        "throughput_chars_s": throughput_chars_s,
        "sentences_per_sec": sentences_per_sec,
        "total_sentences": sentence_count,
        "character_rates": {
            "hiragana_per_sec": hiragana_per_sec,
            "katakana_per_sec": katakana_per_sec,
            "kanji_per_sec": kanji_per_sec
        }
    },
    "latency_percentiles": {
        "p50": p50,
        "p90": p90,
        "p99": p99
    }
}

with open("$RESULTS_DIR/throughput_japanese_wikipedia_${TIMESTAMP}.json", "w") as f:
    json.dump(throughput_data, f, indent=2, ensure_ascii=False)
EOF
    
    # Step 5: Generate combined report
    print_header "Step 5: Combined Report Generation"
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json

# Load all data
with open("$RESULTS_DIR/perf_japanese_wikipedia_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

with open("$RESULTS_DIR/throughput_japanese_wikipedia_${TIMESTAMP}.json") as f:
    throughput_data = json.load(f)

# Create combined report
combined = {
    "benchmark": "japanese_wikipedia_performance",
    "timestamp": "$TIMESTAMP",
    "configuration": {
        "sample_size_mb": $SAMPLE_SIZE_MB,
        "benchmark_runs": $BENCHMARK_RUNS,
        "warmup_runs": $WARMUP_RUNS
    },
    "dataset": {
        "language": "Japanese",
        "source": "Wikipedia (HF 20231101.ja)",
        "size_mb": throughput_data["input"]["size_mb"],
        "characters": throughput_data["input"]["characters"],
        "character_distribution": throughput_data["input"]["character_distribution"]
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
with open("$RESULTS_DIR/combined_japanese_wikipedia_${TIMESTAMP}.json", "w") as f:
    json.dump(combined, f, indent=2, ensure_ascii=False)

print("Combined report saved successfully")
EOF
    
    # Cleanup
    rm -f "$output_file"
    
    print_header "Benchmark Completed Successfully!"
    echo "Results saved to:"
    echo "  - Performance: $RESULTS_DIR/perf_japanese_wikipedia_${TIMESTAMP}.json"
    echo "  - Throughput: $RESULTS_DIR/throughput_japanese_wikipedia_${TIMESTAMP}.json"
    echo "  - Combined: $RESULTS_DIR/combined_japanese_wikipedia_${TIMESTAMP}.json"
    
    print_warning "Note: Japanese processing includes multi-script analysis (hiragana, katakana, kanji)"
}

# Run main function
main "$@"