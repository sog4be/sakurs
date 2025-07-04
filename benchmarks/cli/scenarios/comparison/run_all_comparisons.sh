#!/bin/bash
# Run all comparison benchmarks with comprehensive reporting
# Executes enhanced Hyperfine-based comparisons for both languages

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results/comparison"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
SUMMARY_DIR="$RESULTS_DIR/summary_${TIMESTAMP}"
COMPARISON_RUNS=${COMPARISON_RUNS:-10}

# Create directories
mkdir -p "$SUMMARY_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Functions
print_header() {
    echo ""
    echo -e "${BLUE}===============================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===============================================${NC}"
    echo ""
}

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
    local has_errors=0
    
    print_status "Checking prerequisites..."
    
    # Check for required commands
    for cmd in sakurs hyperfine python3 bc; do
        if ! command -v "$cmd" &> /dev/null; then
            print_error "$cmd not found"
            has_errors=1
        else
            print_status "✓ $cmd found"
        fi
    done
    
    # Check for baseline tools
    if ! python3 -c "import nltk" 2>/dev/null; then
        print_error "NLTK not installed (pip install nltk)"
        has_errors=1
    else
        print_status "✓ NLTK available"
    fi
    
    if ! python3 -c "import ja_sentence_segmenter" 2>/dev/null; then
        print_error "ja_sentence_segmenter not installed (pip install ja-sentence-segmenter)"
        has_errors=1
    else
        print_status "✓ ja_sentence_segmenter available"
    fi
    
    # Check for comparison scripts
    if [ ! -f "$SCRIPT_DIR/english_vs_punkt_hyperfine.sh" ]; then
        print_error "English comparison script not found"
        has_errors=1
    fi
    
    if [ ! -f "$SCRIPT_DIR/japanese_vs_jaseg_hyperfine.sh" ]; then
        print_error "Japanese comparison script not found"
        has_errors=1
    fi
    
    return $has_errors
}

run_comparison() {
    local name=$1
    local script=$2
    local lang_prefix=$3
    
    print_header "Running $name Comparison"
    
    # Set environment variables for consistency
    export COMPARISON_RUNS="$COMPARISON_RUNS"
    
    # Run the comparison
    if "$script"; then
        print_status "$name comparison completed successfully"
        
        # Copy results to summary directory
        local latest_analysis=$(ls -t "$RESULTS_DIR"/analysis_*${lang_prefix}*.json 2>/dev/null | head -1)
        if [ -n "$latest_analysis" ]; then
            cp "$latest_analysis" "$SUMMARY_DIR/${lang_prefix}_comparison_results.json"
        fi
        
        return 0
    else
        print_error "$name comparison failed"
        return 1
    fi
}

generate_comparison_summary() {
    print_header "Generating Comparison Summary Report"
    
    python3 - <<'EOF'
import json
import os
import sys
from pathlib import Path
from datetime import datetime

summary_dir = sys.argv[1]
timestamp = sys.argv[2]
comparison_runs = int(sys.argv[3])

# Load all comparison result files
results = {}
for file in Path(summary_dir).glob("*_comparison_results.json"):
    lang = file.stem.split('_')[0]  # english or japanese
    try:
        with open(file) as f:
            results[lang] = json.load(f)
    except Exception as e:
        print(f"Warning: Could not load {file}: {e}")

if not results:
    print("No comparison results found!")
    sys.exit(1)

# Generate summary
summary = {
    "benchmark_suite": "Sakurs Baseline Comparison Benchmarks",
    "timestamp": timestamp,
    "configuration": {
        "comparison_runs": comparison_runs,
        "baselines": {
            "english": "NLTK Punkt Tokenizer",
            "japanese": "ja_sentence_segmenter"
        }
    },
    "languages_tested": list(results.keys()),
    "results": {}
}

# Process each language
for lang, data in results.items():
    lang_key = lang.title()
    
    # Get baseline name
    if lang == "english":
        baseline_name = "NLTK Punkt"
        baseline_key = "punkt"
    else:
        baseline_name = "ja_sentence_segmenter"
        baseline_key = "ja_sentence_segmenter"
    
    summary["results"][lang_key] = {
        "baseline": baseline_name,
        "dataset": data["dataset"]["name"],
        "sakurs": {
            "mean_time": round(data["sakurs"]["mean_time"], 3),
            "throughput_mb_s": round(data["sakurs"]["throughput_mb_s"], 2),
            "throughput_chars_s": int(data["sakurs"]["throughput_chars_s"])
        },
        "baseline_tool": {
            "mean_time": round(data[baseline_key]["mean_time"], 3),
            "throughput_mb_s": round(data[baseline_key]["throughput_mb_s"], 2),
            "throughput_chars_s": int(data[baseline_key]["throughput_chars_s"])
        },
        "comparison": {
            "speedup": round(data["comparison"]["speedup"], 2),
            "sakurs_faster": data["comparison"]["sakurs_faster"]
        }
    }
    
    # Add language-specific metrics
    if lang == "japanese" and "character_rates" in data["sakurs"]:
        summary["results"][lang_key]["sakurs"]["character_analysis"] = {
            "hiragana_per_sec": int(data["sakurs"]["character_rates"]["hiragana_per_sec"]),
            "katakana_per_sec": int(data["sakurs"]["character_rates"]["katakana_per_sec"]),
            "kanji_per_sec": int(data["sakurs"]["character_rates"]["kanji_per_sec"])
        }

# Save summary
output_file = Path(summary_dir) / "comparison_summary.json"
with open(output_file, 'w') as f:
    json.dump(summary, f, indent=2, ensure_ascii=False)

# Generate markdown report
md_output = Path(summary_dir) / "comparison_report.md"
with open(md_output, 'w', encoding='utf-8') as f:
    f.write(f"# Sakurs Baseline Comparison Results\n\n")
    f.write(f"Generated: {timestamp}\n")
    f.write(f"Comparison runs: {comparison_runs} per benchmark\n\n")
    
    f.write("## Performance Comparison Overview\n\n")
    
    # Create comparison table
    f.write("| Language | Baseline | Sakurs Time (s) | Baseline Time (s) | Speedup | Verdict |\n")
    f.write("|----------|----------|-----------------|-------------------|---------|----------|\n")
    
    for lang_key, data in summary["results"].items():
        baseline = data["baseline"]
        sakurs_time = data["sakurs"]["mean_time"]
        baseline_time = data["baseline_tool"]["mean_time"]
        speedup = data["comparison"]["speedup"]
        verdict = "🚀 Faster" if data["comparison"]["sakurs_faster"] else "🐌 Slower"
        
        f.write(f"| {lang_key} | {baseline} | "
                f"{sakurs_time:.3f} | "
                f"{baseline_time:.3f} | "
                f"{speedup:.2f}x | "
                f"{verdict} |\n")
    
    f.write("\n## Throughput Comparison\n\n")
    
    f.write("| Language | Tool | Throughput (MB/s) | Throughput (chars/s) |\n")
    f.write("|----------|------|-------------------|----------------------|\n")
    
    for lang_key, data in summary["results"].items():
        baseline = data["baseline"]
        
        # Sakurs row
        f.write(f"| {lang_key} | Sakurs | "
                f"{data['sakurs']['throughput_mb_s']:.2f} | "
                f"{data['sakurs']['throughput_chars_s']:,} |\n")
        
        # Baseline row
        f.write(f"| {lang_key} | {baseline} | "
                f"{data['baseline_tool']['throughput_mb_s']:.2f} | "
                f"{data['baseline_tool']['throughput_chars_s']:,} |\n")
    
    f.write("\n## Detailed Results\n\n")
    
    for lang_key, data in summary["results"].items():
        f.write(f"### {lang_key} vs {data['baseline']}\n\n")
        f.write(f"- **Dataset**: {data['dataset']}\n")
        f.write(f"- **Sakurs Performance**: {data['sakurs']['mean_time']:.3f}s, {data['sakurs']['throughput_mb_s']:.2f} MB/s\n")
        f.write(f"- **{data['baseline']} Performance**: {data['baseline_tool']['mean_time']:.3f}s, {data['baseline_tool']['throughput_mb_s']:.2f} MB/s\n")
        f.write(f"- **Speed Improvement**: {data['comparison']['speedup']:.2f}x {'faster' if data['comparison']['sakurs_faster'] else 'slower'}\n")
        
        # Language-specific metrics
        if "character_analysis" in data["sakurs"]:
            f.write(f"- **Character Processing (Sakurs)**:\n")
            char_analysis = data["sakurs"]["character_analysis"]
            f.write(f"  - Hiragana: {char_analysis['hiragana_per_sec']:,} chars/s\n")
            f.write(f"  - Katakana: {char_analysis['katakana_per_sec']:,} chars/s\n")
            f.write(f"  - Kanji: {char_analysis['kanji_per_sec']:,} chars/s\n")
        
        f.write("\n")

print(f"Comparison summary generated: {output_file}")
print(f"Markdown report generated: {md_output}")

# Generate LaTeX table for academic papers
latex_output = Path(summary_dir) / "baseline_comparison.tex"
with open(latex_output, 'w') as f:
    f.write("% Sakurs Baseline Comparison Results\n")
    f.write("\\begin{table}[h]\n")
    f.write("\\centering\n")
    f.write("\\begin{tabular}{llrrr}\n")
    f.write("\\hline\n")
    f.write("Language & Baseline & Sakurs (s) & Baseline (s) & Speedup \\\\\n")
    f.write("\\hline\n")
    
    for lang_key, data in summary["results"].items():
        baseline = data["baseline"]
        sakurs_time = data["sakurs"]["mean_time"]
        baseline_time = data["baseline_tool"]["mean_time"]
        speedup = data["comparison"]["speedup"]
        
        f.write(f"{lang_key} & {baseline} & "
                f"{sakurs_time:.3f} & "
                f"{baseline_time:.3f} & "
                f"{speedup:.2f}x \\\\\n")
    
    f.write("\\hline\n")
    f.write("\\end{tabular}\n")
    f.write(f"\\caption{{Sakurs performance comparison against established baselines.}}\n")
    f.write("\\label{tab:baseline-comparison}\n")
    f.write("\\end{table}\n")

print(f"LaTeX table generated: {latex_output}")

EOF "$SUMMARY_DIR" "$TIMESTAMP" "$COMPARISON_RUNS"
}

generate_comparison_plots() {
    print_header "Generating Comparison Visualizations"
    
    # Check if we have the analysis script
    local analysis_script="$ROOT_DIR/benchmarks/cli/scripts/analyze_results.py"
    if [ -f "$analysis_script" ]; then
        print_status "Generating comparison plots..."
        
        cd "$ROOT_DIR/benchmarks/cli/scripts"
        python3 analyze_results.py \
            -i "$SUMMARY_DIR"/*_comparison_results.json \
            -o "$SUMMARY_DIR" \
            -f plots || print_warning "Plot generation failed (optional)"
    else
        print_warning "Analysis script not found, skipping plot generation"
    fi
}

# Main execution
main() {
    print_header "Sakurs Baseline Comparison Suite"
    echo "Timestamp: $TIMESTAMP"
    echo "Comparison runs: $COMPARISON_RUNS per benchmark"
    echo "Results will be saved to: $SUMMARY_DIR"
    
    # Check prerequisites
    if ! check_prerequisites; then
        print_error "Prerequisites check failed. Please install missing dependencies."
        exit 1
    fi
    
    # Track overall success
    local all_success=true
    
    # Run English vs NLTK Punkt comparison
    if ! run_comparison "English vs NLTK Punkt" "$SCRIPT_DIR/english_vs_punkt_hyperfine.sh" "english"; then
        all_success=false
        print_warning "English comparison failed, continuing with other comparisons..."
    fi
    
    # Run Japanese vs ja_sentence_segmenter comparison
    if ! run_comparison "Japanese vs ja_sentence_segmenter" "$SCRIPT_DIR/japanese_vs_jaseg_hyperfine.sh" "japanese"; then
        all_success=false
        print_warning "Japanese comparison failed, continuing..."
    fi
    
    # Generate summary report if we have at least one successful comparison
    if ls "$SUMMARY_DIR"/*_comparison_results.json &> /dev/null; then
        generate_comparison_summary
        generate_comparison_plots
    else
        print_error "No successful comparisons to summarize"
        exit 1
    fi
    
    # Final status
    if [ "$all_success" = true ]; then
        print_header "All Baseline Comparisons Completed Successfully!"
    else
        print_header "Baseline Comparisons Completed with Some Failures"
        print_warning "Check individual results for details"
    fi
    
    echo ""
    echo "Summary results saved to: $SUMMARY_DIR"
    echo ""
    echo "Key files:"
    echo "  - comparison_summary.json      # Machine-readable results"
    echo "  - comparison_report.md         # Human-readable report"
    echo "  - baseline_comparison.tex      # LaTeX table for papers"
    if [ -f "$SUMMARY_DIR/performance_comparison.png" ]; then
        echo "  - performance_comparison.png   # Performance visualization"
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
        echo "  COMPARISON_RUNS     Number of Hyperfine runs per comparison (default: 10)"
        echo ""
        echo "This script runs all baseline comparison benchmarks."
        exit 0
        ;;
    *)
        main
        ;;
esac