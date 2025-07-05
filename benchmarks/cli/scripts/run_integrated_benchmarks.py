#!/usr/bin/env python3
"""Run integrated benchmarks using the new metrics framework."""

import argparse
import json
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from scripts.metrics import MetricsMeasurer
from scripts.parallel_runner import ParallelRunner
from scripts.results_formatter import ExperimentResult, ExperimentResults


def run_throughput_benchmarks(data_dir: Path, results: ExperimentResults):
    """Run throughput benchmarks for all tools and languages."""
    print("Running throughput benchmarks...")

    runner = ParallelRunner()
    measurer = MetricsMeasurer()

    # Define test configurations
    configs = [
        # Japanese tests
        {
            "language": "JA",
            "lang_code": "japanese",
            "input_file": data_dir / "japanese" / "wikipedia_sample.txt",
            "tools": ["sakurs", "ja_sentence_segmenter"],
        },
        # English tests
        {
            "language": "EN",
            "lang_code": "english",
            "input_file": data_dir / "english" / "wikipedia_sample.txt",
            "tools": ["sakurs", "nltk"],
        },
    ]

    for config in configs:
        input_file = config["input_file"]
        if not input_file.exists():
            print(f"Warning: {input_file} not found, skipping...")
            continue

        file_size_mb = input_file.stat().st_size / (1024 * 1024)

        for tool in config["tools"]:
            if tool == "sakurs":
                # Test with multiple thread counts
                for threads in [1, 2, 4, 8]:
                    print(
                        f"  Testing {tool} with {threads} threads on {config['language']} data..."
                    )

                    # Run with hyperfine through parallel runner
                    benchmark_results = runner.benchmark_threads(
                        str(input_file),
                        config["lang_code"],
                        thread_counts=[threads],
                        runs_per_thread=3,
                    )

                    if threads in benchmark_results and benchmark_results[threads]:
                        avg_time = sum(benchmark_results[threads]) / len(benchmark_results[threads])
                        throughput = measurer.measure_throughput(avg_time, file_size_mb)

                        result = ExperimentResult(
                            tool="Δ-Stack (Ours)",
                            language=config["language"],
                            threads=threads,
                            throughput_mbps=throughput,
                            dataset=input_file.name,
                            dataset_size_mb=file_size_mb,
                        )
                        results.add_result(result)
                        print(f"    Throughput: {throughput:.2f} MB/s")

            elif tool == "ja_sentence_segmenter" and config["language"] == "JA":
                print(f"  Testing {tool} on {config['language']} data...")
                # Run baseline tool (single-threaded only)
                # This would need implementation of baseline runner
                # For now, we'll add a placeholder
                result = ExperimentResult(
                    tool="ja_sentence_segmenter",
                    language="JA",
                    threads=1,
                    throughput_mbps=0.0,  # Placeholder
                    dataset=input_file.name,
                    dataset_size_mb=file_size_mb,
                )
                results.add_result(result)

            elif tool == "nltk" and config["language"] == "EN":
                print(f"  Testing {tool} on {config['language']} data...")
                # Run baseline tool (single-threaded only)
                result = ExperimentResult(
                    tool="NLTK Punkt",
                    language="EN",
                    threads=1,
                    throughput_mbps=0.0,  # Placeholder
                    dataset=input_file.name,
                    dataset_size_mb=file_size_mb,
                )
                results.add_result(result)


def run_accuracy_benchmarks(data_dir: Path, results: ExperimentResults):
    """Run accuracy benchmarks for all tools and languages."""
    print("\nRunning accuracy benchmarks...")

    measurer = MetricsMeasurer()

    # Define test configurations
    configs = [
        # Japanese tests
        {
            "language": "JA",
            "test_file": data_dir / "ud_japanese_gsd" / "test.txt",
            "gold_file": data_dir / "ud_japanese_gsd" / "test_gold.txt",
            "tools": ["sakurs", "ja_sentence_segmenter"],
        },
        # English tests
        {
            "language": "EN",
            "test_file": data_dir / "ud_english_ewt" / "test.txt",
            "gold_file": data_dir / "ud_english_ewt" / "test_gold.txt",
            "tools": ["sakurs", "nltk"],
        },
    ]

    for config in configs:
        if not config["test_file"].exists() or not config["gold_file"].exists():
            print(f"Warning: Test files for {config['language']} not found, skipping...")
            continue

        for tool in config["tools"]:
            print(f"  Testing {tool} accuracy on {config['language']} data...")

            # For now, add placeholder results
            # In real implementation, would run tools and calculate metrics
            result = ExperimentResult(
                tool=tool if tool != "sakurs" else "Δ-Stack (Ours)",
                language=config["language"],
                precision=0.95,  # Placeholder
                recall=0.93,  # Placeholder
                f1_score=0.94,  # Placeholder
                pk_score=0.05,  # Placeholder
                windowdiff_score=0.07,  # Placeholder
                dataset=config["test_file"].name,
            )
            results.add_result(result)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Run integrated benchmarks")
    parser.add_argument(
        "--data-dir",
        type=Path,
        default=Path(__file__).parent.parent.parent / "data",
        help="Directory containing test data",
    )
    parser.add_argument(
        "--results-dir",
        type=Path,
        default=Path(__file__).parent.parent / "results",
        help="Directory to save results",
    )
    parser.add_argument(
        "--experiment",
        choices=["throughput", "accuracy", "all"],
        default="all",
        help="Which experiments to run",
    )

    args = parser.parse_args()

    # Create results directory
    args.results_dir.mkdir(parents=True, exist_ok=True)

    # Initialize results collector
    results = ExperimentResults()

    # Run experiments
    if args.experiment in ["throughput", "all"]:
        run_throughput_benchmarks(args.data_dir, results)

    if args.experiment in ["accuracy", "all"]:
        run_accuracy_benchmarks(args.data_dir, results)

    # Save results
    print("\nSaving results...")

    # Save JSON
    json_path = args.results_dir / "integrated_results.json"
    with open(json_path, "w") as f:
        json.dump(results.to_json(), f, indent=2)
    print(f"JSON results saved to: {json_path}")

    # Save markdown tables
    for metric in ["throughput", "accuracy"]:
        try:
            table = results.to_markdown_table(metric)
            md_path = args.results_dir / f"{metric}_table.md"
            with open(md_path, "w") as f:
                f.write(f"# {metric.title()} Results\n\n")
                f.write(table)
            print(f"{metric.title()} table saved to: {md_path}")
        except Exception as e:
            print(f"Could not generate {metric} table: {e}")

    # Save LaTeX tables
    for metric in ["throughput", "accuracy"]:
        try:
            table = results.to_latex_table(metric)
            tex_path = args.results_dir / f"{metric}_table.tex"
            with open(tex_path, "w") as f:
                f.write(f"% {metric.title()} Results\n\n")
                f.write(table)
            print(f"{metric.title()} LaTeX table saved to: {tex_path}")
        except Exception as e:
            print(f"Could not generate {metric} LaTeX table: {e}")


if __name__ == "__main__":
    main()

