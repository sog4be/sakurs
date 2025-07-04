#!/bin/bash
# Prepare Wikipedia data for performance benchmarks
# Downloads and prepares 500MB samples for English and Japanese

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DATA_DIR="$ROOT_DIR/benchmarks/data/wikipedia/cli_format"
SAMPLE_SIZE_MB=${SAMPLE_SIZE_MB:-500}
WIKIPEDIA_DATE="20231101"  # Fixed date for reproducibility

# Create directories
mkdir -p "$DATA_DIR"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Functions
print_status() {
    echo -e "${GREEN}[$(date +%H:%M:%S)]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

check_prerequisites() {
    local missing=0
    
    print_status "Checking prerequisites..."
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 not found"
        missing=1
    fi
    
    # Check if datasets is installed
    if ! python3 -c "import datasets" 2>/dev/null; then
        print_error "Python 'datasets' package not installed"
        echo "Install with: pip install datasets"
        missing=1
    fi
    
    return $missing
}

prepare_wikipedia_sample() {
    local language=$1
    local sample_file="$DATA_DIR/wikipedia_${language}_${SAMPLE_SIZE_MB}mb.txt"
    
    print_status "Preparing Wikipedia $language sample (${SAMPLE_SIZE_MB}MB)..."
    
    # Check if already exists
    if [ -f "$sample_file" ]; then
        local size_mb=$(du -m "$sample_file" | cut -f1)
        if [ "$size_mb" -ge $((SAMPLE_SIZE_MB - 10)) ]; then
            print_status "Sample already exists: $sample_file (${size_mb}MB)"
            return 0
        else
            print_warning "Existing sample too small (${size_mb}MB), recreating..."
        fi
    fi
    
    # Create sample using Python script
    print_status "Creating sample from Hugging Face Wikipedia dataset..."
    python3 - <<EOF
import sys
import os
from pathlib import Path

# Add benchmarks/data to path
sys.path.insert(0, "$ROOT_DIR/benchmarks/data")

from wikipedia.loader import WikipediaLoader

# Create loader
loader = WikipediaLoader(
    language="$language",
    size_mb=$SAMPLE_SIZE_MB,
    cache_dir=Path("$DATA_DIR").parent / "cache",
    date="$WIKIPEDIA_DATE"
)

# Download and prepare sample
sample_path = loader.download(force=False)
print(f"Sample created: {sample_path}")

# Copy to CLI format directory
import shutil
output_path = Path("$sample_file")
shutil.copy2(sample_path, output_path)
print(f"Copied to: {output_path}")

# Generate statistics
stats = loader.get_statistics()
print(f"\nStatistics:")
print(f"  Articles: {stats['articles']}")
print(f"  Total characters: {stats['total_characters']:,}")
print(f"  Total words: {stats['total_words']:,}")
print(f"  Average article length: {stats['avg_article_length']:.0f} chars")

if "$language" == "ja":
    print(f"  Hiragana: {stats['hiragana_count']:,}")
    print(f"  Katakana: {stats['katakana_count']:,}")
    print(f"  Kanji: {stats['kanji_count']:,}")
EOF
    
    if [ $? -eq 0 ]; then
        print_status "Successfully prepared $language Wikipedia sample"
    else
        print_error "Failed to prepare $language Wikipedia sample"
        return 1
    fi
}

verify_samples() {
    print_status "Verifying prepared samples..."
    
    local all_good=true
    
    for lang in en ja; do
        local sample_file="$DATA_DIR/wikipedia_${lang}_${SAMPLE_SIZE_MB}mb.txt"
        
        if [ -f "$sample_file" ]; then
            local size_mb=$(du -m "$sample_file" | cut -f1)
            local lines=$(wc -l < "$sample_file")
            local chars=$(wc -m < "$sample_file")
            
            print_status "✓ $lang: ${size_mb}MB, ${lines} lines, ${chars} characters"
        else
            print_error "✗ $lang: Sample not found"
            all_good=false
        fi
    done
    
    if [ "$all_good" = true ]; then
        print_status "All samples verified successfully"
        return 0
    else
        print_error "Some samples are missing"
        return 1
    fi
}

# Main execution
main() {
    print_status "Wikipedia Data Preparation for Performance Benchmarks"
    echo "Target size: ${SAMPLE_SIZE_MB}MB per language"
    echo "Wikipedia snapshot: $WIKIPEDIA_DATE"
    echo "Output directory: $DATA_DIR"
    echo ""
    
    # Check prerequisites
    if ! check_prerequisites; then
        print_error "Prerequisites check failed"
        exit 1
    fi
    
    # Prepare samples
    local failed=0
    
    # English Wikipedia
    if ! prepare_wikipedia_sample "en"; then
        print_error "Failed to prepare English Wikipedia sample"
        failed=1
    fi
    
    echo ""
    
    # Japanese Wikipedia
    if ! prepare_wikipedia_sample "ja"; then
        print_error "Failed to prepare Japanese Wikipedia sample"
        failed=1
    fi
    
    echo ""
    
    # Verify all samples
    if ! verify_samples; then
        failed=1
    fi
    
    echo ""
    
    if [ $failed -eq 0 ]; then
        print_status "Data preparation completed successfully!"
        echo ""
        echo "Prepared files:"
        echo "  - $DATA_DIR/wikipedia_en_${SAMPLE_SIZE_MB}mb.txt"
        echo "  - $DATA_DIR/wikipedia_ja_${SAMPLE_SIZE_MB}mb.txt"
    else
        print_error "Data preparation completed with errors"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo ""
        echo "Environment variables:"
        echo "  SAMPLE_SIZE_MB    Size of each sample in MB (default: 500)"
        echo ""
        echo "This script prepares Wikipedia samples for performance benchmarking."
        exit 0
        ;;
    *)
        main
        ;;
esac