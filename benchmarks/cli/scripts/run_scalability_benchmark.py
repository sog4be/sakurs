#!/usr/bin/env python3
"""Run scalability benchmarks comparing sakurs and ja_sentence_segmenter.

This script measures processing time across different data sizes (1KiB to 10MiB)
to analyze performance characteristics and scaling behavior of both tools.
"""

import json
import logging
import platform
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Any

# Add parent directory to path for imports
sys.path.append(str(Path(__file__).parent.parent))

from scripts.metrics import MetricsMeasurer

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
)
logger = logging.getLogger(__name__)


class ScalabilityBenchmark:
    """Run scalability benchmarks for different data sizes."""

    def __init__(self, output_dir: Path):
        self.output_dir = output_dir
        self.output_dir.mkdir(parents=True, exist_ok=True)
        self.metrics = MetricsMeasurer()

    def run_single_benchmark(self, tool: str, input_file: Path, runs: int = 5) -> dict[str, Any]:
        """Run a single benchmark and collect timing data.

        Args:
            tool: Tool name ('sakurs' or 'ja_sentence_segmenter')
            input_file: Path to input file
            runs: Number of runs for averaging

        Returns:
            Dictionary with benchmark results
        """
        logger.info(f"Running {tool} on {input_file.name} ({runs} runs)...")

        file_size_bytes = input_file.stat().st_size
        timings = []

        for run in range(runs):
            start_time = time.perf_counter()

            if tool == "sakurs":
                # Use the built sakurs executable from the workspace root
                sakurs_path = (
                    Path(__file__).parent.parent.parent.parent / "target" / "release" / "sakurs"
                )
                if not sakurs_path.exists():
                    sakurs_path = (
                        Path(__file__).parent.parent.parent.parent / "target" / "debug" / "sakurs"
                    )
                cmd = [
                    str(sakurs_path),
                    "process",
                    "--input",
                    str(input_file),
                    "--output",
                    "/dev/stdout",
                    "--format",
                    "text",
                    "--language",
                    "japanese",
                    "--quiet",
                ]
                result = subprocess.run(cmd, capture_output=True, text=True)
            elif tool == "ja_sentence_segmenter":
                # Run ja_sentence_segmenter using the wrapper approach
                script = f"""
from ja_sentence_segmenter.common.pipeline import make_pipeline
from ja_sentence_segmenter.concatenate.simple_concatenator import concatenate_matching
from ja_sentence_segmenter.normalize.neologd_normalizer import normalize
from ja_sentence_segmenter.split.simple_splitter import split_newline, split_punctuation

split_punc2 = make_pipeline(normalize, split_newline, concatenate_matching, split_punctuation)

with open('{input_file}', 'r', encoding='utf-8') as f:
    text = f.read()
sentences = list(split_punc2(text))
for s in sentences:
    print(s)
"""
                cmd = [sys.executable, "-c", script]
                result = subprocess.run(cmd, capture_output=True, text=True)
            else:
                raise ValueError(f"Unknown tool: {tool}")

            end_time = time.perf_counter()

            if result.returncode != 0:
                logger.warning(f"Error in run {run + 1}: {result.stderr.strip()}")
                continue

            elapsed = end_time - start_time
            timings.append(elapsed)

            if run == 0:
                # Extract sentence count from output
                # Both tools output one sentence per line in text/lines format
                lines = result.stdout.strip().split("\n")
                sentence_count = len([line for line in lines if line.strip()])

        # Handle case where all runs failed
        if not timings:
            return {
                "tool": tool,
                "file": input_file.name,
                "error": "No successful runs",
                "runs": 0,
            }

        avg_time = sum(timings) / len(timings)
        min_time = min(timings)
        max_time = max(timings)

        # Calculate throughput
        file_size_mb = file_size_bytes / (1024 * 1024)
        throughput_mb_s = file_size_mb / avg_time if avg_time > 0 else 0

        return {
            "tool": tool,
            "file": input_file.name,
            "file_size_bytes": file_size_bytes,
            "file_size_mb": file_size_mb,
            "runs": len(timings),
            "avg_time_s": avg_time,
            "min_time_s": min_time,
            "max_time_s": max_time,
            "throughput_mb_s": throughput_mb_s,
            "sentences": sentence_count if "sentence_count" in locals() else None,
            "timings": timings,
        }

    def run_all_benchmarks(self) -> None:
        """Run benchmarks for all data sizes."""
        # Find sized samples
        base_dir = Path(__file__).parent.parent.parent
        sized_samples_dir = base_dir / "data" / "sized_samples"

        if not sized_samples_dir.exists():
            logger.error(f"Sized samples directory not found at {sized_samples_dir}")
            logger.error("Please run 'uv run python benchmarks/cli/scripts/create_sized_samples.py' first")
            sys.exit(1)

        # Get all sized sample files
        sample_files = sorted(sized_samples_dir.glob("wiki_ja_*.txt"))
        if not sample_files:
            logger.error(f"No sample files found in {sized_samples_dir}")
            sys.exit(1)

        # Tools to benchmark
        tools = ["sakurs", "ja_sentence_segmenter"]

        # Results storage
        all_results = []

        # System info
        system_info = {
            "timestamp": datetime.now().isoformat(),
            "platform": platform.platform(),
            "processor": platform.processor(),
            "python_version": sys.version,
        }

        logger.info("Running scalability benchmarks...")
        logger.info(f"System: {platform.platform()}")
        logger.info(f"Output directory: {self.output_dir}")

        # Run benchmarks
        for sample_file in sample_files:
            logger.info(f"\nBenchmarking {sample_file.name}:")

            for tool in tools:
                try:
                    result = self.run_single_benchmark(tool, sample_file, runs=10)
                    all_results.append(result)

                    if "error" not in result:
                        logger.info(
                            f"  {tool}: {result['avg_time_s']:.4f}s "
                            f"({result['throughput_mb_s']:.2f} MB/s)"
                        )
                except Exception as e:
                    logger.error(f"  Failed to benchmark {tool}: {e}")
                    all_results.append({
                        "tool": tool,
                        "file": sample_file.name,
                        "error": str(e),
                    })

        # Save results
        results_file = self.output_dir / "scalability_results.json"
        with open(results_file, "w") as f:
            json.dump({"system_info": system_info, "results": all_results}, f, indent=2)

        logger.info(f"\nResults saved to: {results_file}")

        # Generate report
        self.generate_report(all_results, system_info)

    def generate_report(
        self, results: list[dict[str, Any]], system_info: dict[str, Any]
    ) -> None:
        """Generate a markdown report from the results."""
        timestamp = datetime.now().strftime("%Y-%m-%d-%H:%M:%S")
        report_file = (
            self.output_dir.parent.parent.parent
            / "temp"
            / f"{timestamp}_scalability-benchmark-report.md"
        )
        report_file.parent.mkdir(parents=True, exist_ok=True)

        with open(report_file, "w") as f:
            f.write("# Scalability Benchmark Report\n\n")
            f.write(f"Generated: {system_info['timestamp']}\n")
            f.write(f"Platform: {system_info['platform']}\n\n")

            f.write("## Summary\n\n")
            f.write(
                "Comparison of processing time for sakurs and ja_sentence_segmenter across different data sizes.\n\n"
            )

            f.write("## Results Table\n\n")
            f.write("| Data Size | Tool | Processing Time (s) | Throughput (MB/s) | Sentences |\n")
            f.write("|-----------|------|---------------------|-------------------|----------|\n")

            # Group results by file
            by_file = {}
            for result in results:
                if "error" not in result:
                    file_name = result["file"]
                    if file_name not in by_file:
                        by_file[file_name] = []
                    by_file[file_name].append(result)

            # Generate table rows
            for file_name in sorted(by_file.keys()):
                file_results = by_file[file_name]
                size_label = file_name.replace("wiki_ja_", "").replace(".txt", "")

                for result in sorted(file_results, key=lambda x: x["tool"]):
                    f.write(
                        f"| {size_label} | {result['tool']} | "
                        f"{result['avg_time_s']:.4f} | "
                        f"{result['throughput_mb_s']:.2f} | "
                        f"{result['sentences'] or 'N/A'} |\n"
                    )

            f.write("\n## Performance Comparison\n\n")

            # Calculate speedup
            f.write("### Speedup (sakurs vs ja_sentence_segmenter)\n\n")
            f.write("| Data Size | sakurs Time (s) | ja_seg Time (s) | Speedup |\n")
            f.write("|-----------|-----------------|------------------|----------|\n")

            for file_name in sorted(by_file.keys()):
                file_results = by_file[file_name]
                size_label = file_name.replace("wiki_ja_", "").replace(".txt", "")

                sakurs_result = next((r for r in file_results if r["tool"] == "sakurs"), None)
                ja_seg_result = next(
                    (r for r in file_results if r["tool"] == "ja_sentence_segmenter"), None
                )

                if sakurs_result and ja_seg_result:
                    speedup = ja_seg_result["avg_time_s"] / sakurs_result["avg_time_s"]
                    f.write(
                        f"| {size_label} | {sakurs_result['avg_time_s']:.4f} | "
                        f"{ja_seg_result['avg_time_s']:.4f} | {speedup:.2f}x |\n"
                    )

            f.write("\n## Detailed Results\n\n")

            # Detailed results for each size
            for file_name in sorted(by_file.keys()):
                size_label = file_name.replace("wiki_ja_", "").replace(".txt", "")
                f.write(f"### {size_label}\n\n")

                file_results = by_file[file_name]
                for result in sorted(file_results, key=lambda x: x["tool"]):
                    f.write(f"**{result['tool']}**\n")
                    f.write(f"- Average time: {result['avg_time_s']:.4f}s\n")
                    f.write(f"- Min time: {result['min_time_s']:.4f}s\n")
                    f.write(f"- Max time: {result['max_time_s']:.4f}s\n")
                    f.write(f"- Throughput: {result['throughput_mb_s']:.2f} MB/s\n")
                    f.write(f"- Sentences: {result['sentences'] or 'N/A'}\n\n")

        logger.info(f"Report saved to: {report_file}")


def main() -> None:
    """Main entry point."""
    try:
        # Create output directory with timestamp
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_dir = (
            Path(__file__).parent.parent / "results" / f"scalability_{timestamp}"
        )

        # Initialize and run benchmarks
        benchmark = ScalabilityBenchmark(output_dir)
        benchmark.run_all_benchmarks()
    except KeyboardInterrupt:
        logger.info("\nBenchmark interrupted by user")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Benchmark failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
