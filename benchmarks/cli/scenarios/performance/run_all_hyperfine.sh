#!/bin/bash
# Run all performance benchmarks with Hyperfine
# Generates comprehensive performance report for all languages

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results/performance"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
SUMMARY_DIR="$RESULTS_DIR/summary_${TIMESTAMP}"
SAMPLE_SIZE_MB=${SAMPLE_SIZE_MB:-500}
BENCHMARK_RUNS=${BENCHMARK_RUNS:-10}

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
    for cmd in sakurs hyperfine uv bc; do
        if ! command -v "$cmd" &> /dev/null; then
            print_error "$cmd not found"
            has_errors=1
        else
            print_status "✓ $cmd found"
        fi
    done
    
    # Check for benchmark scripts
    if [ ! -f "$SCRIPT_DIR/english_wikipedia_hyperfine.sh" ]; then
        print_error "English performance benchmark script not found"
        has_errors=1
    fi
    
    if [ ! -f "$SCRIPT_DIR/japanese_wikipedia_hyperfine.sh" ]; then
        print_error "Japanese performance benchmark script not found"
        has_errors=1
    fi
    
    # Check if data preparation script exists
    if [ ! -f "$SCRIPT_DIR/prepare_wikipedia_data.sh" ]; then
        print_error "Data preparation script not found"
        has_errors=1
    fi
    
    return $has_errors
}

prepare_data_if_needed() {
    print_status "Checking Wikipedia data availability..."
    
    local data_dir="$ROOT_DIR/benchmarks/data/wikipedia/cli_format"
    local en_file="$data_dir/wikipedia_en_${SAMPLE_SIZE_MB}mb.txt"
    local ja_file="$data_dir/wikipedia_ja_${SAMPLE_SIZE_MB}mb.txt"
    
    local needs_preparation=false
    
    if [ ! -f "$en_file" ]; then
        print_warning "English Wikipedia sample not found"
        needs_preparation=true
    fi
    
    if [ ! -f "$ja_file" ]; then
        print_warning "Japanese Wikipedia sample not found"
        needs_preparation=true
    fi
    
    if [ "$needs_preparation" = true ]; then
        print_status "Preparing Wikipedia data..."
        if ! "$SCRIPT_DIR/prepare_wikipedia_data.sh"; then
            print_error "Failed to prepare Wikipedia data"
            return 1
        fi
    else
        print_status "Wikipedia data is ready"
    fi
}

run_benchmark() {
    local name=$1
    local script=$2
    local output_prefix=$3
    
    print_header "Running $name Performance Benchmark"
    
    # Set environment variables for consistency
    export SAMPLE_SIZE_MB="$SAMPLE_SIZE_MB"
    export BENCHMARK_RUNS="$BENCHMARK_RUNS"
    
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

generate_performance_summary() {
    print_header "Generating Performance Summary Report"
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<'EOF'
import json
import os
import sys
from pathlib import Path

summary_dir = sys.argv[1]
timestamp = sys.argv[2]
sample_size_mb = int(sys.argv[3])

# Load all result files
results = {}
for file in Path(summary_dir).glob("*_results.json"):
    lang = file.stem.split('_')[0]  # english or japanese
    with open(file) as f:
        results[lang] = json.load(f)

# Generate summary
summary = {
    "benchmark_suite": "Sakurs Performance Benchmarks",
    "timestamp": timestamp,
    "configuration": {
        "sample_size_mb": sample_size_mb,
        "dataset_source": "Wikipedia (Hugging Face 20231101)"
    },
    "languages_tested": list(results.keys()),
    "results": {}
}

# Process each language
for lang, data in results.items():
    lang_key = lang.title()
    summary["results"][lang_key] = {
        "dataset": data["dataset"],
        "performance": {
            "mean_time_seconds": round(data["performance"]["mean_time"], 3),
            "stddev_seconds": round(data["performance"]["stddev"], 3),
            "throughput_mb_s": round(data["throughput"]["throughput_mb_s"], 2),
            "throughput_chars_s": int(data["throughput"]["throughput_chars_s"]),
            "sentences_per_sec": int(data["throughput"]["sentences_per_sec"]),
            "runs": data["performance"]["runs"]
        },
        "latency_percentiles": {
            "p50": round(data["latency_percentiles"]["p50"], 3),
            "p90": round(data["latency_percentiles"]["p90"], 3),
            "p99": round(data["latency_percentiles"]["p99"], 3)
        }
    }
    
    # Add language-specific metrics
    if lang == "japanese" and "character_rates" in data["throughput"]:
        summary["results"][lang_key]["character_analysis"] = {
            "hiragana_per_sec": int(data["throughput"]["character_rates"]["hiragana_per_sec"]),
            "katakana_per_sec": int(data["throughput"]["character_rates"]["katakana_per_sec"]),
            "kanji_per_sec": int(data["throughput"]["character_rates"]["kanji_per_sec"])
        }

# Save summary
output_file = Path(summary_dir) / "performance_summary.json"
with open(output_file, 'w') as f:
    json.dump(summary, f, indent=2, ensure_ascii=False)

# Generate markdown report
md_output = Path(summary_dir) / "performance_report.md"
with open(md_output, 'w', encoding='utf-8') as f:
    f.write(f"# Sakurs Performance Benchmark Results\n\n")
    f.write(f"Generated: {timestamp}\n")
    f.write(f"Sample Size: {sample_size_mb}MB per language\n\n")
    
    f.write("## Performance Overview\n\n")
    
    # Create comparison table
    f.write("| Language | Source | Throughput (MB/s) | Chars/s | Sentences/s | Mean Time (s) | p90 Latency (s) |\n")
    f.write("|----------|--------|-------------------|---------|-------------|---------------|------------------|\n")
    
    for lang_key, data in summary["results"].items():
        perf = data["performance"]
        latency = data["latency_percentiles"]
        
        f.write(f"| {lang_key} | {data['dataset']['source']} | "
                f"{perf['throughput_mb_s']:.2f} | "
                f"{perf['throughput_chars_s']:,} | "
                f"{perf['sentences_per_sec']:,} | "
                f"{perf['mean_time_seconds']:.3f} ± {perf['stddev_seconds']:.3f} | "
                f"{latency['p90']:.3f} |\n")
    
    f.write("\n## Detailed Results\n\n")
    
    for lang_key, data in summary["results"].items():
        f.write(f"### {lang_key}\n\n")
        f.write(f"- **Dataset**: {data['dataset']['source']}\n")
        f.write(f"- **Total Characters**: {data['dataset']['characters']:,}\n")
        f.write(f"- **Benchmark Runs**: {data['performance']['runs']}\n")
        f.write(f"- **Mean Processing Time**: {data['performance']['mean_time_seconds']:.3f}s\n")
        f.write(f"- **Throughput**: {data['performance']['throughput_mb_s']:.2f} MB/s\n")
        f.write(f"- **Character Rate**: {data['performance']['throughput_chars_s']:,} chars/s\n")
        
        # Language-specific metrics
        if "character_analysis" in data:
            f.write(f"- **Character Processing Rates**:\n")
            char_analysis = data["character_analysis"]
            f.write(f"  - Hiragana: {char_analysis['hiragana_per_sec']:,} chars/s\n")
            f.write(f"  - Katakana: {char_analysis['katakana_per_sec']:,} chars/s\n")
            f.write(f"  - Kanji: {char_analysis['kanji_per_sec']:,} chars/s\n")
        
        f.write(f"\n**Latency Distribution:**\n")
        f.write(f"- p50: {data['latency_percentiles']['p50']:.3f}s\n")
        f.write(f"- p90: {data['latency_percentiles']['p90']:.3f}s\n")
        f.write(f"- p99: {data['latency_percentiles']['p99']:.3f}s\n")
        f.write("\n")

print(f"Performance summary generated: {output_file}")
print(f"Markdown report generated: {md_output}")

# Generate LaTeX table for academic papers
latex_output = Path(summary_dir) / "performance_table.tex"
with open(latex_output, 'w') as f:
    f.write("% Sakurs Performance Benchmark Results\n")
    f.write("\\begin{table}[h]\n")
    f.write("\\centering\n")
    f.write("\\begin{tabular}{llrrrrr}\n")
    f.write("\\hline\n")
    f.write("Language & Dataset & Throughput & Chars/s & Time (s) & p90 Latency & p99 Latency \\\\\n")
    f.write("         &         & (MB/s)     &         &          & (s)         & (s)         \\\\\n")
    f.write("\\hline\n")
    
    for lang_key, data in summary["results"].items():
        perf = data["performance"]
        latency = data["latency_percentiles"]
        dataset_short = "Wiki" if "Wikipedia" in data["dataset"]["source"] else data["dataset"]["source"]
        
        f.write(f"{lang_key} & {dataset_short} & "
                f"{perf['throughput_mb_s']:.2f} & "
                f"{perf['throughput_chars_s']:,} & "
                f"{perf['mean_time_seconds']:.3f} $\\pm$ {perf['stddev_seconds']:.3f} & "
                f"{latency['p90']:.3f} & "
                f"{latency['p99']:.3f} \\\\\n")
    
    f.write("\\hline\n")
    f.write("\\end{tabular}\n")
    f.write(f"\\caption{{Sakurs performance benchmarks on {sample_size_mb}MB Wikipedia samples.}}\n")
    f.write("\\label{tab:performance-benchmarks}\n")
    f.write("\\end{table}\n")

print(f"LaTeX table generated: {latex_output}")

EOF "$SUMMARY_DIR" "$TIMESTAMP" "$SAMPLE_SIZE_MB"
}

generate_comparison_plots() {
    print_header "Generating Performance Visualizations"
    
    # Check if we have the analysis script
    local analysis_script="$ROOT_DIR/benchmarks/cli/scripts/analyze_results.py"
    if [ -f "$analysis_script" ]; then
        print_status "Generating performance plots..."
        
        cd "$ROOT_DIR/benchmarks/cli/scripts"
        cd "$ROOT_DIR/benchmarks" && uv run python cli/scenarios/performance/analyze_results.py \
            -i "$SUMMARY_DIR"/*_results.json \
            -o "$SUMMARY_DIR" \
            -f plots || print_warning "Plot generation failed (optional)"
    else
        print_warning "Analysis script not found, skipping plot generation"
    fi
}

# Main execution
main() {
    print_header "Sakurs Performance Benchmark Suite"
    echo "Timestamp: $TIMESTAMP"
    echo "Sample size: ${SAMPLE_SIZE_MB}MB per language"
    echo "Benchmark runs: $BENCHMARK_RUNS per language"
    echo "Results will be saved to: $SUMMARY_DIR"
    
    # Check prerequisites
    if ! check_prerequisites; then
        print_error "Prerequisites check failed. Please install missing dependencies."
        exit 1
    fi
    
    # Prepare data
    if ! prepare_data_if_needed; then
        print_error "Data preparation failed"
        exit 1
    fi
    
    # Track overall success
    local all_success=true
    
    # Run English performance benchmark
    if ! run_benchmark "English Wikipedia" "$SCRIPT_DIR/english_wikipedia_hyperfine.sh" "english_wikipedia"; then
        all_success=false
        print_warning "English benchmark failed, continuing with other benchmarks..."
    fi
    
    # Run Japanese performance benchmark
    if ! run_benchmark "Japanese Wikipedia" "$SCRIPT_DIR/japanese_wikipedia_hyperfine.sh" "japanese_wikipedia"; then
        all_success=false
        print_warning "Japanese benchmark failed, continuing..."
    fi
    
    # Generate summary report if we have at least one successful benchmark
    if ls "$SUMMARY_DIR"/*_results.json &> /dev/null; then
        generate_performance_summary
        generate_comparison_plots
    else
        print_error "No successful benchmarks to summarize"
        exit 1
    fi
    
    # Final status
    if [ "$all_success" = true ]; then
        print_header "All Performance Benchmarks Completed Successfully!"
    else
        print_header "Performance Benchmarks Completed with Some Failures"
        print_warning "Check individual results for details"
    fi
    
    echo ""
    echo "Summary results saved to: $SUMMARY_DIR"
    echo ""
    echo "Key files:"
    echo "  - performance_summary.json     # Machine-readable results"
    echo "  - performance_report.md        # Human-readable report"
    echo "  - performance_table.tex        # LaTeX table for papers"
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
        echo "  --detailed     Generate detailed analysis (same as default)"
        echo ""
        echo "Environment variables:"
        echo "  SAMPLE_SIZE_MB     Wikipedia sample size (default: 500)"
        echo "  BENCHMARK_RUNS     Number of Hyperfine runs (default: 10)"
        echo ""
        echo "This script runs all performance benchmarks and generates a summary report."
        exit 0
        ;;
    --detailed)
        # Detailed mode is default, so just continue
        main
        ;;
    *)
        main
        ;;
esac