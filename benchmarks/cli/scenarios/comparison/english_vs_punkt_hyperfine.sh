#!/bin/bash
# Enhanced comparison: Sakurs vs NLTK Punkt on English text
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
    
    # Check if Python and NLTK are available
    if ! command -v uv &> /dev/null; then
        print_error "uv not found"
        echo "Install with: curl -LsSf https://astral.sh/uv/install.sh | sh"
        missing=1
    fi
    
    if ! (cd "$ROOT_DIR/benchmarks" && uv run python -c "import nltk") 2>/dev/null; then
        print_error "NLTK not installed"
        echo "Install with: pip install nltk"
        missing=1
    fi
    
    # Check if punkt data is available
    if ! (cd "$ROOT_DIR/benchmarks" && uv run python -c "import nltk; nltk.data.find('tokenizers/punkt')") 2>/dev/null; then
        print_warning "NLTK punkt data not found. Attempting download..."
        cd "$ROOT_DIR/benchmarks" && uv run python -c "import nltk; nltk.download('punkt', quiet=True)"
        if ! (cd "$ROOT_DIR/benchmarks" && uv run python -c "import nltk; nltk.data.find('tokenizers/punkt')") 2>/dev/null; then
            print_error "Failed to download NLTK punkt data"
            missing=1
        else
            print_status "NLTK punkt data downloaded successfully"
        fi
    fi
    
    return $missing
}

check_datasets() {
    local missing=0
    
    print_status "Checking datasets..."
    
    # Check UD English EWT (accuracy dataset)
    local ud_dir="$ROOT_DIR/benchmarks/data/ud_english_ewt/cli_format"
    if [ ! -f "$ud_dir/ewt_plain.txt" ] || [ ! -f "$ud_dir/ewt_sentences.txt" ]; then
        print_error "UD English EWT data not found in $ud_dir"
        echo "Please run: cd $ROOT_DIR/benchmarks && uv run python cli/scripts/prepare_data.py"
        missing=1
    else
        print_status "✓ UD English EWT data available"
    fi
    
    # Check Wikipedia English (performance dataset)
    local wiki_dir="$ROOT_DIR/benchmarks/data/wikipedia/cli_format"
    if [ ! -f "$wiki_dir/wikipedia_en_500mb.txt" ]; then
        print_warning "Wikipedia English data not found in $wiki_dir"
        print_status "Will use UD EWT for all tests"
    else
        print_status "✓ Wikipedia English data available"
    fi
    
    return $missing
}

run_accuracy_comparison() {
    local dataset_name=$1
    local input_file=$2
    local reference_file=$3
    
    print_header "Accuracy Comparison: $dataset_name"
    
    # Define output files
    local sakurs_output="$RESULTS_DIR/temp_sakurs_${dataset_name}_${TIMESTAMP}.txt"
    local punkt_output="$RESULTS_DIR/temp_punkt_${dataset_name}_${TIMESTAMP}.txt"
    
    # Run sakurs
    print_status "Running Sakurs segmentation..."
    sakurs process \
        --input "$input_file" \
        --output "$sakurs_output" \
        --format sentences \
        --language english
    
    # Run NLTK Punkt
    print_status "Running NLTK Punkt segmentation..."
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import sys
sys.path.insert(0, "$ROOT_DIR/benchmarks/baselines")
from nltk_punkt.segmenter import create_segmenter

# Load text
with open("$input_file", 'r', encoding='utf-8') as f:
    text = f.read()

# Segment with Punkt
segmenter = create_segmenter('english')
sentences = segmenter.extract_sentences(text)

# Save sentences
with open("$punkt_output", 'w', encoding='utf-8') as f:
    for sentence in sentences:
        f.write(sentence.strip() + '\n')
EOF
    
    # Evaluate accuracy for both
    print_status "Evaluating accuracy..."
    
    # Sakurs accuracy
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$sakurs_output" \
        --reference "$reference_file" \
        --output "$RESULTS_DIR/accuracy_sakurs_${dataset_name}_${TIMESTAMP}.json" \
        --format json
    
    # Punkt accuracy
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$punkt_output" \
        --reference "$reference_file" \
        --output "$RESULTS_DIR/accuracy_punkt_${dataset_name}_${TIMESTAMP}.json" \
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
    echo "NLTK Punkt:"
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$punkt_output" \
        --reference "$reference_file" \
        --format text
    
    # Cleanup
    rm -f "$sakurs_output" "$punkt_output"
}

run_performance_comparison() {
    local dataset_name=$1
    local input_file=$2
    
    print_header "Performance Comparison: $dataset_name"
    
    # Get file metrics
    local size_bytes=$(wc -c < "$input_file" | tr -d ' ')
    local size_mb=$(echo "scale=2; $size_bytes / 1024 / 1024" | bc)
    local char_count=$(wc -m < "$input_file" | tr -d ' ')
    
    print_status "Dataset: $dataset_name (${size_mb}MB, ${char_count} characters)"
    
    # Run Hyperfine comparison
    print_status "Running Hyperfine performance comparison..."
    
    local temp_sakurs="$RESULTS_DIR/temp_sakurs_perf_${TIMESTAMP}.txt"
    local temp_punkt="$RESULTS_DIR/temp_punkt_perf_${TIMESTAMP}.txt"
    
    hyperfine \
        --warmup "$COMPARISON_WARMUP" \
        --runs "$COMPARISON_RUNS" \
        --export-json "$RESULTS_DIR/perf_comparison_${dataset_name}_${TIMESTAMP}.json" \
        --export-markdown "$RESULTS_DIR/perf_comparison_${dataset_name}_${TIMESTAMP}.md" \
        --show-output \
        --command-name "sakurs" \
        --command-name "nltk-punkt" \
        "sakurs process --input '$input_file' --output '$temp_sakurs' --format sentences --language english" \
        "cd '$ROOT_DIR/benchmarks' && uv run python -c \"
import sys
sys.path.insert(0, '$ROOT_DIR/benchmarks/baselines')
from nltk_punkt.segmenter import create_segmenter

with open('$input_file', 'r', encoding='utf-8') as f:
    text = f.read()

segmenter = create_segmenter('english')
sentences = segmenter.extract_sentences(text)

with open('$temp_punkt', 'w', encoding='utf-8') as f:
    for sentence in sentences:
        f.write(sentence.strip() + '\n')
\""
    
    # Analyze performance results
    print_status "Analyzing performance results..."
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json

# Load performance results
with open("$RESULTS_DIR/perf_comparison_${dataset_name}_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

# Extract metrics
sakurs_result = perf_data["results"][0]
punkt_result = perf_data["results"][1]

size_mb = $size_mb
char_count = $char_count

# Calculate metrics
sakurs_throughput_mb = size_mb / sakurs_result["mean"]
punkt_throughput_mb = size_mb / punkt_result["mean"]
sakurs_throughput_chars = char_count / sakurs_result["mean"]
punkt_throughput_chars = char_count / punkt_result["mean"]

speedup = punkt_result["mean"] / sakurs_result["mean"]

print(f"Performance Results for $dataset_name:")
print(f"====================================")
print(f"")
print(f"Sakurs:")
print(f"  Mean time: {sakurs_result['mean']:.3f}s ± {sakurs_result['stddev']:.3f}s")
print(f"  Throughput: {sakurs_throughput_mb:.2f} MB/s ({sakurs_throughput_chars:,.0f} chars/s)")
print(f"")
print(f"NLTK Punkt:")
print(f"  Mean time: {punkt_result['mean']:.3f}s ± {punkt_result['stddev']:.3f}s")
print(f"  Throughput: {punkt_throughput_mb:.2f} MB/s ({punkt_throughput_chars:,.0f} chars/s)")
print(f"")
print(f"Speedup: {speedup:.2f}x {'(Sakurs faster)' if speedup > 1 else '(Punkt faster)'}")

# Save analysis
analysis = {
    "benchmark": f"english_comparison_{dataset_name.lower()}",
    "timestamp": "$TIMESTAMP",
    "dataset": {
        "name": "$dataset_name",
        "size_mb": size_mb,
        "characters": char_count
    },
    "sakurs": {
        "mean_time": sakurs_result["mean"],
        "stddev": sakurs_result["stddev"],
        "throughput_mb_s": sakurs_throughput_mb,
        "throughput_chars_s": sakurs_throughput_chars
    },
    "punkt": {
        "mean_time": punkt_result["mean"],
        "stddev": punkt_result["stddev"],
        "throughput_mb_s": punkt_throughput_mb,
        "throughput_chars_s": punkt_throughput_chars
    },
    "comparison": {
        "speedup": speedup,
        "sakurs_faster": speedup > 1
    }
}

with open("$RESULTS_DIR/analysis_${dataset_name}_${TIMESTAMP}.json", "w") as f:
    json.dump(analysis, f, indent=2)
EOF
    
    # Cleanup
    rm -f "$temp_sakurs" "$temp_punkt"
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
        --language english \
        2>&1 | tee "$sakurs_mem" | grep -E "(Maximum resident|User time|System time)" || true
    
    # Test Punkt memory usage
    print_status "Profiling NLTK Punkt memory usage..."
    local punkt_mem="$RESULTS_DIR/memory_punkt_${dataset_name}_${TIMESTAMP}.txt"
    
    $time_cmd -v sh -c "cd '$ROOT_DIR/benchmarks' && uv run python -c \"
import sys
sys.path.insert(0, '$ROOT_DIR/benchmarks/baselines')
from nltk_punkt.segmenter import create_segmenter

with open('$input_file', 'r', encoding='utf-8') as f:
    text = f.read()

segmenter = create_segmenter('english')
sentences = segmenter.extract_sentences(text)

with open('$temp_output', 'w', encoding='utf-8') as f:
    for sentence in sentences:
        f.write(sentence.strip() + '\n')
\"" 2>&1 | tee "$punkt_mem" | grep -E "(Maximum resident|User time|System time)" || true
    
    # Compare memory usage
    echo ""
    echo "Memory Usage Comparison:"
    echo "======================="
    if grep -q "Maximum resident" "$sakurs_mem" 2>/dev/null; then
        local sakurs_memory=$(grep "Maximum resident" "$sakurs_mem" | awk '{print $NF}')
        echo "Sakurs peak memory: $sakurs_memory"
    fi
    
    if grep -q "Maximum resident" "$punkt_mem" 2>/dev/null; then
        local punkt_memory=$(grep "Maximum resident" "$punkt_mem" | awk '{print $NF}')
        echo "NLTK Punkt peak memory: $punkt_memory"
    fi
    
    # Cleanup
    rm -f "$temp_output"
}

# Main execution
main() {
    print_header "Enhanced English vs NLTK Punkt Comparison"
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
    local ud_dir="$ROOT_DIR/benchmarks/data/ud_english_ewt/cli_format"
    local wiki_dir="$ROOT_DIR/benchmarks/data/wikipedia/cli_format"
    
    # Run comparisons on UD English EWT
    run_accuracy_comparison "UD_EWT" "$ud_dir/ewt_plain.txt" "$ud_dir/ewt_sentences.txt"
    run_performance_comparison "UD_EWT" "$ud_dir/ewt_plain.txt"
    run_memory_comparison "$ud_dir/ewt_plain.txt" "UD_EWT"
    
    # Run performance comparison on Wikipedia if available
    if [ -f "$wiki_dir/wikipedia_en_500mb.txt" ]; then
        run_performance_comparison "Wikipedia" "$wiki_dir/wikipedia_en_500mb.txt"
        run_memory_comparison "$wiki_dir/wikipedia_en_500mb.txt" "Wikipedia"
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
}

# Run main function
main "$@"