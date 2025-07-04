#!/bin/bash
# Enhanced accuracy benchmark for UD Japanese-BCCWJ with Hyperfine
# Special handling for Japanese text and character-based metrics

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/ud_japanese_bccwj/cli_format"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results/accuracy"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Create results directory
mkdir -p "$RESULTS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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

check_prerequisites() {
    local missing=0
    
    # Check if data is prepared
    if [ ! -f "$DATA_DIR/bccwj_plain.txt" ]; then
        print_error "UD Japanese-BCCWJ data not found. Please run prepare_data.py first."
        print_warning "Note: Original text may be reconstructed from tokens due to licensing."
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
    
    # Check if Python and required packages are available
    if ! command -v uv &> /dev/null; then
        print_error "Python 3 not found"
        missing=1
    fi
    
    return $missing
}

validate_accuracy() {
    local predicted_file=$1
    local min_f1=${2:-0.80}  # Slightly lower threshold for Japanese due to complexity
    
    print_status "Validating accuracy (Japanese text)..."
    
    # Run evaluation
    local accuracy_json="$RESULTS_DIR/validation_ja_${TIMESTAMP}.json"
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$predicted_file" \
        --reference "$DATA_DIR/bccwj_sentences.txt" \
        --output "$accuracy_json" \
        --format json
    
    # Extract F1 score
    local f1_score=$(cd "$ROOT_DIR/benchmarks" && uv run python -c "
import json
with open('$accuracy_json') as f:
    data = json.load(f)
    print(f\"{data['metrics']['f1']:.4f}\")
")
    
    print_status "F1 Score: $f1_score (minimum required: $min_f1)"
    
    # Check if accuracy meets threshold
    local meets_threshold=$(cd "$ROOT_DIR/benchmarks" && uv run python -c "
import json
with open('$accuracy_json') as f:
    data = json.load(f)
    print('true' if data['metrics']['f1'] >= $min_f1 else 'false')
")
    
    if [ "$meets_threshold" = "true" ]; then
        print_status "Accuracy validation PASSED"
        return 0
    else
        print_error "Accuracy validation FAILED - F1 score below threshold"
        return 1
    fi
}

analyze_japanese_text() {
    local file=$1
    
    print_status "Analyzing Japanese text characteristics..."
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import sys

def analyze_japanese_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        text = f.read()
    
    # Count character types
    hiragana = sum(1 for c in text if '\u3040' <= c <= '\u309f')
    katakana = sum(1 for c in text if '\u30a0' <= c <= '\u30ff')
    kanji = sum(1 for c in text if '\u4e00' <= c <= '\u9fff')
    total_chars = len(text)
    
    print(f"  Total characters: {total_chars:,}")
    print(f"  Hiragana: {hiragana:,} ({hiragana/total_chars*100:.1f}%)")
    print(f"  Katakana: {katakana:,} ({katakana/total_chars*100:.1f}%)")
    print(f"  Kanji: {kanji:,} ({kanji/total_chars*100:.1f}%)")
    
    # Count sentences
    with open(filepath, 'r', encoding='utf-8') as f:
        sentences = [line.strip() for line in f if line.strip()]
    print(f"  Sentences: {len(sentences):,}")
    
    if sentences:
        avg_len = sum(len(s) for s in sentences) / len(sentences)
        print(f"  Average sentence length: {avg_len:.1f} characters")

analyze_japanese_file("$file")
EOF
}

# Main execution
main() {
    print_status "Starting Enhanced Japanese BCCWJ Accuracy Benchmark"
    echo "Timestamp: $TIMESTAMP"
    echo "Results directory: $RESULTS_DIR"
    echo ""
    
    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi
    
    # Analyze input data
    if [ -f "$DATA_DIR/bccwj_plain.txt" ]; then
        print_status "Input data analysis:"
        analyze_japanese_text "$DATA_DIR/bccwj_sentences.txt"
        echo ""
    fi
    
    # Step 1: Initial accuracy validation
    print_status "Step 1: Initial accuracy validation"
    local temp_output="$RESULTS_DIR/temp_validation_ja_${TIMESTAMP}.txt"
    
    sakurs process \
        --input "$DATA_DIR/bccwj_plain.txt" \
        --output "$temp_output" \
        --format sentences \
        --language japanese
    
    if ! validate_accuracy "$temp_output" 0.80; then
        print_warning "Initial accuracy below threshold, but continuing..."
        print_warning "Japanese text reconstruction may affect accuracy."
    fi
    
    # Clean up validation file
    rm -f "$temp_output"
    
    # Step 2: Performance benchmark with Hyperfine
    print_status "Step 2: Running performance benchmark with Hyperfine"
    
    # Define output file for predictions
    local output_file="$RESULTS_DIR/bccwj_predicted_${TIMESTAMP}.txt"
    
    # Run Hyperfine benchmark
    # Note: Japanese processing may be slower due to character analysis
    hyperfine \
        --warmup 3 \
        --runs 10 \
        --export-json "$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.json" \
        --export-markdown "$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.md" \
        --show-output \
        --command-name "sakurs-japanese-bccwj" \
        "sakurs process --input '$DATA_DIR/bccwj_plain.txt' --output '$output_file' --format sentences --language japanese"
    
    # Step 3: Final accuracy evaluation
    print_status "Step 3: Final accuracy evaluation"
    
    # Run detailed evaluation
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$output_file" \
        --reference "$DATA_DIR/bccwj_sentences.txt" \
        --output "$RESULTS_DIR/accuracy_japanese_bccwj_${TIMESTAMP}.json" \
        --format json
    
    # Step 4: Character-based analysis
    print_status "Step 4: Character-based analysis"
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json

# Analyze predicted output
with open("$output_file", 'r', encoding='utf-8') as f:
    predicted_sentences = [line.strip() for line in f if line.strip()]

with open("$DATA_DIR/bccwj_sentences.txt", 'r', encoding='utf-8') as f:
    reference_sentences = [line.strip() for line in f if line.strip()]

# Character-level statistics
pred_chars = sum(len(s) for s in predicted_sentences)
ref_chars = sum(len(s) for s in reference_sentences)
char_diff = abs(pred_chars - ref_chars)

print(f"Predicted: {len(predicted_sentences)} sentences, {pred_chars:,} characters")
print(f"Reference: {len(reference_sentences)} sentences, {ref_chars:,} characters")
print(f"Character difference: {char_diff:,} ({char_diff/ref_chars*100:.2f}%)")
EOF
    
    # Step 5: Display combined results
    print_status "Step 5: Results summary"
    
    echo ""
    echo "=== Performance Results ==="
    if command -v jq &> /dev/null; then
        # Extract key metrics with jq if available
        local mean_time=$(jq '.results[0].mean' "$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.json")
        local stddev=$(jq '.results[0].stddev' "$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.json")
        local min_time=$(jq '.results[0].min' "$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.json")
        local max_time=$(jq '.results[0].max' "$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.json")
        
        echo "Mean time: ${mean_time}s ± ${stddev}s"
        echo "Range: ${min_time}s - ${max_time}s"
        
        # Calculate characters per second
        local total_chars=$(wc -m < "$DATA_DIR/bccwj_plain.txt" | tr -d ' ')
        local chars_per_sec=$(echo "scale=0; $total_chars / $mean_time" | bc)
        echo "Processing speed: ${chars_per_sec} characters/second"
    else
        # Fallback: show markdown summary
        cat "$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.md"
    fi
    
    echo ""
    echo "=== Accuracy Results ==="
    cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
        --predicted "$output_file" \
        --reference "$DATA_DIR/bccwj_sentences.txt" \
        --format text
    
    # Step 6: Generate combined report
    print_status "Step 6: Generating combined report"
    
    # Create a combined JSON report with Japanese-specific metrics
    cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json
from datetime import datetime

# Load performance results
with open("$RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

# Load accuracy results
with open("$RESULTS_DIR/accuracy_japanese_bccwj_${TIMESTAMP}.json") as f:
    acc_data = json.load(f)

# Calculate additional metrics
with open("$DATA_DIR/bccwj_plain.txt", 'r', encoding='utf-8') as f:
    text = f.read()
    total_chars = len(text)
    chars_per_sec = total_chars / perf_data["results"][0]["mean"]

# Combine results
combined = {
    "benchmark": "japanese_bccwj_accuracy",
    "timestamp": "$TIMESTAMP",
    "dataset": "UD Japanese-BCCWJ",
    "notes": "Text may be reconstructed from tokens due to licensing",
    "performance": {
        "mean_time": perf_data["results"][0]["mean"],
        "stddev": perf_data["results"][0]["stddev"],
        "min": perf_data["results"][0]["min"],
        "max": perf_data["results"][0]["max"],
        "runs": len(perf_data["results"][0]["times"]),
        "characters_per_second": int(chars_per_sec)
    },
    "accuracy": acc_data["metrics"],
    "text_statistics": {
        "total_characters": total_chars,
        "predicted_sentences": acc_data["metrics"]["predicted_sentences"],
        "reference_sentences": acc_data["metrics"]["reference_sentences"]
    },
    "system": {
        "command": perf_data["results"][0]["command"],
        "hyperfine_version": perf_data.get("hyperfine_version", "unknown")
    }
}

# Save combined report
with open("$RESULTS_DIR/combined_japanese_bccwj_${TIMESTAMP}.json", "w") as f:
    json.dump(combined, f, indent=2, ensure_ascii=False)

print(f"Combined report saved to: combined_japanese_bccwj_${TIMESTAMP}.json")
EOF
    
    # Cleanup temporary files
    rm -f "$output_file"
    
    print_status "Benchmark completed successfully!"
    echo ""
    echo "Results saved to:"
    echo "  - Performance: $RESULTS_DIR/perf_japanese_bccwj_${TIMESTAMP}.json"
    echo "  - Accuracy: $RESULTS_DIR/accuracy_japanese_bccwj_${TIMESTAMP}.json"
    echo "  - Combined: $RESULTS_DIR/combined_japanese_bccwj_${TIMESTAMP}.json"
    
    print_warning "Note: Japanese accuracy may be affected by text reconstruction from tokens."
}

# Run main function
main "$@"