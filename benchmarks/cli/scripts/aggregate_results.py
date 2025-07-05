"""Aggregate benchmark results and generate formatted tables."""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, List, Optional

from metrics import BenchmarkResult, MetricsMeasurer


class ResultsAggregator:
    """Aggregate and format benchmark results."""

    def __init__(self):
        self.measurer = MetricsMeasurer()
        self.results: List[BenchmarkResult] = []

    def load_results_from_directory(self, input_dir: Path) -> None:
        """Load all JSON result files from a directory.

        Args:
            input_dir: Directory containing result JSON files
        """
        json_files = list(input_dir.glob("*.json"))
        for json_file in json_files:
            if json_file.name == "metadata.json" or json_file.name == "aggregated_results.json":
                continue

            try:
                loaded = self.measurer.load_results(str(json_file))
                self.results.extend(loaded)
            except Exception as e:
                print(f"Warning: Failed to load {json_file}: {e}", file=sys.stderr)

    def aggregate_by_metric(self) -> Dict[str, Dict]:
        """Aggregate results by metric type.

        Returns:
            Dictionary organized by metric type (throughput, memory, accuracy)
        """
        aggregated = {
            "throughput": {},
            "memory": {},
            "accuracy": {},
        }

        for result in self.results:
            # Throughput results
            if result.throughput:
                key = (result.language, result.tool, result.num_threads)
                if key not in aggregated["throughput"]:
                    aggregated["throughput"][key] = []
                aggregated["throughput"][key].append(result.throughput.throughput_mbps)

            # Memory results
            if result.memory:
                key = (result.language, result.tool, result.num_threads)
                if key not in aggregated["memory"]:
                    aggregated["memory"][key] = []
                aggregated["memory"][key].append(result.memory.peak_memory_mb)

            # Accuracy results
            if result.accuracy:
                key = (result.language, result.tool)
                aggregated["accuracy"][key] = {
                    "precision": result.accuracy.precision,
                    "recall": result.accuracy.recall,
                    "f1": result.accuracy.f1_score,
                    "pk": result.accuracy.pk,
                    "window_diff": result.accuracy.window_diff,
                }

        # Average multiple runs
        for metric_type in ["throughput", "memory"]:
            for key, values in aggregated[metric_type].items():
                if values:
                    aggregated[metric_type][key] = sum(values) / len(values)

        return aggregated

    def generate_markdown_tables(self, aggregated: Dict[str, Dict]) -> str:
        """Generate markdown tables for the results.

        Args:
            aggregated: Aggregated results dictionary

        Returns:
            Markdown string with formatted tables
        """
        md_lines = ["# Δ-Stack Monoid — Experimental Results\n"]
        md_lines.append("## Results Tables\n")

        # Table 1: Throughput
        md_lines.append("### Table 1: Throughput on 500 MiB Wikipedia (MB/s)\n")
        md_lines.append("| Lang | Tool | 1 T | 2 T | 4 T | 8 T |")
        md_lines.append("| --- | --- | --- | --- | --- | --- |")

        # Japanese results
        ja_tools = ["ja_sentence_segmenter", "Δ-Stack (Ours)"]
        for i, tool in enumerate(ja_tools):
            line = "| JA " if i == 0 else "|  "

            tool_key = "ja_seg" if tool == "ja_sentence_segmenter" else "sakurs"
            line += f"| {tool} "

            for threads in [1, 2, 4, 8]:
                key = ("ja", tool_key, threads)
                if key in aggregated["throughput"]:
                    value = aggregated["throughput"][key]
                    line += f"| {value:.1f} "
                elif threads == 1 or tool_key == "sakurs":
                    line += "| ___ "
                else:
                    line += "| — "
            line += "|"
            md_lines.append(line)

        # English results
        en_tools = ["NLTK Punkt", "Δ-Stack (Ours)"]
        for i, tool in enumerate(en_tools):
            line = "| EN " if i == 0 else "|  "

            tool_key = "nltk" if tool == "NLTK Punkt" else "sakurs"
            line += f"| {tool} "

            for threads in [1, 2, 4, 8]:
                key = ("en", tool_key, threads)
                if key in aggregated["throughput"]:
                    value = aggregated["throughput"][key]
                    if tool_key == "sakurs":
                        line += f"| **{value:.1f}** "
                    else:
                        line += f"| {value:.1f} "
                elif threads == 1 or tool_key == "sakurs":
                    line += "| ___ "
                else:
                    line += "| — "
            line += "|"
            md_lines.append(line)

        # Table 2: Memory
        md_lines.append("\n### Table 2: Peak Resident Memory (MiB) on 500 MiB Wikipedia\n")
        md_lines.append("| Lang | Tool | 1 T | 8 T |")
        md_lines.append("| --- | --- | --- | --- |")

        # Japanese memory results
        for i, tool in enumerate(ja_tools):
            line = "| JA " if i == 0 else "|  "

            tool_key = "ja_seg" if tool == "ja_sentence_segmenter" else "sakurs"
            line += f"| {tool} "

            for threads in [1, 8]:
                key = ("ja", tool_key, threads)
                if key in aggregated["memory"]:
                    value = aggregated["memory"][key]
                    if tool_key == "sakurs":
                        line += f"| **{value:.0f}** "
                    else:
                        line += f"| {value:.0f} "
                elif threads == 1 or (threads == 8 and tool_key == "sakurs"):
                    line += "| ___ "
                else:
                    line += "| — "
            line += "|"
            md_lines.append(line)

        # English memory results
        for i, tool in enumerate(en_tools):
            line = "| EN " if i == 0 else "|  "

            tool_key = "nltk" if tool == "NLTK Punkt" else "sakurs"
            line += f"| {tool} "

            for threads in [1, 8]:
                key = ("en", tool_key, threads)
                if key in aggregated["memory"]:
                    value = aggregated["memory"][key]
                    if tool_key == "sakurs":
                        line += f"| **{value:.0f}** "
                    else:
                        line += f"| {value:.0f} "
                elif threads == 1 or (threads == 8 and tool_key == "sakurs"):
                    line += "| ___ "
                else:
                    line += "| — "
            line += "|"
            md_lines.append(line)

        # Table 3: Accuracy
        md_lines.append("\n### Table 3: Sentence Boundary Accuracy on Gold Corpora (%)\n")
        md_lines.append("| Lang | Tool | Precision | Recall | F1 | **Pk** | **WindowDiff** |")
        md_lines.append("| --- | --- | --- | --- | --- | --- | --- |")

        # Japanese accuracy results
        for i, tool in enumerate(ja_tools):
            line = "| JA " if i == 0 else "|  "

            tool_key = "ja_seg" if tool == "ja_sentence_segmenter" else "sakurs"
            line += f"| {tool} "

            key = ("ja", tool_key)
            if key in aggregated["accuracy"]:
                acc = aggregated["accuracy"][key]
                if tool_key == "sakurs":
                    line += f"| **{acc['precision'] * 100:.1f}** | **{acc['recall'] * 100:.1f}** | **{acc['f1'] * 100:.1f}** "
                else:
                    line += f"| {acc['precision'] * 100:.1f} | {acc['recall'] * 100:.1f} | {acc['f1'] * 100:.1f} "

                if acc["pk"] is not None:
                    line += f"| {acc['pk']:.3f} "
                else:
                    line += "| ___ "

                if acc["window_diff"] is not None:
                    line += f"| {acc['window_diff']:.3f} |"
                else:
                    line += "| ___ |"
            else:
                line += "| ___ | ___ | ___ | ___ | ___ |"
            md_lines.append(line)

        # English accuracy results
        for i, tool in enumerate(en_tools):
            line = "| EN " if i == 0 else "|  "

            tool_key = "nltk" if tool == "NLTK Punkt" else "sakurs"
            line += f"| {tool} "

            key = ("en", tool_key)
            if key in aggregated["accuracy"]:
                acc = aggregated["accuracy"][key]
                if tool_key == "sakurs":
                    line += f"| **{acc['precision'] * 100:.1f}** | **{acc['recall'] * 100:.1f}** | **{acc['f1'] * 100:.1f}** "
                else:
                    line += f"| {acc['precision'] * 100:.1f} | {acc['recall'] * 100:.1f} | {acc['f1'] * 100:.1f} "

                if acc["pk"] is not None:
                    line += f"| {acc['pk']:.3f} "
                else:
                    line += "| ___ "

                if acc["window_diff"] is not None:
                    line += f"| {acc['window_diff']:.3f} |"
                else:
                    line += "| ___ |"
            else:
                line += "| ___ | ___ | ___ | ___ | ___ |"
            md_lines.append(line)

        return "\n".join(md_lines)

    def save_aggregated_json(self, aggregated: Dict[str, Dict], output_file: str) -> None:
        """Save aggregated results to JSON file.

        Args:
            aggregated: Aggregated results dictionary
            output_file: Output JSON file path
        """
        # Convert to serializable format
        serializable = {}
        for metric_type, data in aggregated.items():
            serializable[metric_type] = {}
            for key, value in data.items():
                if isinstance(key, tuple):
                    key_str = "_".join(str(k) for k in key)
                else:
                    key_str = str(key)
                serializable[metric_type][key_str] = value

        with open(output_file, "w") as f:
            json.dump(serializable, f, indent=2)


def main():
    """Main function."""
    parser = argparse.ArgumentParser(
        description="Aggregate benchmark results and generate formatted tables"
    )
    parser.add_argument(
        "--input-dir",
        type=Path,
        required=True,
        help="Directory containing result JSON files",
    )
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Output file for aggregated JSON results",
    )
    parser.add_argument(
        "--template",
        type=Path,
        help="Output file for markdown table template",
    )

    args = parser.parse_args()

    # Check input directory
    if not args.input_dir.exists():
        print(f"Error: Input directory not found: {args.input_dir}", file=sys.stderr)
        sys.exit(1)

    # Create aggregator and load results
    aggregator = ResultsAggregator()
    aggregator.load_results_from_directory(args.input_dir)

    if not aggregator.results:
        print("Warning: No results found to aggregate", file=sys.stderr)
        sys.exit(0)

    # Aggregate results
    aggregated = aggregator.aggregate_by_metric()

    # Save aggregated JSON
    aggregator.save_aggregated_json(aggregated, str(args.output))
    print(f"Aggregated results saved to: {args.output}")

    # Generate and save markdown tables if requested
    if args.template:
        markdown_tables = aggregator.generate_markdown_tables(aggregated)
        with open(args.template, "w") as f:
            f.write(markdown_tables)
        print(f"Markdown tables saved to: {args.template}")


if __name__ == "__main__":
    main()
