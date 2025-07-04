#!/bin/bash
# Benchmark accuracy on UD Japanese-BCCWJ dataset

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/ud_japanese_bccwj/cli_format"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Ensure directories exist
mkdir -p "$RESULTS_DIR"
mkdir -p "$DATA_DIR"

# Check if data is prepared
if [ ! -f "$DATA_DIR/bccwj_plain.txt" ]; then
    echo "Error: UD Japanese-BCCWJ data not found. Please run prepare_data.py first."
    echo "Note: Original text may not be available due to licensing."
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

echo "Running accuracy benchmark on UD Japanese-BCCWJ..."
echo "Note: Results may be affected by token reconstruction if original text is not available."

# Run segmentation with sakurs
echo "Segmenting with sakurs..."
sakurs process \
    --input "$DATA_DIR/bccwj_plain.txt" \
    --output "$RESULTS_DIR/bccwj_sakurs_${TIMESTAMP}.txt" \
    --format text \
    --language japanese

# Run segmentation with ja_sentence_segmenter (if available)
if python -c "import ja_sentence_segmenter" 2>/dev/null; then
    echo "Segmenting with ja_sentence_segmenter..."
    python "$ROOT_DIR/benchmarks/baselines/ja_sentence_segmenter/benchmark.py" \
        "$DATA_DIR/bccwj_plain.txt" \
        --output "$RESULTS_DIR/bccwj_jaseg_${TIMESTAMP}.txt" \
        --format lines
        
    # Evaluate ja_sentence_segmenter accuracy
    echo "Evaluating ja_sentence_segmenter accuracy..."
    python "$SCRIPT_DIR/../../scripts/evaluate_accuracy.py" \
        --predicted "$RESULTS_DIR/bccwj_jaseg_${TIMESTAMP}.txt" \
        --reference "$DATA_DIR/bccwj_sentences.txt" \
        --output "$RESULTS_DIR/japanese_bccwj_jaseg_accuracy_${TIMESTAMP}.json" \
        --format json
else
    echo "Warning: ja_sentence_segmenter not installed. Skipping baseline comparison."
    echo "Install with: pip install ja-sentence-segmenter"
fi

# Evaluate sakurs accuracy
echo "Evaluating sakurs accuracy..."
python "$SCRIPT_DIR/../../scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/bccwj_sakurs_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/bccwj_sentences.txt" \
    --output "$RESULTS_DIR/japanese_bccwj_accuracy_${TIMESTAMP}.json" \
    --format json

# Display results
echo ""
echo "Results saved to:"
echo "  - Sakurs: $RESULTS_DIR/japanese_bccwj_accuracy_${TIMESTAMP}.json"
if [ -f "$RESULTS_DIR/japanese_bccwj_jaseg_accuracy_${TIMESTAMP}.json" ]; then
    echo "  - ja_sentence_segmenter: $RESULTS_DIR/japanese_bccwj_jaseg_accuracy_${TIMESTAMP}.json"
fi

echo ""
echo "Sakurs Summary:"
python "$SCRIPT_DIR/../../scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/bccwj_sakurs_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/bccwj_sentences.txt" \
    --format text

# Clean up intermediate files (optional)
# rm "$RESULTS_DIR/bccwj_sakurs_${TIMESTAMP}.txt"
# rm "$RESULTS_DIR/bccwj_jaseg_${TIMESTAMP}.txt"