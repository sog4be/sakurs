#!/bin/bash
# Enhanced comparison: Sakurs vs ja_sentence_segmenter on Japanese text
# Uses Hyperfine for statistical rigor and comprehensive analysis

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results/comparison"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
COMPARISON_RUNS=${COMPARISON_RUNS:-10}
COMPARISON_WARMUP=${COMPARISON_WARMUP:-3}
MEMORY_PROFILING=${MEMORY_PROFILING:-true}

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
    
    print_status "Checking prerequisites..."
    
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
    
    # Check if Python and ja_sentence_segmenter are available
    if ! command -v uv &> /dev/null; then
        print_error "uv not found"
        echo "Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
        missing=1
    fi
    
    if ! (cd "$ROOT_DIR/benchmarks" && uv run python -c "import ja_sentence_segmenter") 2>/dev/null; then
        print_error "ja_sentence_segmenter not installed"
        echo "Install with: uv pip install ja-sentence-segmenter"
        missing=1
    fi
    
    return $missing
}

check_datasets() {
    local missing=0
    
    print_status "Checking datasets..."
    
    # Check UD Japanese-GSD (accuracy dataset)
    local ud_dir="$ROOT_DIR/benchmarks/data/ud_japanese_gsd/cli_format"
    if [ ! -f "$ud_dir/gsd_plain.txt" ] || [ ! -f "$ud_dir/gsd_sentences.txt" ]; then
        print_error "UD Japanese-GSD data not found in $ud_dir"
        echo "Please run: cd $ROOT_DIR/benchmarks && uv run python cli/scripts/prepare_data.py"
        missing=1
    else
        print_status "✓ UD Japanese-GSD data available"
    fi
    
    # Check Wikipedia Japanese (performance dataset)
    local wiki_dir="$ROOT_DIR/benchmarks/data/wikipedia/cli_format"
    if [ ! -f "$wiki_dir/wikipedia_ja_500mb.txt" ]; then
        print_warning "Wikipedia Japanese data not found in $wiki_dir"
        print_status "Will use UD GSD for all tests"
    else
        print_status "✓ Wikipedia Japanese data available"
    fi
    
    return $missing
}

analyze_japanese_text() {
    local input_file=$1
    local dataset_name=$2
    
    print_status "Analyzing Japanese text characteristics for $dataset_name..."
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
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
    
    print(f"  Dataset: $dataset_name")
    print(f"  File size: {size_mb:.2f}MB ({size_bytes:,} bytes)")
    print(f"  Total characters: {total_chars:,}")
    print(f"  Hiragana: {hiragana:,} ({hiragana/total_chars*100:.1f}%)")
    print(f"  Katakana: {katakana:,} ({katakana/total_chars*100:.1f}%)")
    print(f"  Kanji: {kanji:,} ({kanji/total_chars*100:.1f}%)")
    print(f"  Romaji: {romaji:,} ({romaji/total_chars*100:.1f}%)")
    
    # Export for later use
    print(f"export PERF_SIZE_BYTES_{dataset_name.upper()}={size_bytes}")
    print(f"export PERF_SIZE_MB_{dataset_name.upper()}={size_mb}")
    print(f"export PERF_CHAR_COUNT_{dataset_name.upper()}={total_chars}")
    print(f"export PERF_HIRAGANA_{dataset_name.upper()}={hiragana}")
    print(f"export PERF_KATAKANA_{dataset_name.upper()}={katakana}")
    print(f"export PERF_KANJI_{dataset_name.upper()}={kanji}")

analyze_file("$input_file")
EOF
}

run_accuracy_comparison() {
    local dataset_name=$1
    local input_file=$2
    local reference_file=$3
    
    print_header "Accuracy Comparison: $dataset_name"
    
    # Analyze text characteristics
    local metrics_output=$(analyze_japanese_text "$input_file" "$dataset_name")
    echo "$metrics_output" | grep -v "^export"
    # Source the exports
    eval "$(echo "$metrics_output" | grep "^export")"
    
    # Define output files
    local sakurs_output="$RESULTS_DIR/temp_sakurs_${dataset_name}_${TIMESTAMP}.txt"
    local jaseg_output="$RESULTS_DIR/temp_jaseg_${dataset_name}_${TIMESTAMP}.txt"
    
    # Run sakurs
    print_status "Running Sakurs segmentation..."
    sakurs process \
        --input "$input_file" \
        --output "$sakurs_output" \
        --format sentences \
        --language japanese
    
    # Run ja_sentence_segmenter
    print_status "Running ja_sentence_segmenter..."
    cd "$ROOT_DIR/benchmarks" && uv run python "baselines/ja_sentence_segmenter/benchmark.py" \
        "$input_file" \
        --output "$jaseg_output" \
        --format lines
    
    # Evaluate accuracy for both
    print_status "Evaluating accuracy..."
    
    # Sakurs accuracy
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$sakurs_output" \
        --reference "$reference_file" \
        --output "$RESULTS_DIR/accuracy_sakurs_${dataset_name}_${TIMESTAMP}.json" \
        --format json
    
    # ja_sentence_segmenter accuracy
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$jaseg_output" \
        --reference "$reference_file" \
        --output "$RESULTS_DIR/accuracy_jaseg_${dataset_name}_${TIMESTAMP}.json" \
        --format json
    
    # Display results
    echo ""
    echo "Accuracy Results for $dataset_name:"
    echo "=================================="
    echo ""
    echo "Sakurs:"
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$sakurs_output" \
        --reference "$reference_file" \
        --format text
    
    echo ""
    echo "ja_sentence_segmenter:"
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$jaseg_output" \
        --reference "$reference_file" \
        --format text
    
    # Cleanup
    rm -f "$sakurs_output" "$jaseg_output"
}

run_performance_comparison() {
    local dataset_name=$1
    local input_file=$2
    
    print_header "Performance Comparison: $dataset_name"
    
    # Analyze text and get metrics
    local metrics_output=$(analyze_japanese_text "$input_file" "$dataset_name")
    echo "$metrics_output" | grep -v "^export"
    eval "$(echo "$metrics_output" | grep "^export")"
    
    # Get metrics from environment variables
    local var_suffix="${dataset_name^^}"
    local size_bytes_var="PERF_SIZE_BYTES_${var_suffix}"
    local size_mb_var="PERF_SIZE_MB_${var_suffix}"
    local char_count_var="PERF_CHAR_COUNT_${var_suffix}"
    
    local size_bytes=${!size_bytes_var}
    local size_mb=${!size_mb_var}
    local char_count=${!char_count_var}
    
    print_status "Dataset: $dataset_name (${size_mb}MB, ${char_count} characters)"
    
    # Run Hyperfine comparison
    print_status "Running Hyperfine performance comparison..."
    
    local temp_sakurs="$RESULTS_DIR/temp_sakurs_perf_${TIMESTAMP}.txt"
    local temp_jaseg="$RESULTS_DIR/temp_jaseg_perf_${TIMESTAMP}.txt"
    
    hyperfine \
        --warmup "$COMPARISON_WARMUP" \
        --runs "$COMPARISON_RUNS" \
        --export-json "$RESULTS_DIR/perf_comparison_${dataset_name}_${TIMESTAMP}.json" \
        --export-markdown "$RESULTS_DIR/perf_comparison_${dataset_name}_${TIMESTAMP}.md" \
        --show-output \
        --command-name "sakurs" \
        --command-name "ja_sentence_segmenter" \
        "sakurs process --input '$input_file' --output '$temp_sakurs' --format sentences --language japanese" \
        "cd '$ROOT_DIR/benchmarks' && uv run python 'baselines/ja_sentence_segmenter/benchmark.py' '$input_file' --output '$temp_jaseg' --format lines"
    
    # Analyze performance results with character-based metrics
    print_status "Analyzing performance results..."
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json

# Load performance results
with open("$RESULTS_DIR/perf_comparison_${dataset_name}_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

# Extract metrics
sakurs_result = perf_data["results"][0]
jaseg_result = perf_data["results"][1]

size_mb = $size_mb
char_count = $char_count
hiragana_count = ${!PERF_HIRAGANA_${var_suffix}}
katakana_count = ${!PERF_KATAKANA_${var_suffix}}
kanji_count = ${!PERF_KANJI_${var_suffix}}

# Calculate metrics
sakurs_throughput_mb = size_mb / sakurs_result["mean"]
jaseg_throughput_mb = size_mb / jaseg_result["mean"]
sakurs_throughput_chars = char_count / sakurs_result["mean"]
jaseg_throughput_chars = char_count / jaseg_result["mean"]

# Character-specific rates for Sakurs
sakurs_hiragana_rate = hiragana_count / sakurs_result["mean"]
sakurs_katakana_rate = katakana_count / sakurs_result["mean"]
sakurs_kanji_rate = kanji_count / sakurs_result["mean"]

# Character-specific rates for ja_sentence_segmenter
jaseg_hiragana_rate = hiragana_count / jaseg_result["mean"]
jaseg_katakana_rate = katakana_count / jaseg_result["mean"]
jaseg_kanji_rate = kanji_count / jaseg_result["mean"]

speedup = jaseg_result["mean"] / sakurs_result["mean"]

print(f"Performance Results for $dataset_name:")
print(f"====================================")
print(f"")
print(f"Sakurs:")
print(f"  Mean time: {sakurs_result['mean']:.3f}s ± {sakurs_result['stddev']:.3f}s")
print(f"  Throughput: {sakurs_throughput_mb:.2f} MB/s ({sakurs_throughput_chars:,.0f} chars/s)")
print(f"  Character rates:")
print(f"    - Hiragana: {sakurs_hiragana_rate:,.0f} chars/s")
print(f"    - Katakana: {sakurs_katakana_rate:,.0f} chars/s")
print(f"    - Kanji: {sakurs_kanji_rate:,.0f} chars/s")
print(f"")
print(f"ja_sentence_segmenter:")
print(f"  Mean time: {jaseg_result['mean']:.3f}s ± {jaseg_result['stddev']:.3f}s")
print(f"  Throughput: {jaseg_throughput_mb:.2f} MB/s ({jaseg_throughput_chars:,.0f} chars/s)")
print(f"  Character rates:")
print(f"    - Hiragana: {jaseg_hiragana_rate:,.0f} chars/s")
print(f"    - Katakana: {jaseg_katakana_rate:,.0f} chars/s")
print(f"    - Kanji: {jaseg_kanji_rate:,.0f} chars/s")
print(f"")
print(f"Speedup: {speedup:.2f}x {'(Sakurs faster)' if speedup > 1 else '(ja_sentence_segmenter faster)'}")

# Save analysis
analysis = {
    "benchmark": f"japanese_comparison_{dataset_name.lower()}",
    "timestamp": "$TIMESTAMP",
    "dataset": {
        "name": "$dataset_name",
        "size_mb": size_mb,
        "characters": char_count,
        "character_distribution": {
            "hiragana": hiragana_count,
            "katakana": katakana_count,
            "kanji": kanji_count
        }
    },
    "sakurs": {
        "mean_time": sakurs_result["mean"],
        "stddev": sakurs_result["stddev"],
        "throughput_mb_s": sakurs_throughput_mb,
        "throughput_chars_s": sakurs_throughput_chars,
        "character_rates": {
            "hiragana_per_sec": sakurs_hiragana_rate,
            "katakana_per_sec": sakurs_katakana_rate,
            "kanji_per_sec": sakurs_kanji_rate
        }
    },
    "ja_sentence_segmenter": {
        "mean_time": jaseg_result["mean"],
        "stddev": jaseg_result["stddev"],
        "throughput_mb_s": jaseg_throughput_mb,
        "throughput_chars_s": jaseg_throughput_chars,
        "character_rates": {
            "hiragana_per_sec": jaseg_hiragana_rate,
            "katakana_per_sec": jaseg_katakana_rate,
            "kanji_per_sec": jaseg_kanji_rate
        }
    },
    "comparison": {
        "speedup": speedup,
        "sakurs_faster": speedup > 1
    }
}

with open("$RESULTS_DIR/analysis_${dataset_name}_${TIMESTAMP}.json", "w") as f:
    json.dump(analysis, f, indent=2, ensure_ascii=False)
EOF
    
    # Cleanup
    rm -f "$temp_sakurs" "$temp_jaseg"
}

run_memory_comparison() {
    local input_file=$1
    local dataset_name=$2
    
    if [ "$MEMORY_PROFILING" != "true" ]; then
        print_warning "Memory profiling disabled"
        return 0
    fi
    
    print_header "Memory Usage Comparison: $dataset_name"
    
    # Use time command for memory profiling
    local time_cmd="time"
    if command -v gtime &> /dev/null; then
        time_cmd="gtime"
    fi
    
    # Test Sakurs memory usage
    print_status "Profiling Sakurs memory usage..."
    local sakurs_mem="$RESULTS_DIR/memory_sakurs_${dataset_name}_${TIMESTAMP}.txt"
    local temp_output="$RESULTS_DIR/temp_memory_${TIMESTAMP}.txt"
    
    $time_cmd -v sakurs process \
        --input "$input_file" \
        --output "$temp_output" \
        --format sentences \
        --language japanese \
        2>&1 | tee "$sakurs_mem" | grep -E "(Maximum resident|User time|System time)" || true
    
    # Test ja_sentence_segmenter memory usage
    print_status "Profiling ja_sentence_segmenter memory usage..."
    local jaseg_mem="$RESULTS_DIR/memory_jaseg_${dataset_name}_${TIMESTAMP}.txt"
    
    $time_cmd -v sh -c "cd '$ROOT_DIR/benchmarks' && uv run python 'baselines/ja_sentence_segmenter/benchmark.py' \
        '$input_file' \
        --output '$temp_output' \
        --format lines" \
        2>&1 | tee "$jaseg_mem" | grep -E "(Maximum resident|User time|System time)" || true
    
    # Compare memory usage
    echo ""
    echo "Memory Usage Comparison:"
    echo "======================="
    if grep -q "Maximum resident" "$sakurs_mem" 2>/dev/null; then
        local sakurs_memory=$(grep "Maximum resident" "$sakurs_mem" | awk '{print $NF}')
        echo "Sakurs peak memory: $sakurs_memory"
    fi
    
    if grep -q "Maximum resident" "$jaseg_mem" 2>/dev/null; then
        local jaseg_memory=$(grep "Maximum resident" "$jaseg_mem" | awk '{print $NF}')
        echo "ja_sentence_segmenter peak memory: $jaseg_memory"
    fi
    
    # Cleanup
    rm -f "$temp_output"
}

# Main execution
main() {
    print_header "Enhanced Japanese vs ja_sentence_segmenter Comparison"
    echo "Timestamp: $TIMESTAMP"
    echo "Comparison runs: $COMPARISON_RUNS (with $COMPARISON_WARMUP warmup)"
    echo "Memory profiling: $MEMORY_PROFILING"
    echo "Results directory: $RESULTS_DIR"
    
    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi
    
    if ! check_datasets; then
        exit 1
    fi
    
    # Dataset paths
    local ud_dir="$ROOT_DIR/benchmarks/data/ud_japanese_gsd/cli_format"
    local wiki_dir="$ROOT_DIR/benchmarks/data/wikipedia/cli_format"
    
    # Run comparisons on UD Japanese-GSD
    run_accuracy_comparison "UD_GSD" "$ud_dir/gsd_plain.txt" "$ud_dir/gsd_sentences.txt"
    run_performance_comparison "UD_GSD" "$ud_dir/gsd_plain.txt"
    run_memory_comparison "$ud_dir/gsd_plain.txt" "UD_GSD"
    
    # Run performance comparison on Wikipedia if available
    if [ -f "$wiki_dir/wikipedia_ja_500mb.txt" ]; then
        run_performance_comparison "Wikipedia" "$wiki_dir/wikipedia_ja_500mb.txt"
        run_memory_comparison "$wiki_dir/wikipedia_ja_500mb.txt" "Wikipedia"
    else
        print_warning "Wikipedia dataset not available, skipping Wikipedia performance test"
    fi
    
    print_header "Comparison Completed Successfully!"
    echo "Results saved to: $RESULTS_DIR"
    echo ""
    echo "Generated files:"
    echo "  - Performance: perf_comparison_*_${TIMESTAMP}.json"
    echo "  - Accuracy: accuracy_*_${TIMESTAMP}.json"
    echo "  - Analysis: analysis_*_${TIMESTAMP}.json"
    if [ "$MEMORY_PROFILING" = "true" ]; then
        echo "  - Memory: memory_*_${TIMESTAMP}.txt"
    fi
    
    print_warning "Note: Japanese text may be reconstructed from tokens, affecting accuracy comparison"
}

# Run main function
main "$@"