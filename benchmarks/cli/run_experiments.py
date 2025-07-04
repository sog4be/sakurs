#!/usr/bin/env python3
"""Master experiment runner using the new metrics framework."""

import argparse
import json
import os
import subprocess
import sys
import tempfile
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from cli.scripts.metrics import BenchmarkMetrics, MetricsMeasurer
from cli.scripts.parallel_runner import ParallelRunner
from cli.scripts.results_formatter import ExperimentResult, ExperimentResults


class ExperimentRunner:
    """Manages and runs benchmark experiments."""

    def __init__(self, results_dir: Path):
        """Initialize the experiment runner.

        Args:
            results_dir: Directory to store results
        """
        self.results_dir = results_dir
        self.results_dir.mkdir(parents=True, exist_ok=True)
        self.measurer = MetricsMeasurer()
        self.parallel_runner = ParallelRunner()
        self.results = ExperimentResults()

    def run_throughput_benchmark(
        self,
        tool: str,
        language: str,
        input_file: Path,
        thread_counts: List[int] = [1, 2, 4, 8],
    ) -> Dict[int, float]:
        """Run throughput benchmark for a tool.

        Args:
            tool: Tool name (sakurs, nltk, ja_sentence_segmenter)
            language: Language code (EN, JA)
            input_file: Input file path
            thread_counts: List of thread counts to test

        Returns:
            Dictionary mapping thread count to throughput
        """
        file_size_mb = input_file.stat().st_size / (1024 * 1024)
        results = {}

        if tool == "sakurs":
            # Use parallel runner for sakurs
            benchmark_results = self.parallel_runner.benchmark_threads(
                str(input_file),
                language.lower() if language.lower() in ["english", "japanese"] else "english",
                thread_counts=thread_counts,
                runs_per_thread=3,
            )
            
            for threads, times in benchmark_results.items():
                if times:
                    avg_time = sum(times) / len(times)
                    throughput = self.measurer.measure_throughput(avg_time, file_size_mb)
                    results[threads] = throughput
                    
                    # Add to results collection
                    result = ExperimentResult(
                        tool="Δ-Stack (Ours)",
                        language=language,
                        threads=threads,
                        throughput_mbps=throughput,
                        dataset=str(input_file.name),
                        dataset_size_mb=file_size_mb,
                    )
                    self.results.add_result(result)
        
        elif tool == "nltk" and language == "EN":
            # Run NLTK benchmark (single-threaded only)
            duration, _ = self._run_nltk_benchmark(input_file)
            throughput = self.measurer.measure_throughput(duration, file_size_mb)
            results[1] = throughput
            
            result = ExperimentResult(
                tool="NLTK Punkt",
                language="EN",
                threads=1,
                throughput_mbps=throughput,
                dataset=str(input_file.name),
                dataset_size_mb=file_size_mb,
            )
            self.results.add_result(result)
        
        elif tool == "ja_sentence_segmenter" and language == "JA":
            # Run ja_sentence_segmenter benchmark (single-threaded only)
            duration, _ = self._run_jaseg_benchmark(input_file)
            throughput = self.measurer.measure_throughput(duration, file_size_mb)
            results[1] = throughput
            
            result = ExperimentResult(
                tool="ja_sentence_segmenter",
                language="JA",
                threads=1,
                throughput_mbps=throughput,
                dataset=str(input_file.name),
                dataset_size_mb=file_size_mb,
            )
            self.results.add_result(result)
        
        return results

    def run_accuracy_benchmark(
        self,
        tool: str,
        language: str,
        test_file: Path,
        gold_file: Path,
    ) -> BenchmarkMetrics:
        """Run accuracy benchmark for a tool.

        Args:
            tool: Tool name
            language: Language code
            test_file: Test input file
            gold_file: Gold standard file

        Returns:
            Benchmark metrics with accuracy scores
        """
        metrics = BenchmarkMetrics()
        
        # Run the tool and get predictions
        if tool == "sakurs":
            predictions = self._run_sakurs_for_accuracy(test_file, language)
        elif tool == "nltk" and language == "EN":
            predictions = self._run_nltk_for_accuracy(test_file)
        elif tool == "ja_sentence_segmenter" and language == "JA":
            predictions = self._run_jaseg_for_accuracy(test_file)
        else:
            return metrics
        
        # Load gold standard
        gold_sentences = self._load_gold_sentences(gold_file)
        
        # Calculate metrics
        pred_boundaries = self._get_sentence_boundaries(predictions)
        gold_boundaries = self._get_sentence_boundaries(gold_sentences)
        
        accuracy_metrics = self.measurer.calculate_precision_recall_f1(
            pred_boundaries, gold_boundaries
        )
        
        metrics.precision = accuracy_metrics["precision"]
        metrics.recall = accuracy_metrics["recall"]
        metrics.f1_score = accuracy_metrics["f1"]
        
        # Calculate Pk and WindowDiff
        pred_segmentation = self._create_segmentation_string(predictions)
        gold_segmentation = self._create_segmentation_string(gold_sentences)
        
        if len(pred_segmentation) == len(gold_segmentation):
            pk_wd_metrics = self.measurer.calculate_pk_windowdiff(
                gold_segmentation, pred_segmentation
            )
            metrics.pk_score = pk_wd_metrics["pk"]
            metrics.windowdiff_score = pk_wd_metrics["windowdiff"]
        
        # Add to results collection
        result = ExperimentResult(
            tool=self._format_tool_name(tool),
            language=language,
            precision=metrics.precision,
            recall=metrics.recall,
            f1_score=metrics.f1_score,
            pk_score=metrics.pk_score,
            windowdiff_score=metrics.windowdiff_score,
            dataset=str(test_file.name),
        )
        self.results.add_result(result)
        
        return metrics

    def _run_nltk_benchmark(self, input_file: Path) -> tuple:
        """Run NLTK Punkt benchmark."""
        cmd = [
            "uv", "run", "python", "-c",
            f"""
import time
import nltk
nltk.download('punkt', quiet=True)
from nltk.tokenize import sent_tokenize

start = time.perf_counter()
with open('{input_file}', 'r', encoding='utf-8') as f:
    text = f.read()
sentences = sent_tokenize(text)
duration = time.perf_counter() - start
print(f"{{duration}}")
"""
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(Path(__file__).parent.parent))
        if result.returncode == 0:
            duration = float(result.stdout.strip())
            return duration, None
        else:
            raise RuntimeError(f"NLTK benchmark failed: {result.stderr}")

    def _run_jaseg_benchmark(self, input_file: Path) -> tuple:
        """Run ja_sentence_segmenter benchmark."""
        cmd = [
            "uv", "run", "python", "-c",
            f"""
import time
import ja_sentence_segmenter

start = time.perf_counter()
with open('{input_file}', 'r', encoding='utf-8') as f:
    text = f.read()
sentences = ja_sentence_segmenter.split_sentences(text)
duration = time.perf_counter() - start
print(f"{{duration}}")
"""
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(Path(__file__).parent.parent))
        if result.returncode == 0:
            duration = float(result.stdout.strip())
            return duration, None
        else:
            raise RuntimeError(f"ja_sentence_segmenter benchmark failed: {result.stderr}")

    def _run_sakurs_for_accuracy(self, test_file: Path, language: str) -> List[str]:
        """Run sakurs and get sentence predictions."""
        with tempfile.NamedTemporaryFile(mode='w', delete=False) as output_file:
            output_path = output_file.name
        
        try:
            lang_code = "japanese" if language == "JA" else "english"
            result = self.parallel_runner.run_with_threads(
                str(test_file),
                output_path,
                lang_code,
                num_threads=1,
            )
            
            if result.returncode == 0:
                with open(output_path, 'r', encoding='utf-8') as f:
                    return f.read().strip().split('\n')
            else:
                raise RuntimeError(f"Sakurs failed: {result.stderr}")
        finally:
            if os.path.exists(output_path):
                os.unlink(output_path)

    def _run_nltk_for_accuracy(self, test_file: Path) -> List[str]:
        """Run NLTK and get sentence predictions."""
        cmd = [
            "uv", "run", "python", "-c",
            f"""
import nltk
nltk.download('punkt', quiet=True)
from nltk.tokenize import sent_tokenize

with open('{test_file}', 'r', encoding='utf-8') as f:
    text = f.read()
sentences = sent_tokenize(text)
for s in sentences:
    print(s)
"""
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(Path(__file__).parent.parent))
        if result.returncode == 0:
            return result.stdout.strip().split('\n')
        else:
            raise RuntimeError(f"NLTK failed: {result.stderr}")

    def _run_jaseg_for_accuracy(self, test_file: Path) -> List[str]:
        """Run ja_sentence_segmenter and get sentence predictions."""
        cmd = [
            "uv", "run", "python", "-c",
            f"""
import ja_sentence_segmenter

with open('{test_file}', 'r', encoding='utf-8') as f:
    text = f.read()
sentences = ja_sentence_segmenter.split_sentences(text)
for s in sentences:
    print(s)
"""
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(Path(__file__).parent.parent))
        if result.returncode == 0:
            return result.stdout.strip().split('\n')
        else:
            raise RuntimeError(f"ja_sentence_segmenter failed: {result.stderr}")

    def _load_gold_sentences(self, gold_file: Path) -> List[str]:
        """Load gold standard sentences."""
        with open(gold_file, 'r', encoding='utf-8') as f:
            return f.read().strip().split('\n')

    def _get_sentence_boundaries(self, sentences: List[str]) -> List[int]:
        """Get sentence boundary positions from sentences."""
        boundaries = []
        pos = 0
        for sentence in sentences:
            if sentence:  # Skip empty lines
                pos += len(sentence)
                boundaries.append(pos)
        return boundaries

    def _create_segmentation_string(self, sentences: List[str]) -> str:
        """Create binary segmentation string for Pk/WindowDiff."""
        if not sentences:
            return ""
        
        result = []
        for i, sentence in enumerate(sentences):
            if sentence:  # Skip empty lines
                # Add 0s for each character
                result.extend(['0'] * len(sentence))
                # Add boundary marker at end (except for last sentence)
                if i < len(sentences) - 1:
                    result[-1] = '1'
        
        return ''.join(result)

    def _format_tool_name(self, tool: str) -> str:
        """Format tool name for display."""
        if tool == "sakurs":
            return "Δ-Stack (Ours)"
        elif tool == "nltk":
            return "NLTK Punkt"
        elif tool == "ja_sentence_segmenter":
            return "ja_sentence_segmenter"
        return tool

    def save_results(self, filename: str = None):
        """Save results to file."""
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"experiment_results_{timestamp}.json"
        
        output_path = self.results_dir / filename
        with open(output_path, 'w') as f:
            json.dump(self.results.to_json(), f, indent=2)
        
        print(f"Results saved to: {output_path}")
        
        # Also generate markdown tables
        for metric in ["throughput", "memory", "accuracy"]:
            try:
                table = self.results.to_markdown_table(metric)
                table_path = self.results_dir / f"{metric}_table_{timestamp}.md"
                with open(table_path, 'w') as f:
                    f.write(f"# {metric.title()} Results\n\n")
                    f.write(table)
                print(f"{metric.title()} table saved to: {table_path}")
            except Exception as e:
                print(f"Could not generate {metric} table: {e}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Run benchmark experiments")
    parser.add_argument(
        "--experiment",
        choices=["throughput", "accuracy", "all"],
        default="all",
        help="Type of experiment to run",
    )
    parser.add_argument(
        "--language",
        choices=["EN", "JA", "all"],
        default="all",
        help="Language to test",
    )
    parser.add_argument(
        "--threads",
        nargs="+",
        type=int,
        default=[1, 2, 4, 8],
        help="Thread counts for throughput tests",
    )
    parser.add_argument(
        "--results-dir",
        type=Path,
        default=Path(__file__).parent / "results",
        help="Directory to store results",
    )
    
    args = parser.parse_args()
    
    runner = ExperimentRunner(args.results_dir)
    
    # Example usage - you would replace with actual file paths
    print("Experiment runner ready. Please implement specific experiment logic.")
    print(f"Results will be saved to: {args.results_dir}")
    
    # Save empty results as example
    runner.save_results()


if __name__ == "__main__":
    main()