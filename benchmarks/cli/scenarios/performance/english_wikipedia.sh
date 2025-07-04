#!/bin/bash
# Performance benchmark on English Wikipedia sample

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/wikipedia/cache"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")

# Default sample size
SAMPLE_SIZE=${SAMPLE_SIZE:-500}
SAMPLE_FILE="wikipedia_en_${SAMPLE_SIZE}mb.txt"

# Ensure directories exist
mkdir -p "$RESULTS_DIR"

# Check if Wikipedia sample exists
if [ ! -f "$DATA_DIR/$SAMPLE_FILE" ]; then
    echo "Error: Wikipedia sample not found: $DATA_DIR/$SAMPLE_FILE"
    echo "Please prepare the data first:"
    echo "  cd $ROOT_DIR/benchmarks/cli && python scripts/prepare_data.py"
    exit 1
fi

# Check if sakurs is available
if ! command -v sakurs &> /dev/null; then
    echo "Error: sakurs not found in PATH"
    exit 1
fi

# Check if hyperfine is available
if ! command -v hyperfine &> /dev/null; then
    echo "Error: hyperfine not found. Please install it:"
    echo "  brew install hyperfine  # macOS"
    echo "  cargo install hyperfine # Cross-platform"
    exit 1
fi

echo "Running performance benchmark on English Wikipedia (${SAMPLE_SIZE}MB sample)..."

# Get file size for throughput calculation
FILE_SIZE=$(wc -c < "$DATA_DIR/$SAMPLE_FILE" | tr -d ' ')
FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1048576" | bc)

echo "Sample size: ${FILE_SIZE_MB} MB"

# Basic performance benchmark
echo ""
echo "1. Basic Performance Test"
echo "========================"
hyperfine \
    --warmup 3 \
    --runs 10 \
    --export-json "$RESULTS_DIR/perf_wikipedia_en_basic_${TIMESTAMP}.json" \
    --export-markdown "$RESULTS_DIR/perf_wikipedia_en_basic_${TIMESTAMP}.md" \
    "sakurs process --input '$DATA_DIR/$SAMPLE_FILE' --output /dev/null --format text --language english"

# Thread scaling benchmark
echo ""
echo "2. Thread Scaling Test"
echo "====================="
hyperfine \
    --warmup 2 \
    --runs 5 \
    --parameter-list threads 1,2,4,8 \
    --export-json "$RESULTS_DIR/perf_wikipedia_en_threads_${TIMESTAMP}.json" \
    --export-markdown "$RESULTS_DIR/perf_wikipedia_en_threads_${TIMESTAMP}.md" \
    "sakurs process --input '$DATA_DIR/$SAMPLE_FILE' --output /dev/null --format text --language english --threads {threads}"

# Memory usage test (if available on platform)
echo ""
echo "3. Memory Usage Test"
echo "==================="
if command -v /usr/bin/time &> /dev/null; then
    echo "Running memory profiling..."
    
    # macOS time command
    if [[ "$OSTYPE" == "darwin"* ]]; then
        /usr/bin/time -l sakurs process \
            --input "$DATA_DIR/$SAMPLE_FILE" \
            --output /dev/null \
            --format text \
            --language english \
            2>&1 | tee "$RESULTS_DIR/perf_wikipedia_en_memory_${TIMESTAMP}.txt"
    else
        # Linux time command
        /usr/bin/time -v sakurs process \
            --input "$DATA_DIR/$SAMPLE_FILE" \
            --output /dev/null \
            --format text \
            --language english \
            2>&1 | tee "$RESULTS_DIR/perf_wikipedia_en_memory_${TIMESTAMP}.txt"
    fi
else
    echo "Memory profiling not available (time command not found)"
fi

# Calculate and display throughput
echo ""
echo "Performance Summary"
echo "==================="

if command -v jq &> /dev/null && [ -f "$RESULTS_DIR/perf_wikipedia_en_basic_${TIMESTAMP}.json" ]; then
    MEAN_TIME=$(jq '.results[0].mean' "$RESULTS_DIR/perf_wikipedia_en_basic_${TIMESTAMP}.json")
    THROUGHPUT=$(echo "scale=2; $FILE_SIZE_MB / $MEAN_TIME" | bc)
    
    echo "File size: ${FILE_SIZE_MB} MB"
    echo "Mean processing time: ${MEAN_TIME} seconds"
    echo "Throughput: ${THROUGHPUT} MB/s"
    
    # Sentences per second (estimate)
    ESTIMATED_SENTENCES=$(echo "scale=0; $FILE_SIZE / 100" | bc)  # Rough estimate: ~100 chars/sentence
    SENTENCES_PER_SEC=$(echo "scale=0; $ESTIMATED_SENTENCES / $MEAN_TIME" | bc)
    echo "Estimated sentences/sec: ${SENTENCES_PER_SEC}"
fi

echo ""
echo "Results saved to: $RESULTS_DIR/perf_wikipedia_en_*_${TIMESTAMP}.*"