#!/bin/bash
# Accuracy benchmark for UD Japanese-GSD

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Running UD Japanese-GSD accuracy benchmark${NC}"
echo "==========================================="

# Check if data exists
DATA_DIR="../../data/ud_japanese_gsd/cli_format"
TEST_FILE="$DATA_DIR/gsd_plain.txt"
GOLD_FILE="$DATA_DIR/gsd_sentences.txt"

if [ ! -f "$TEST_FILE" ]; then
    echo -e "${YELLOW}Test file not found: $TEST_FILE${NC}"
    echo "Please run: cd ../.. && uv run python cli/scripts/prepare_data.py"
    exit 1
fi

if [ ! -f "$GOLD_FILE" ]; then
    echo -e "${YELLOW}Gold file not found: $GOLD_FILE${NC}"
    echo "Please run: cd ../.. && uv run python cli/scripts/prepare_data.py"
    exit 1
fi

# Run sakurs
echo -e "\n${GREEN}Testing sakurs...${NC}"
OUTPUT_FILE="/tmp/sakurs_gsd_output.txt"
sakurs process --input "$TEST_FILE" --output "$OUTPUT_FILE" --format plain

# Calculate metrics
echo -e "\n${GREEN}Calculating metrics...${NC}"
cd ../..
uv run python cli/scripts/evaluate_accuracy.py \
    --predicted "$OUTPUT_FILE" \
    --reference "$GOLD_FILE" \
    --language japanese

# Clean up
rm -f "$OUTPUT_FILE"

echo -e "\n${BLUE}UD Japanese-GSD benchmark complete!${NC}"