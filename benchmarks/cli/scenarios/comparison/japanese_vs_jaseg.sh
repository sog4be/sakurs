#!/bin/bash
# Compare sakurs vs ja_sentence_segmenter on Japanese text (basic version)
# NOTE: For comprehensive comparison with Hyperfine and statistical analysis,
# use japanese_vs_jaseg_hyperfine.sh instead.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/ud_japanese_gsd/cli_format"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Ensure directories exist
mkdir -p "$RESULTS_DIR"

# Check if data is prepared
if [ ! -f "$DATA_DIR/gsd_plain.txt" ]; then
    echo "Error: Japanese benchmark data not found. Please run prepare_data.py first."
    exit 1
fi

# Check if both tools are available
if ! command -v sakurs &> /dev/null; then
    echo "Error: sakurs not found in PATH"
    exit 1
fi

if ! (cd "$ROOT_DIR/benchmarks" && uv run python -c "import ja_sentence_segmenter") 2>/dev/null; then
    echo "Error: ja_sentence_segmenter not installed"
    echo "Install with: pip install ja-sentence-segmenter"
    exit 1
fi

echo "Comparing sakurs vs ja_sentence_segmenter on Japanese text..."

# Get file size for throughput calculation
FILE_SIZE=$(wc -c < "$DATA_DIR/gsd_plain.txt" | tr -d ' ')
FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1048576" | bc)

echo "Test data: UD Japanese-GSD (${FILE_SIZE_MB} MB)"

# Run hyperfine comparison
echo "Running performance comparison..."
hyperfine \
    --warmup 2 \
    --runs 5 \
    --export-json "$RESULTS_DIR/japanese_comparison_${TIMESTAMP}.json" \
    "sakurs process --input '$DATA_DIR/gsd_plain.txt' --output /dev/null --format text --language japanese" \
    "cd '$ROOT_DIR/benchmarks' && uv run python 'baselines/ja_sentence_segmenter/benchmark.py' '$DATA_DIR/gsd_plain.txt' --output /dev/null"

# Extract and display results
echo ""
echo "Results saved to: $RESULTS_DIR/japanese_comparison_${TIMESTAMP}.json"

# Parse results with jq if available
if command -v jq &> /dev/null; then
    echo ""
    echo "Performance Summary:"
    echo "==================="
    
    SAKURS_MEAN=$(jq '.results[0].mean' "$RESULTS_DIR/japanese_comparison_${TIMESTAMP}.json")
    JASEG_MEAN=$(jq '.results[1].mean' "$RESULTS_DIR/japanese_comparison_${TIMESTAMP}.json")
    
    SAKURS_THROUGHPUT=$(echo "scale=2; $FILE_SIZE_MB / $SAKURS_MEAN" | bc)
    JASEG_THROUGHPUT=$(echo "scale=2; $FILE_SIZE_MB / $JASEG_MEAN" | bc)
    
    SPEEDUP=$(echo "scale=2; $JASEG_MEAN / $SAKURS_MEAN" | bc)
    
    echo "Sakurs:"
    echo "  Mean time: ${SAKURS_MEAN}s"
    echo "  Throughput: ${SAKURS_THROUGHPUT} MB/s"
    echo ""
    echo "ja_sentence_segmenter:"
    echo "  Mean time: ${JASEG_MEAN}s"
    echo "  Throughput: ${JASEG_THROUGHPUT} MB/s"
    echo ""
    echo "Speedup: ${SPEEDUP}x"
fi

# Run accuracy comparison
echo ""
echo "Running accuracy comparison..."

# Get predictions from both
sakurs process \
    --input "$DATA_DIR/gsd_plain.txt" \
    --output "$RESULTS_DIR/comp_sakurs_${TIMESTAMP}.txt" \
    --format text \
    --language japanese

cd "$ROOT_DIR/benchmarks" && uv run python "baselines/ja_sentence_segmenter/benchmark.py" \
    "$DATA_DIR/gsd_plain.txt" \
    --output "$RESULTS_DIR/comp_jaseg_${TIMESTAMP}.txt" \
    --format lines

# Evaluate accuracy for both
echo "Evaluating accuracy..."
cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/comp_sakurs_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/gsd_sentences.txt" \
    --output "$RESULTS_DIR/comp_sakurs_accuracy_${TIMESTAMP}.json" \
    --format json

cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/comp_jaseg_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/gsd_sentences.txt" \
    --output "$RESULTS_DIR/comp_jaseg_accuracy_${TIMESTAMP}.json" \
    --format json

# Display accuracy comparison
echo ""
echo "Accuracy Comparison:"
echo "==================="
echo ""
echo "Sakurs:"
cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/comp_sakurs_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/gsd_sentences.txt" \
    --format text

echo ""
echo "ja_sentence_segmenter:"
cd "$ROOT_DIR/benchmarks" && uv run python "cli/scripts/evaluate_accuracy.py" \
    --predicted "$RESULTS_DIR/comp_jaseg_${TIMESTAMP}.txt" \
    --reference "$DATA_DIR/gsd_sentences.txt" \
    --format text

# Clean up intermediate files
rm "$RESULTS_DIR/comp_sakurs_${TIMESTAMP}.txt"
rm "$RESULTS_DIR/comp_jaseg_${TIMESTAMP}.txt"