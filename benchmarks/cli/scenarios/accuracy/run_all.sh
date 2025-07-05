#!/bin/bash
# Run all accuracy benchmarks

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Running all accuracy benchmarks..."
echo "================================"

# English benchmarks
echo ""
echo "1. UD English EWT"
echo "-----------------"
bash "$SCRIPT_DIR/english_ewt.sh"

# Japanese benchmarks
echo ""
echo "2. UD Japanese GSD"
echo "------------------"
bash "$SCRIPT_DIR/japanese_gsd.sh"

echo ""
echo "All accuracy benchmarks complete!"