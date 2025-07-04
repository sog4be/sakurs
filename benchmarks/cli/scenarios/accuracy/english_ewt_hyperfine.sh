#!/bin/bash
# Enhanced accuracy benchmark for UD English EWT with Hyperfine
# Measures both accuracy and performance with proper statistical analysis

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/ud_english_ewt/cli_format"
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
    if [ ! -f "$DATA_DIR/ewt_plain.txt" ]; then
        print_error "UD English EWT data not found. Please run prepare_data.py first."
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
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 not found"
        missing=1
    fi
    
    return $missing
}

validate_accuracy() {
    local predicted_file=$1
    local min_f1=${2:-0.85}  # Minimum acceptable F1 score
    
    print_status "Validating accuracy..."
    
    # Run evaluation
    local accuracy_json="$RESULTS_DIR/validation_${TIMESTAMP}.json"
    python3 "$SCRIPT_DIR/../../scripts/evaluate_accuracy.py" \
        --predicted "$predicted_file" \
        --reference "$DATA_DIR/ewt_sentences.txt" \
        --output "$accuracy_json" \
        --format json
    
    # Extract F1 score
    local f1_score=$(python3 -c "
import json
with open('$accuracy_json') as f:
    data = json.load(f)
    print(f\"{data['metrics']['f1']:.4f}\")
")
    
    print_status "F1 Score: $f1_score (minimum required: $min_f1)"
    
    # Check if accuracy meets threshold
    local meets_threshold=$(python3 -c "
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

# Main execution
main() {
    print_status "Starting Enhanced English EWT Accuracy Benchmark"
    echo "Timestamp: $TIMESTAMP"
    echo "Results directory: $RESULTS_DIR"
    echo ""
    
    # Check prerequisites
    if ! check_prerequisites; then
        exit 1
    fi
    
    # Step 1: Initial accuracy validation
    print_status "Step 1: Initial accuracy validation"
    local temp_output="$RESULTS_DIR/temp_validation_${TIMESTAMP}.txt"
    
    sakurs process \
        --input "$DATA_DIR/ewt_plain.txt" \
        --output "$temp_output" \
        --format sentences \
        --language english
    
    if ! validate_accuracy "$temp_output"; then
        print_error "Initial accuracy validation failed. Aborting benchmark."
        exit 1
    fi
    
    # Clean up validation file
    rm -f "$temp_output"
    
    # Step 2: Performance benchmark with Hyperfine
    print_status "Step 2: Running performance benchmark with Hyperfine"
    
    # Define output file for predictions (will be overwritten each run)
    local output_file="$RESULTS_DIR/ewt_predicted_${TIMESTAMP}.txt"
    
    # Run Hyperfine benchmark
    hyperfine \
        --warmup 3 \
        --runs 10 \
        --export-json "$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.json" \
        --export-markdown "$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.md" \
        --show-output \
        --command-name "sakurs-english-ewt" \
        "sakurs process --input '$DATA_DIR/ewt_plain.txt' --output '$output_file' --format sentences --language english"
    
    # Step 3: Final accuracy evaluation
    print_status "Step 3: Final accuracy evaluation"
    
    # Run detailed evaluation on the last output
    python3 "$SCRIPT_DIR/../../scripts/evaluate_accuracy.py" \
        --predicted "$output_file" \
        --reference "$DATA_DIR/ewt_sentences.txt" \
        --output "$RESULTS_DIR/accuracy_english_ewt_${TIMESTAMP}.json" \
        --format json
    
    # Step 4: Display combined results
    print_status "Step 4: Results summary"
    
    echo ""
    echo "=== Performance Results ==="
    if command -v jq &> /dev/null; then
        # Extract key metrics with jq if available
        local mean_time=$(jq '.results[0].mean' "$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.json")
        local stddev=$(jq '.results[0].stddev' "$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.json")
        local min_time=$(jq '.results[0].min' "$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.json")
        local max_time=$(jq '.results[0].max' "$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.json")
        
        echo "Mean time: ${mean_time}s ± ${stddev}s"
        echo "Range: ${min_time}s - ${max_time}s"
    else
        # Fallback: show markdown summary
        cat "$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.md"
    fi
    
    echo ""
    echo "=== Accuracy Results ==="
    python3 "$SCRIPT_DIR/../../scripts/evaluate_accuracy.py" \
        --predicted "$output_file" \
        --reference "$DATA_DIR/ewt_sentences.txt" \
        --format text
    
    # Step 5: Generate combined report
    print_status "Step 5: Generating combined report"
    
    # Create a combined JSON report
    python3 - <<EOF
import json
from datetime import datetime

# Load performance results
with open("$RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.json") as f:
    perf_data = json.load(f)

# Load accuracy results
with open("$RESULTS_DIR/accuracy_english_ewt_${TIMESTAMP}.json") as f:
    acc_data = json.load(f)

# Combine results
combined = {
    "benchmark": "english_ewt_accuracy",
    "timestamp": "$TIMESTAMP",
    "dataset": "UD English EWT",
    "performance": {
        "mean_time": perf_data["results"][0]["mean"],
        "stddev": perf_data["results"][0]["stddev"],
        "min": perf_data["results"][0]["min"],
        "max": perf_data["results"][0]["max"],
        "runs": len(perf_data["results"][0]["times"])
    },
    "accuracy": acc_data["metrics"],
    "system": {
        "command": perf_data["results"][0]["command"],
        "hyperfine_version": perf_data.get("hyperfine_version", "unknown")
    }
}

# Save combined report
with open("$RESULTS_DIR/combined_english_ewt_${TIMESTAMP}.json", "w") as f:
    json.dump(combined, f, indent=2)

print(f"Combined report saved to: combined_english_ewt_${TIMESTAMP}.json")
EOF
    
    # Cleanup temporary files
    rm -f "$output_file"
    
    print_status "Benchmark completed successfully!"
    echo ""
    echo "Results saved to:"
    echo "  - Performance: $RESULTS_DIR/perf_english_ewt_${TIMESTAMP}.json"
    echo "  - Accuracy: $RESULTS_DIR/accuracy_english_ewt_${TIMESTAMP}.json"
    echo "  - Combined: $RESULTS_DIR/combined_english_ewt_${TIMESTAMP}.json"
}

# Run main function
main "$@"