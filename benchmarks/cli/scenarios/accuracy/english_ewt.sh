#!/bin/bash
# Benchmark accuracy on UD English EWT dataset
# NOTE: This is the basic version. For comprehensive benchmarking with Hyperfine,
# use english_ewt_hyperfine.sh instead.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/ud_english_ewt/cli_format"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Ensure directories exist
mkdir -p "$RESULTS_DIR"

# Check if data is prepared
if [ ! -f "$DATA_DIR/ewt_plain.txt" ]; then
    echo "Error: UD English EWT data not found. Please run prepare_data.py first."
    exit 1
fi

# Check if sakurs is available
if ! command -v sakurs &> /dev/null; then
    echo "Error: sakurs not found in PATH"
    echo "Please build and add to PATH:"
    echo "  cd $ROOT_DIR && cargo build --release --bin sakurs"
    echo "  export PATH=\$PATH:$ROOT_DIR/target/release"
    exit 1
fi

echo "Running accuracy benchmark on UD English EWT..."

# Run segmentation
echo "Segmenting with sakurs..."
sakurs process \
    --input "$DATA_DIR/ewt_plain.txt" \
    --output "$RESULTS_DIR/ewt_predicted_${TIMESTAMP}.txt" \
    --format text \
    --language english

# Evaluate accuracy
echo "Evaluating accuracy..."
cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/ewt_predicted_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/ewt_sentences.txt" \
    --output "$RESULTS_DIR/english_ewt_accuracy_${TIMESTAMP}.json" \
    --format json

# Display results
echo ""
echo "Results saved to: $RESULTS_DIR/english_ewt_accuracy_${TIMESTAMP}.json"
echo ""
echo "Summary:"
cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/ewt_predicted_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/ewt_sentences.txt" \
    --format text

# Clean up intermediate files (optional)
# rm "$RESULTS_DIR/ewt_predicted_${TIMESTAMP}.txt"