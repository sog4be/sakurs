#!/bin/bash
# Run all accuracy benchmarks with Hyperfine
# Generates comprehensive report for all languages

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results/accuracy"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
SUMMARY_DIR="$RESULTS_DIR/summary_${TIMESTAMP}"

# Create directories
mkdir -p "$SUMMARY_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
    for cmd in sakurs hyperfine uv jq bc; do
        if ! command -v "$cmd" &> /dev/null; then
            print_error "$cmd not found"
            has_errors=1
        else
            print_status "✓ $cmd found"
        fi
    done
    
    # Check for benchmark scripts
    if [ ! -f "$SCRIPT_DIR/english_ewt_hyperfine.sh" ]; then
        print_error "English benchmark script not found"
        has_errors=1
    fi
    
    if [ ! -f "$SCRIPT_DIR/japanese_bccwj_hyperfine.sh" ]; then
        print_error "Japanese benchmark script not found"
        has_errors=1
    fi
    
    return $has_errors
}

run_benchmark() {
    local name=$1
    local script=$2
    local output_prefix=$3
    
    print_header "Running $name Benchmark"
    
    # Run the benchmark
    if "$script"; then
        print_status "$name benchmark completed successfully"
        
        # Copy results to summary directory
        local latest_combined=$(ls -t "$RESULTS_DIR"/combined_${output_prefix}_*.json 2>/dev/null | head -1)
        if [ -n "$latest_combined" ]; then
            cp "$latest_combined" "$SUMMARY_DIR/${output_prefix}_results.json"
        fi
        
        return 0
    else
        print_error "$name benchmark failed"
        return 1
    fi
}

generate_summary_report() {
    print_header "Generating Summary Report"
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<'EOF'
import json
import os
import sys
from pathlib import Path

summary_dir = sys.argv[1]
timestamp = sys.argv[2]

# Load all result files
results = {}
for file in Path(summary_dir).glob("*_results.json"):
    lang = file.stem.split('_')[0]  # english or japanese
    with open(file) as f:
        results[lang] = json.load(f)

# Generate summary
summary = {
    "benchmark_suite": "Sakurs Accuracy Benchmarks",
    "timestamp": timestamp,
    "languages_tested": list(results.keys()),
    "results": {}
}

# Process each language
for lang, data in results.items():
    summary["results"][lang] = {
        "dataset": data["dataset"],
        "performance": {
            "mean_time_seconds": round(data["performance"]["mean_time"], 3),
            "stddev_seconds": round(data["performance"]["stddev"], 3),
            "runs": data["performance"]["runs"]
        },
        "accuracy": {
            "f1_score": round(data["accuracy"]["f1"], 4),
            "precision": round(data["accuracy"]["precision"], 4),
            "recall": round(data["accuracy"]["recall"], 4),
            "sentences": data["accuracy"]["reference_sentences"]
        }
    }
    
    # Add language-specific metrics
    if lang == "japanese" and "characters_per_second" in data["performance"]:
        summary["results"][lang]["performance"]["chars_per_second"] = data["performance"]["characters_per_second"]

# Save summary
output_file = Path(summary_dir) / "benchmark_summary.json"
with open(output_file, 'w') as f:
    json.dump(summary, f, indent=2)

# Generate markdown report
md_output = Path(summary_dir) / "benchmark_summary.md"
with open(md_output, 'w') as f:
    f.write(f"# Sakurs Accuracy Benchmark Summary\n\n")
    f.write(f"Generated: {timestamp}\n\n")
    
    f.write("## Results Overview\n\n")
    
    # Create comparison table
    f.write("| Language | Dataset | F1 Score | Precision | Recall | Mean Time (s) | Throughput |\n")
    f.write("|----------|---------|----------|-----------|---------|---------------|------------|\n")
    
    for lang, data in results.items():
        r = summary["results"][lang]
        throughput = f"{r['performance'].get('chars_per_second', 'N/A'):,}" if lang == "japanese" else "N/A"
        throughput_unit = "chars/s" if lang == "japanese" else ""
        
        f.write(f"| {lang.title()} | {r['dataset']} | "
                f"{r['accuracy']['f1_score']:.4f} | "
                f"{r['accuracy']['precision']:.4f} | "
                f"{r['accuracy']['recall']:.4f} | "
                f"{r['performance']['mean_time_seconds']:.3f} ± {r['performance']['stddev_seconds']:.3f} | "
                f"{throughput} {throughput_unit} |\n")
    
    f.write("\n## Detailed Results\n\n")
    
    for lang, data in results.items():
        f.write(f"### {lang.title()}\n\n")
        f.write(f"- **Dataset**: {data['dataset']}\n")
        f.write(f"- **Total Sentences**: {data['accuracy']['reference_sentences']:,}\n")
        f.write(f"- **Benchmark Runs**: {data['performance']['runs']}\n")
        
        if "notes" in data:
            f.write(f"- **Notes**: {data['notes']}\n")
        
        f.write("\n")

print(f"Summary report generated: {output_file}")
print(f"Markdown report generated: {md_output}")

# Generate LaTeX table for academic papers
latex_output = Path(summary_dir) / "benchmark_table.tex"
with open(latex_output, 'w') as f:
    f.write("% Sakurs Accuracy Benchmark Results\n")
    f.write("\\begin{table}[h]\n")
    f.write("\\centering\n")
    f.write("\\begin{tabular}{lllrrr}\n")
    f.write("\\hline\n")
    f.write("Language & Dataset & F1 & Precision & Recall & Time (s) \\\\\n")
    f.write("\\hline\n")
    
    for lang, data in results.items():
        r = summary["results"][lang]
        f.write(f"{lang.title()} & {r['dataset']} & "
                f"{r['accuracy']['f1_score']:.4f} & "
                f"{r['accuracy']['precision']:.4f} & "
                f"{r['accuracy']['recall']:.4f} & "
                f"{r['performance']['mean_time_seconds']:.3f} $\\pm$ {r['performance']['stddev_seconds']:.3f} \\\\\n")
    
    f.write("\\hline\n")
    f.write("\\end{tabular}\n")
    f.write(f"\\caption{{Sakurs accuracy benchmark results on standard datasets.}}\n")
    f.write("\\label{tab:accuracy-benchmarks}\n")
    f.write("\\end{table}\n")

print(f"LaTeX table generated: {latex_output}")

EOF "$SUMMARY_DIR" "$TIMESTAMP"
}

# Main execution
main() {
    print_header "Sakurs Accuracy Benchmark Suite"
    echo "Timestamp: $TIMESTAMP"
    echo "Results will be saved to: $SUMMARY_DIR"
    
    # Check prerequisites
    if ! check_prerequisites; then
        print_error "Prerequisites check failed. Please install missing dependencies."
        exit 1
    fi
    
    # Track overall success
    local all_success=true
    
    # Run English benchmark
    if ! run_benchmark "English EWT" "$SCRIPT_DIR/english_ewt_hyperfine.sh" "english_ewt"; then
        all_success=false
        print_warning "English benchmark failed, continuing with other benchmarks..."
    fi
    
    # Run Japanese benchmark
    if ! run_benchmark "Japanese BCCWJ" "$SCRIPT_DIR/japanese_bccwj_hyperfine.sh" "japanese_bccwj"; then
        all_success=false
        print_warning "Japanese benchmark failed, continuing..."
    fi
    
    # Generate summary report if we have at least one successful benchmark
    if ls "$SUMMARY_DIR"/*_results.json &> /dev/null; then
        generate_summary_report
    else
        print_error "No successful benchmarks to summarize"
        exit 1
    fi
    
    # Final status
    if [ "$all_success" = true ]; then
        print_header "All Benchmarks Completed Successfully!"
    else
        print_header "Benchmarks Completed with Some Failures"
        print_warning "Check individual results for details"
    fi
    
    echo ""
    echo "Summary results saved to: $SUMMARY_DIR"
    echo ""
    echo "Key files:"
    echo "  - benchmark_summary.json    # Machine-readable results"
    echo "  - benchmark_summary.md      # Human-readable report"
    echo "  - benchmark_table.tex       # LaTeX table for papers"
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --quiet, -q    Reduce output verbosity"
        echo ""
        echo "This script runs all accuracy benchmarks and generates a summary report."
        exit 0
        ;;
    --quiet|-q)
        # Redirect verbose output
        exec 3>&1
        exec 1>/dev/null
        main
        exec 1>&3
        ;;
    *)
        main
        ;;
esac