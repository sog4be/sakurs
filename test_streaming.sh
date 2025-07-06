#!/bin/bash
# Test script for streaming processing with large files

set -e

echo "=== Sakurs Streaming Processing Test ==="
echo

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Create test directory
TEST_DIR="temp/streaming_tests"
mkdir -p "$TEST_DIR"

# Function to generate test file
generate_test_file() {
    local size_mb=$1
    local filename=$2
    echo -e "${YELLOW}Generating ${size_mb}MB test file: ${filename}${NC}"
    
    # Base text pattern
    local base_text="This is a test sentence. It contains multiple words and ends with a period. Another sentence follows. And yet another one! How about a question? "
    
    # Calculate iterations needed
    local base_len=${#base_text}
    local target_size=$((size_mb * 1024 * 1024))
    local iterations=$((target_size / base_len))
    
    # Generate file
    > "$filename"
    for ((i=0; i<iterations; i++)); do
        echo -n "$base_text" >> "$filename"
        if ((i % 10000 == 0)); then
            echo -ne "\rProgress: $((i * 100 / iterations))%"
        fi
    done
    echo -e "\rProgress: 100%"
    
    local actual_size=$(ls -lh "$filename" | awk '{print $5}')
    echo -e "${GREEN}Generated: ${actual_size}${NC}"
}

# Function to test file processing
test_file() {
    local filename=$1
    local description=$2
    
    echo
    echo -e "${YELLOW}Testing: ${description}${NC}"
    echo "File: $filename"
    
    # Get file size
    local file_size=$(ls -lh "$filename" | awk '{print $5}')
    echo "Size: $file_size"
    
    # Test with streaming flag
    echo -n "Processing with --stream flag... "
    local start_time=$(date +%s.%N)
    
    if target/release/sakurs process -i "$filename" --stream --quiet -f text > "$TEST_DIR/output_stream.txt" 2>&1; then
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc)
        echo -e "${GREEN}OK (${duration}s)${NC}"
        
        # Count sentences
        local sentence_count=$(wc -l < "$TEST_DIR/output_stream.txt")
        echo "Sentences found: $sentence_count"
    else
        echo -e "${RED}FAILED${NC}"
        cat "$TEST_DIR/output_stream.txt"
        return 1
    fi
    
    # Test with adaptive processing (auto-detection)
    echo -n "Processing with --adaptive flag... "
    start_time=$(date +%s.%N)
    
    if target/release/sakurs process -i "$filename" --adaptive --quiet -f text > "$TEST_DIR/output_adaptive.txt" 2>&1; then
        end_time=$(date +%s.%N)
        duration=$(echo "$end_time - $start_time" | bc)
        echo -e "${GREEN}OK (${duration}s)${NC}"
        
        # Compare outputs
        if diff -q "$TEST_DIR/output_stream.txt" "$TEST_DIR/output_adaptive.txt" > /dev/null; then
            echo -e "${GREEN}✓ Outputs match${NC}"
        else
            echo -e "${RED}✗ Outputs differ!${NC}"
            echo "Differences:"
            diff "$TEST_DIR/output_stream.txt" "$TEST_DIR/output_adaptive.txt" | head -20
        fi
    else
        echo -e "${RED}FAILED${NC}"
        cat "$TEST_DIR/output_adaptive.txt"
        return 1
    fi
}

# Build the project in release mode
echo -e "${YELLOW}Building sakurs in release mode...${NC}"
cargo build --release --bin sakurs

# Test different file sizes
echo
echo "=== Generating Test Files ==="

# Small file (1MB)
if [ ! -f "$TEST_DIR/test_1mb.txt" ]; then
    generate_test_file 1 "$TEST_DIR/test_1mb.txt"
else
    echo -e "${GREEN}Using existing 1MB test file${NC}"
fi

# Medium file (50MB)
if [ ! -f "$TEST_DIR/test_50mb.txt" ]; then
    generate_test_file 50 "$TEST_DIR/test_50mb.txt"
else
    echo -e "${GREEN}Using existing 50MB test file${NC}"
fi

# Large file (100MB)
if [ ! -f "$TEST_DIR/test_100mb.txt" ]; then
    generate_test_file 100 "$TEST_DIR/test_100mb.txt"
else
    echo -e "${GREEN}Using existing 100MB test file${NC}"
fi

# Very large file (500MB) - optional
if [ "$1" == "--large" ]; then
    if [ ! -f "$TEST_DIR/test_500mb.txt" ]; then
        generate_test_file 500 "$TEST_DIR/test_500mb.txt"
    else
        echo -e "${GREEN}Using existing 500MB test file${NC}"
    fi
fi

# Run tests
echo
echo "=== Running Tests ==="

test_file "$TEST_DIR/test_1mb.txt" "Small file (1MB)"
test_file "$TEST_DIR/test_50mb.txt" "Medium file (50MB)"
test_file "$TEST_DIR/test_100mb.txt" "Large file (100MB)"

if [ "$1" == "--large" ]; then
    test_file "$TEST_DIR/test_500mb.txt" "Very large file (500MB)"
fi

# Memory usage test (if available on macOS)
if command -v gtime &> /dev/null; then
    echo
    echo "=== Memory Usage Test ==="
    echo -e "${YELLOW}Testing memory usage with 100MB file...${NC}"
    
    echo "Streaming mode:"
    gtime -v cargo run --release --bin sakurs -- process -i "$TEST_DIR/test_100mb.txt" --stream --quiet -f text > /dev/null 2>&1 | grep "Maximum resident set size"
    
    echo "Adaptive mode:"
    gtime -v cargo run --release --bin sakurs -- process -i "$TEST_DIR/test_100mb.txt" --adaptive --quiet -f text > /dev/null 2>&1 | grep "Maximum resident set size"
else
    echo
    echo -e "${YELLOW}Note: Install GNU time (gtime) to measure memory usage${NC}"
    echo "  brew install gnu-time"
fi

echo
echo -e "${GREEN}=== All tests completed ===${NC}"

# Cleanup option
echo
read -p "Clean up test files? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$TEST_DIR"
    echo "Test files removed."
else
    echo "Test files kept in: $TEST_DIR"
fi