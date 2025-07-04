#!/bin/bash
# Simple performance benchmark using Brown Corpus (basic version)
# NOTE: For comprehensive Wikipedia performance benchmarking with Hyperfine,
# use the enhanced scripts: english_wikipedia_hyperfine.sh, japanese_wikipedia_hyperfine.sh

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/brown_corpus/subsets"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Ensure directories exist
mkdir -p "$RESULTS_DIR"

# Check if Brown Corpus is available as a temporary performance test
if [ ! -f "$DATA_DIR/medium/text.txt" ]; then
    echo "Error: Brown Corpus not found. Please download it first."
    echo "  cd $ROOT_DIR/benchmarks/data/brown_corpus && make download"
    exit 1
fi

# Check if sakurs is available
if ! command -v sakurs &> /dev/null; then
    echo "Error: sakurs not found in PATH"
    exit 1
fi

echo "Running simple performance benchmark on Brown Corpus (medium subset)..."

# Get file size for throughput calculation
FILE_SIZE=$(stat -f%z "$DATA_DIR/medium/text.txt" 2>/dev/null || stat -c%s "$DATA_DIR/medium/text.txt")
FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1048576" | bc)

echo "File size: ${FILE_SIZE_MB} MB"

# Run Hyperfine benchmark
echo "Running Hyperfine benchmark..."
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "$RESULTS_DIR/performance_brown_medium_${TIMESTAMP}.json" \
    "sakurs process --input '$DATA_DIR/medium/text.txt' --output /dev/null --format text --language english"

# Calculate and display throughput
echo ""
echo "Results saved to: $RESULTS_DIR/performance_brown_medium_${TIMESTAMP}.json"

# Extract mean time from JSON and calculate throughput
if command -v jq &> /dev/null; then
    MEAN_TIME=$(jq '.results[0].mean' "$RESULTS_DIR/performance_brown_medium_${TIMESTAMP}.json")
    THROUGHPUT=$(echo "scale=2; $FILE_SIZE_MB / $MEAN_TIME" | bc)
    echo "Mean time: ${MEAN_TIME}s"
    echo "Throughput: ${THROUGHPUT} MB/s"
fi