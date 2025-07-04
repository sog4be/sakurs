#!/bin/bash
# Run all performance benchmarks (basic version)
# NOTE: For comprehensive performance benchmarking with Hyperfine,
# use run_all_hyperfine.sh instead.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Running basic performance benchmarks..."
echo "======================================"

# Check if enhanced scripts are available
if [ -f "$SCRIPT_DIR/run_all_hyperfine.sh" ]; then
    echo ""
    echo "Enhanced Hyperfine-based benchmarks are available!"
    echo "For comprehensive performance testing, run:"
    echo "  $SCRIPT_DIR/run_all_hyperfine.sh"
    echo ""
    echo "The enhanced version includes:"
    echo "  - Wikipedia performance benchmarks (500MB samples)"
    echo "  - Statistical analysis with multiple runs"
    echo "  - Academic-ready reports and visualizations"
    echo ""
fi

# Basic benchmark using Brown Corpus
echo ""
echo "1. Basic Performance Test (Brown Corpus)"
echo "----------------------------------------"
if [ -f "$SCRIPT_DIR/simple_benchmark.sh" ]; then
    bash "$SCRIPT_DIR/simple_benchmark.sh"
else
    echo "Simple benchmark script not found"
fi

echo ""
echo "Basic performance benchmarks complete!"
echo ""
echo "For Wikipedia-scale performance testing, run:"
echo "  ./run_all_hyperfine.sh"