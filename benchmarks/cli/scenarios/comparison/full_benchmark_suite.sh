#!/bin/bash
# Complete benchmark suite: Accuracy + Performance + Comparison
# Runs all benchmarks and generates unified academic-ready report

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
RESULTS_DIR="$ROOT_DIR/benchmarks/cli/results"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
SUITE_DIR="$RESULTS_DIR/full_suite_${TIMESTAMP}"
WITH_ANALYSIS=${WITH_ANALYSIS:-false}

# Create directories
mkdir -p "$SUITE_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Functions
print_header() {
    echo ""
    echo -e "${PURPLE}===============================================${NC}"
    echo -e "${PURPLE}$1${NC}"
    echo -e "${PURPLE}===============================================${NC}"
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

print_phase() {
    echo ""
    echo -e "${BLUE}### $1 ###${NC}"
    echo ""
}

check_prerequisites() {
    local has_errors=0
    
    print_status "Checking complete benchmark suite prerequisites..."
    
    # Check for required commands
    for cmd in sakurs hyperfine uv bc jq; do
        if ! command -v "$cmd" &> /dev/null; then
            print_error "$cmd not found"
            has_errors=1
        else
            print_status "✓ $cmd found"
        fi
    done
    
    # Check for benchmark scripts
    local scripts=(
        "scenarios/accuracy/run_all_hyperfine.sh"
        "scenarios/performance/run_all_hyperfine.sh"
        "scenarios/comparison/run_all_comparisons.sh"
    )
    
    for script in "${scripts[@]}"; do
        if [ ! -f "$ROOT_DIR/benchmarks/cli/$script" ]; then
            print_error "Required script not found: $script"
            has_errors=1
        else
            print_status "✓ $script available"
        fi
    done
    
    # Check baseline tools
    if ! (cd "$ROOT_DIR/benchmarks" && uv run python -c "import nltk") 2>/dev/null; then
        print_warning "NLTK not installed - comparison benchmarks will be skipped"
    fi
    
    if ! (cd "$ROOT_DIR/benchmarks" && uv run python -c "import ja_sentence_segmenter") 2>/dev/null; then
        print_warning "ja_sentence_segmenter not installed - Japanese comparison will be skipped"
    fi
    
    return $has_errors
}

prepare_all_data() {
    print_phase "Data Preparation"
    
    print_status "Preparing all benchmark datasets..."
    
    # Prepare UD corpora
    print_status "Preparing UD corpora for accuracy benchmarks..."
    cd "$ROOT_DIR/benchmarks"
    cd "$ROOT_DIR/benchmarks" && uv run python cli/scripts/prepare_data.py || print_warning "Some UD data preparation failed"
    
    # Prepare Wikipedia data
    print_status "Preparing Wikipedia data for performance benchmarks..."
    cd "$ROOT_DIR/benchmarks/cli/scenarios/performance"
    if [ -f "prepare_wikipedia_data.sh" ]; then
        ./prepare_wikipedia_data.sh || print_warning "Wikipedia data preparation failed"
    else
        print_warning "Wikipedia preparation script not found"
    fi
    
    cd "$ROOT_DIR"
}

run_accuracy_benchmarks() {
    print_phase "Accuracy Benchmarks"
    
    print_status "Running comprehensive accuracy benchmarks..."
    
    cd "$ROOT_DIR/benchmarks/cli/scenarios/accuracy"
    
    if [ -f "run_all_hyperfine.sh" ]; then
        if ./run_all_hyperfine.sh; then
            print_status "Accuracy benchmarks completed successfully"
            
            # Copy results to suite directory
            local latest_summary=$(ls -td "$ROOT_DIR/benchmarks/cli/results/accuracy"/summary_* 2>/dev/null | head -1)
            if [ -n "$latest_summary" ]; then
                cp -r "$latest_summary" "$SUITE_DIR/accuracy_results"
                print_status "Accuracy results copied to suite directory"
            fi
            
            return 0
        else
            print_error "Accuracy benchmarks failed"
            return 1
        fi
    else
        print_error "Accuracy benchmark script not found"
        return 1
    fi
}

run_performance_benchmarks() {
    print_phase "Performance Benchmarks"
    
    print_status "Running comprehensive performance benchmarks..."
    
    cd "$ROOT_DIR/benchmarks/cli/scenarios/performance"
    
    if [ -f "run_all_hyperfine.sh" ]; then
        if ./run_all_hyperfine.sh; then
            print_status "Performance benchmarks completed successfully"
            
            # Copy results to suite directory
            local latest_summary=$(ls -td "$ROOT_DIR/benchmarks/cli/results/performance"/summary_* 2>/dev/null | head -1)
            if [ -n "$latest_summary" ]; then
                cp -r "$latest_summary" "$SUITE_DIR/performance_results"
                print_status "Performance results copied to suite directory"
            fi
            
            return 0
        else
            print_error "Performance benchmarks failed"
            return 1
        fi
    else
        print_error "Performance benchmark script not found"
        return 1
    fi
}

run_comparison_benchmarks() {
    print_phase "Baseline Comparison Benchmarks"
    
    print_status "Running baseline comparison benchmarks..."
    
    cd "$ROOT_DIR/benchmarks/cli/scenarios/comparison"
    
    if [ -f "run_all_comparisons.sh" ]; then
        if ./run_all_comparisons.sh; then
            print_status "Comparison benchmarks completed successfully"
            
            # Copy results to suite directory
            local latest_summary=$(ls -td "$ROOT_DIR/benchmarks/cli/results/comparison"/summary_* 2>/dev/null | head -1)
            if [ -n "$latest_summary" ]; then
                cp -r "$latest_summary" "$SUITE_DIR/comparison_results"
                print_status "Comparison results copied to suite directory"
            fi
            
            return 0
        else
            print_error "Comparison benchmarks failed"
            return 1
        fi
    else
        print_error "Comparison benchmark script not found"
        return 1
    fi
}

generate_unified_report() {
    print_phase "Unified Report Generation"
    
    print_status "Generating comprehensive unified report..."
    
    cd "$ROOT_DIR/benchmarks" && uv run python - <<'EOF'
import json
import os
import sys
from pathlib import Path
from datetime import datetime

suite_dir = Path(sys.argv[1])
timestamp = sys.argv[2]

# Load results from each benchmark type
benchmark_results = {}

# Load accuracy results
accuracy_dir = suite_dir / "accuracy_results"
if accuracy_dir.exists():
    summary_file = accuracy_dir / "benchmark_summary.json"
    if summary_file.exists():
        with open(summary_file) as f:
            benchmark_results["accuracy"] = json.load(f)

# Load performance results
performance_dir = suite_dir / "performance_results"
if performance_dir.exists():
    summary_file = performance_dir / "performance_summary.json"
    if summary_file.exists():
        with open(summary_file) as f:
            benchmark_results["performance"] = json.load(f)

# Load comparison results
comparison_dir = suite_dir / "comparison_results"
if comparison_dir.exists():
    summary_file = comparison_dir / "comparison_summary.json"
    if summary_file.exists():
        with open(summary_file) as f:
            benchmark_results["comparison"] = json.load(f)

# Generate unified summary
unified_report = {
    "sakurs_benchmark_suite": {
        "timestamp": timestamp,
        "benchmark_types": list(benchmark_results.keys()),
        "summary": {}
    }
}

# Process each benchmark type
for bench_type, data in benchmark_results.items():
    if bench_type == "accuracy":
        unified_report["sakurs_benchmark_suite"]["summary"]["accuracy"] = {
            "languages_tested": data.get("languages_tested", []),
            "datasets": {},
            "overall_performance": {}
        }
        
        for lang, result in data.get("results", {}).items():
            unified_report["sakurs_benchmark_suite"]["summary"]["accuracy"]["datasets"][lang] = {
                "dataset": result["dataset"],
                "f1_score": result["accuracy"]["f1_score"],
                "precision": result["accuracy"]["precision"],
                "recall": result["accuracy"]["recall"],
                "mean_time_seconds": result["performance"]["mean_time_seconds"]
            }
    
    elif bench_type == "performance":
        unified_report["sakurs_benchmark_suite"]["summary"]["performance"] = {
            "languages_tested": data.get("languages_tested", []),
            "datasets": {}
        }
        
        for lang, result in data.get("results", {}).items():
            unified_report["sakurs_benchmark_suite"]["summary"]["performance"]["datasets"][lang] = {
                "dataset": result["dataset"]["source"],
                "throughput_mb_s": result["performance"]["throughput_mb_s"],
                "throughput_chars_s": result["performance"]["throughput_chars_s"],
                "mean_time_seconds": result["performance"]["mean_time_seconds"],
                "p90_latency": result["latency_percentiles"]["p90"]
            }
    
    elif bench_type == "comparison":
        unified_report["sakurs_benchmark_suite"]["summary"]["comparison"] = {
            "baselines": data.get("configuration", {}).get("baselines", {}),
            "results": {}
        }
        
        for lang, result in data.get("results", {}).items():
            unified_report["sakurs_benchmark_suite"]["summary"]["comparison"]["results"][lang] = {
                "baseline": result["baseline"],
                "speedup": result["comparison"]["speedup"],
                "sakurs_faster": result["comparison"]["sakurs_faster"],
                "sakurs_throughput": result["sakurs"]["throughput_mb_s"],
                "baseline_throughput": result["baseline_tool"]["throughput_mb_s"]
            }

# Save unified report
output_file = suite_dir / "unified_benchmark_report.json"
with open(output_file, 'w') as f:
    json.dump(unified_report, f, indent=2, ensure_ascii=False)

# Generate comprehensive markdown report
md_output = suite_dir / "comprehensive_benchmark_report.md"
with open(md_output, 'w', encoding='utf-8') as f:
    f.write("# Sakurs Comprehensive Benchmark Report\n\n")
    f.write(f"Generated: {timestamp}\n\n")
    f.write("This report summarizes results from all Sakurs benchmarks:\n")
    f.write("- **Accuracy Benchmarks**: Performance on annotated corpora (UD English EWT, UD Japanese-GSD)\n")
    f.write("- **Performance Benchmarks**: Throughput and latency on large datasets (Wikipedia 500MB samples)\n")
    f.write("- **Baseline Comparisons**: Head-to-head comparison with established tools (NLTK Punkt, ja_sentence_segmenter)\n\n")
    
    # Executive Summary
    f.write("## Executive Summary\n\n")
    
    if "accuracy" in benchmark_results:
        f.write("### Accuracy Performance\n")
        acc_data = benchmark_results["accuracy"]
        for lang, result in acc_data.get("results", {}).items():
            f1 = result["accuracy"]["f1_score"]
            f.write(f"- **{lang}**: F1 score {f1:.4f} on {result['dataset']}\n")
        f.write("\n")
    
    if "performance" in benchmark_results:
        f.write("### Throughput Performance\n")
        perf_data = benchmark_results["performance"]
        for lang, result in perf_data.get("results", {}).items():
            throughput = result["performance"]["throughput_mb_s"]
            f.write(f"- **{lang}**: {throughput:.2f} MB/s on {result['dataset']['source']}\n")
        f.write("\n")
    
    if "comparison" in benchmark_results:
        f.write("### Baseline Comparison\n")
        comp_data = benchmark_results["comparison"]
        for lang, result in comp_data.get("results", {}).items():
            speedup = result["comparison"]["speedup"]
            status = "faster" if result["comparison"]["sakurs_faster"] else "slower"
            f.write(f"- **{lang}**: {speedup:.2f}x {status} than {result['baseline']}\n")
        f.write("\n")
    
    # Detailed sections for each benchmark type
    for bench_type, data in benchmark_results.items():
        f.write(f"## {bench_type.title()} Benchmarks\n\n")
        
        if bench_type == "accuracy":
            f.write("| Language | Dataset | F1 Score | Precision | Recall | Time (s) |\n")
            f.write("|----------|---------|----------|-----------|--------|----------|\n")
            
            for lang, result in data.get("results", {}).items():
                acc = result["accuracy"]
                time_s = result["performance"]["mean_time_seconds"]
                f.write(f"| {lang} | {result['dataset']} | "
                        f"{acc['f1_score']:.4f} | "
                        f"{acc['precision']:.4f} | "
                        f"{acc['recall']:.4f} | "
                        f"{time_s:.3f} |\n")
        
        elif bench_type == "performance":
            f.write("| Language | Source | Throughput (MB/s) | Chars/s | p90 Latency (s) |\n")
            f.write("|----------|--------|-------------------|---------|------------------|\n")
            
            for lang, result in data.get("results", {}).items():
                perf = result["performance"]
                latency = result["latency_percentiles"]
                f.write(f"| {lang} | {result['dataset']['source']} | "
                        f"{perf['throughput_mb_s']:.2f} | "
                        f"{perf['throughput_chars_s']:,} | "
                        f"{latency['p90']:.3f} |\n")
        
        elif bench_type == "comparison":
            f.write("| Language | Baseline | Speedup | Sakurs (MB/s) | Baseline (MB/s) | Result |\n")
            f.write("|----------|----------|---------|---------------|-----------------|--------|\n")
            
            for lang, result in data.get("results", {}).items():
                speedup = result["comparison"]["speedup"]
                sakurs_tp = result["sakurs"]["throughput_mb_s"]
                baseline_tp = result["baseline_tool"]["throughput_mb_s"]
                verdict = "🚀 Faster" if result["comparison"]["sakurs_faster"] else "🐌 Slower"
                
                f.write(f"| {lang} | {result['baseline']} | "
                        f"{speedup:.2f}x | "
                        f"{sakurs_tp:.2f} | "
                        f"{baseline_tp:.2f} | "
                        f"{verdict} |\n")
        
        f.write("\n")

print(f"Unified report generated: {output_file}")
print(f"Comprehensive report generated: {md_output}")

# Generate academic LaTeX summary
latex_output = suite_dir / "academic_summary.tex"
with open(latex_output, 'w') as f:
    f.write("% Sakurs Comprehensive Benchmark Results\n")
    f.write("% Academic summary for publication\n\n")
    
    # Accuracy results table
    if "accuracy" in benchmark_results:
        f.write("\\begin{table}[h]\n")
        f.write("\\centering\n")
        f.write("\\caption{Sakurs Accuracy Benchmark Results}\n")
        f.write("\\begin{tabular}{llrrr}\n")
        f.write("\\hline\n")
        f.write("Language & Dataset & F1 & Precision & Recall \\\\\n")
        f.write("\\hline\n")
        
        acc_data = benchmark_results["accuracy"]
        for lang, result in acc_data.get("results", {}).items():
            acc = result["accuracy"]
            f.write(f"{lang} & {result['dataset']} & "
                    f"{acc['f1_score']:.4f} & "
                    f"{acc['precision']:.4f} & "
                    f"{acc['recall']:.4f} \\\\\n")
        
        f.write("\\hline\n")
        f.write("\\end{tabular}\n")
        f.write("\\label{tab:accuracy-results}\n")
        f.write("\\end{table}\n\n")
    
    # Performance results table
    if "performance" in benchmark_results:
        f.write("\\begin{table}[h]\n")
        f.write("\\centering\n")
        f.write("\\caption{Sakurs Performance Benchmark Results}\n")
        f.write("\\begin{tabular}{llrr}\n")
        f.write("\\hline\n")
        f.write("Language & Dataset & Throughput (MB/s) & p90 Latency (s) \\\\\n")
        f.write("\\hline\n")
        
        perf_data = benchmark_results["performance"]
        for lang, result in perf_data.get("results", {}).items():
            perf = result["performance"]
            latency = result["latency_percentiles"]
            dataset = "Wikipedia" if "Wikipedia" in result["dataset"]["source"] else "Other"
            f.write(f"{lang} & {dataset} & "
                    f"{perf['throughput_mb_s']:.2f} & "
                    f"{latency['p90']:.3f} \\\\\n")
        
        f.write("\\hline\n")
        f.write("\\end{tabular}\n")
        f.write("\\label{tab:performance-results}\n")
        f.write("\\end{table}\n\n")

print(f"Academic LaTeX summary generated: {latex_output}")

EOF "$SUITE_DIR" "$TIMESTAMP"
}

run_advanced_analysis() {
    if [ "$WITH_ANALYSIS" != "true" ]; then
        print_status "Advanced analysis skipped (use --with-analysis to enable)"
        return 0
    fi
    
    print_phase "Advanced Statistical Analysis"
    
    print_status "Running advanced statistical analysis..."
    
    # Check if analysis script exists
    local analysis_script="$ROOT_DIR/benchmarks/cli/scripts/analyze_results.py"
    if [ ! -f "$analysis_script" ]; then
        print_warning "Analysis script not found, skipping advanced analysis"
        return 0
    fi
    
    cd "$ROOT_DIR/benchmarks/cli/scripts"
    
    # Collect all result files
    local result_files=()
    for dir in "$SUITE_DIR"/*/; do
        if [ -d "$dir" ]; then
            mapfile -t -O "${#result_files[@]}" result_files < <(find "$dir" -name "*.json" -type f)
        fi
    done
    
    if [ ${#result_files[@]} -gt 0 ]; then
        cd "$ROOT_DIR/benchmarks" && uv run python cli/scenarios/comparison/analyze_results.py \
            -i "${result_files[@]}" \
            -o "$SUITE_DIR/advanced_analysis" \
            -f all \
            --statistical-tests \
            --performance-focus || print_warning "Advanced analysis failed (optional)"
        
        print_status "Advanced analysis completed"
    else
        print_warning "No result files found for advanced analysis"
    fi
}

# Main execution
main() {
    print_header "Sakurs Complete Benchmark Suite"
    echo "Timestamp: $TIMESTAMP"
    echo "Suite directory: $SUITE_DIR"
    echo "Advanced analysis: $WITH_ANALYSIS"
    echo ""
    
    # Check prerequisites
    if ! check_prerequisites; then
        print_error "Prerequisites check failed. Some benchmarks may be skipped."
    fi
    
    # Track overall success
    local phases_run=0
    local phases_succeeded=0
    
    # Phase 1: Data Preparation
    prepare_all_data
    
    # Phase 2: Accuracy Benchmarks
    ((phases_run++))
    if run_accuracy_benchmarks; then
        ((phases_succeeded++))
    fi
    
    # Phase 3: Performance Benchmarks
    ((phases_run++))
    if run_performance_benchmarks; then
        ((phases_succeeded++))
    fi
    
    # Phase 4: Comparison Benchmarks
    ((phases_run++))
    if run_comparison_benchmarks; then
        ((phases_succeeded++))
    fi
    
    # Phase 5: Unified Report
    if [ $phases_succeeded -gt 0 ]; then
        generate_unified_report
        run_advanced_analysis
    else
        print_error "No benchmarks succeeded, cannot generate unified report"
        exit 1
    fi
    
    # Final status
    print_header "Benchmark Suite Completed!"
    echo "Phases run: $phases_run"
    echo "Phases succeeded: $phases_succeeded"
    echo ""
    echo "Results saved to: $SUITE_DIR"
    echo ""
    echo "Key files:"
    echo "  - unified_benchmark_report.json    # Complete machine-readable results"
    echo "  - comprehensive_benchmark_report.md # Complete human-readable report"
    echo "  - academic_summary.tex             # LaTeX tables for papers"
    echo ""
    echo "Individual benchmark results:"
    [ -d "$SUITE_DIR/accuracy_results" ] && echo "  - accuracy_results/                 # Accuracy benchmark details"
    [ -d "$SUITE_DIR/performance_results" ] && echo "  - performance_results/              # Performance benchmark details"
    [ -d "$SUITE_DIR/comparison_results" ] && echo "  - comparison_results/               # Baseline comparison details"
    [ -d "$SUITE_DIR/advanced_analysis" ] && echo "  - advanced_analysis/                # Statistical analysis and plots"
    
    if [ $phases_succeeded -eq $phases_run ]; then
        print_status "All benchmark phases completed successfully! 🎉"
    else
        print_warning "Some benchmark phases failed. Check individual results for details."
    fi
}

# Handle command line arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h         Show this help message"
        echo "  --with-analysis    Enable advanced statistical analysis"
        echo ""
        echo "Environment variables:"
        echo "  WITH_ANALYSIS      Enable advanced analysis (true/false, default: false)"
        echo ""
        echo "This script runs the complete Sakurs benchmark suite:"
        echo "1. Accuracy benchmarks on UD corpora"
        echo "2. Performance benchmarks on Wikipedia data"
        echo "3. Baseline comparisons with established tools"
        echo "4. Unified reporting and analysis"
        exit 0
        ;;
    --with-analysis)
        WITH_ANALYSIS=true
        main
        ;;
    *)
        main
        ;;
esac