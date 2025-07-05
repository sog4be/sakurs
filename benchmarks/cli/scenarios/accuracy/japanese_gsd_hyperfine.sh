#!/bin/bash
# Hyperfine accuracy benchmark for UD Japanese-GSD
# Measures both performance and accuracy of sentence segmentation

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/ud_japanese_gsd/cli_format"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results/accuracy"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Create results directory
mkdir -p "$RESULTS_DIR"

# Files
TEST_FILE="$DATA_DIR/gsd_plain.txt"
GOLD_FILE="$DATA_DIR/gsd_sentences.txt"
OUTPUT_FILE="/tmp/sakurs_gsd_output_${TIMESTAMP}.txt"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check prerequisites
if [ ! -f "$TEST_FILE" ]; then
    echo -e "${RED}Error: Test file not found: $TEST_FILE${NC}"
    echo "Please run: cd $ROOT_DIR/benchmarks && uv run python cli/scripts/prepare_data.py"
    exit 1
fi

if [ ! -f "$GOLD_FILE" ]; then
    echo -e "${RED}Error: Gold file not found: $GOLD_FILE${NC}"
    echo "Please run: cd $ROOT_DIR/benchmarks && uv run python cli/scripts/prepare_data.py"
    exit 1
fi

if ! command -v hyperfine &> /dev/null; then
    echo -e "${RED}Error: hyperfine not found${NC}"
    echo "Please install: brew install hyperfine (macOS) or cargo install hyperfine"
    exit 1
fi

echo -e "${BLUE}UD Japanese-GSD Accuracy Benchmark${NC}"
echo "=================================="
echo "Test file: $TEST_FILE"
echo "Gold file: $GOLD_FILE"
echo ""

# Get file statistics
FILE_SIZE=$(wc -c < "$TEST_FILE")
FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1048576" | bc)
GOLD_SENTENCES=$(wc -l < "$GOLD_FILE")

echo "Dataset Statistics:"
echo "  Size: ${FILE_SIZE_MB} MB (${FILE_SIZE} bytes)"
echo "  Reference sentences: ${GOLD_SENTENCES}"
echo ""

# Run performance benchmark with hyperfine
echo -e "${GREEN}Running performance benchmark...${NC}"
PERF_JSON="$RESULTS_DIR/performance_japanese_gsd_${TIMESTAMP}.json"

hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "$PERF_JSON" \
    --command-name "sakurs-japanese-gsd" \
    "sakurs process --input '$TEST_FILE' --output '$OUTPUT_FILE' --format plain"

# Extract performance metrics
echo ""
echo -e "${GREEN}Extracting performance metrics...${NC}"
cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json
import sys

with open("$PERF_JSON") as f:
    data = json.load(f)

result = data["results"][0]
mean_time = result["mean"]
stddev = result["stddev"]
min_time = result["min"]
max_time = result["max"]

print(f"Performance Results:")
print(f"  Mean time: {mean_time:.3f} seconds")
print(f"  Std dev: {stddev:.3f} seconds")
print(f"  Min time: {min_time:.3f} seconds")
print(f"  Max time: {max_time:.3f} seconds")

# Calculate throughput
file_size_mb = $FILE_SIZE / (1024 * 1024)
chars_per_sec = $FILE_SIZE / mean_time
mb_per_sec = file_size_mb / mean_time

print(f"\nThroughput:")
print(f"  {chars_per_sec:,.0f} characters/second")
print(f"  {mb_per_sec:.2f} MB/second")
EOF

# Run accuracy evaluation
echo ""
echo -e "${GREEN}Running accuracy evaluation...${NC}"
# Generate output for accuracy check
sakurs process --input "$TEST_FILE" --output "$OUTPUT_FILE" --format plain

# Calculate accuracy metrics
ACCURACY_JSON="$RESULTS_DIR/accuracy_japanese_gsd_${TIMESTAMP}.json"
cd "$ROOT_DIR/benchmarks" && uv run python cli/scripts/evaluate_accuracy.py \
    --predicted "$OUTPUT_FILE" \
    --reference "$GOLD_FILE" \
    --language japanese \
    --output "$ACCURACY_JSON"

# Generate combined report
echo ""
echo -e "${GREEN}Generating combined report...${NC}"
COMBINED_JSON="$RESULTS_DIR/combined_japanese_gsd_${TIMESTAMP}.json"

cd "$ROOT_DIR/benchmarks" && uv run python - <<EOF
import json
from pathlib import Path

# Load performance data
with open("$PERF_JSON") as f:
    perf_data = json.load(f)

# Load accuracy data
with open("$ACCURACY_JSON") as f:
    acc_data = json.load(f)

# Combine results
combined = {
    "benchmark": "UD Japanese-GSD Accuracy",
    "timestamp": "$TIMESTAMP",
    "dataset": "UD_Japanese-GSD_r2.16",
    "file_statistics": {
        "size_bytes": $FILE_SIZE,
        "size_mb": round($FILE_SIZE / (1024 * 1024), 2),
        "encoding": "UTF-8"
    },
    "performance": {
        "tool": "sakurs",
        "runs": perf_data["results"][0]["times"].__len__(),
        "mean_time": perf_data["results"][0]["mean"],
        "stddev": perf_data["results"][0]["stddev"],
        "min_time": perf_data["results"][0]["min"],
        "max_time": perf_data["results"][0]["max"],
        "characters_per_second": int($FILE_SIZE / perf_data["results"][0]["mean"]),
        "mb_per_second": round(($FILE_SIZE / (1024 * 1024)) / perf_data["results"][0]["mean"], 2)
    },
    "accuracy": acc_data,
    "notes": "Full text available, suitable for accurate benchmarking"
}

# Save combined results
with open("$COMBINED_JSON", 'w') as f:
    json.dump(combined, f, indent=2)

print(f"Combined results saved to: {Path('$COMBINED_JSON').name}")

# Print summary
print("\n" + "="*50)
print("SUMMARY - UD Japanese-GSD")
print("="*50)
print(f"Performance: {combined['performance']['mean_time']:.3f} ± {combined['performance']['stddev']:.3f} seconds")
print(f"Throughput: {combined['performance']['characters_per_second']:,} chars/sec")
print(f"Accuracy: F1={acc_data['f1']:.4f}, P={acc_data['precision']:.4f}, R={acc_data['recall']:.4f}")
if 'pk' in acc_data:
    print(f"Pk Score: {acc_data['pk']:.4f}")
if 'window_diff' in acc_data:
    print(f"WindowDiff: {acc_data['window_diff']:.4f}")
print("="*50)
EOF

# Clean up
rm -f "$OUTPUT_FILE"

echo ""
echo -e "${BLUE}Benchmark complete!${NC}"
echo "Results saved to:"
echo "  - Performance: $(basename "$PERF_JSON")"
echo "  - Accuracy: $(basename "$ACCURACY_JSON")"
echo "  - Combined: $(basename "$COMBINED_JSON")"