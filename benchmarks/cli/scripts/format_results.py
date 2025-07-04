#!/usr/bin/env python3
"""Format benchmark results for publication."""

import json
import logging
from pathlib import Path

import click
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)


def load_results(results_dir: Path) -> dict[str, list[dict]]:
    """Load all benchmark results from directory."""
    results = {"accuracy": [], "performance": [], "comparison": []}

    for json_file in results_dir.glob("*.json"):
        with open(json_file) as f:
            data = json.load(f)

        # Categorize by filename pattern
        if "accuracy" in json_file.name:
            results["accuracy"].append(data)
        elif "performance" in json_file.name:
            results["performance"].append(data)
        elif "comparison" in json_file.name:
            results["comparison"].append(data)

    return results


def format_accuracy_table(accuracy_results: list[dict]) -> pd.DataFrame:
    """Format accuracy results as DataFrame."""
    rows = []

    for result in accuracy_results:
        row = {
            "Dataset": result.get("dataset", "Unknown"),
            "Language": result.get("language", "Unknown"),
            "Precision": f"{result['metrics']['precision']:.4f}",
            "Recall": f"{result['metrics']['recall']:.4f}",
            "F1": f"{result['metrics']['f1']:.4f}",
            "Pk": f"{result['metrics']['pk']:.4f}",
            "WindowDiff": f"{result['metrics']['window_diff']:.4f}",
        }
        rows.append(row)

    return pd.DataFrame(rows)


def format_performance_table(performance_results: list[dict]) -> pd.DataFrame:
    """Format performance results as DataFrame."""
    rows = []

    for result in performance_results:
        # Extract Hyperfine results
        if "results" in result and result["results"]:
            hyperfine_data = result["results"][0]
            row = {
                "Dataset": result.get("dataset", "Unknown"),
                "Language": result.get("language", "Unknown"),
                "Mean Time (s)": f"{hyperfine_data['mean']:.3f}",
                "Std Dev (s)": f"{hyperfine_data['stddev']:.3f}",
                "Min (s)": f"{hyperfine_data['min']:.3f}",
                "Max (s)": f"{hyperfine_data['max']:.3f}",
                "Throughput (MB/s)": f"{result.get('throughput_mbps', 0):.2f}",
            }
            rows.append(row)

    return pd.DataFrame(rows)


def generate_latex_tables(results: dict[str, list[dict]], output_dir: Path):
    """Generate LaTeX tables for paper."""
    output_dir.mkdir(exist_ok=True)

    # Accuracy table
    if results["accuracy"]:
        df = format_accuracy_table(results["accuracy"])
        latex = df.to_latex(
            index=False, caption="Segmentation Accuracy Results", label="tab:accuracy"
        )

        with open(output_dir / "accuracy_table.tex", "w") as f:
            f.write(latex)

    # Performance table
    if results["performance"]:
        df = format_performance_table(results["performance"])
        latex = df.to_latex(
            index=False, caption="Performance Benchmark Results", label="tab:performance"
        )

        with open(output_dir / "performance_table.tex", "w") as f:
            f.write(latex)


def generate_plots(results: dict[str, list[dict]], output_dir: Path):
    """Generate plots for visualization."""
    output_dir.mkdir(exist_ok=True)

    # Set style
    sns.set_style("whitegrid")
    plt.rcParams["figure.figsize"] = (10, 6)

    # Accuracy comparison plot
    if results["accuracy"]:
        df = format_accuracy_table(results["accuracy"])

        # F1 score comparison
        plt.figure()
        df.plot(x="Dataset", y="F1", kind="bar", legend=False)
        plt.title("F1 Score by Dataset")
        plt.ylabel("F1 Score")
        plt.xticks(rotation=45)
        plt.tight_layout()
        plt.savefig(output_dir / "f1_scores.png", dpi=300)
        plt.close()

    # Performance comparison plot
    if results["performance"]:
        df = format_performance_table(results["performance"])

        # Throughput comparison
        plt.figure()
        df.plot(x="Dataset", y="Throughput (MB/s)", kind="bar", legend=False)
        plt.title("Throughput Performance")
        plt.ylabel("Throughput (MB/s)")
        plt.xticks(rotation=45)
        plt.tight_layout()
        plt.savefig(output_dir / "throughput.png", dpi=300)
        plt.close()


@click.command()
@click.option(
    "--results-dir",
    "-r",
    required=True,
    type=click.Path(exists=True),
    help="Directory containing benchmark results",
)
@click.option(
    "--output-dir", "-o", type=click.Path(), help="Output directory for formatted results"
)
@click.option(
    "--format",
    "output_format",
    type=click.Choice(["latex", "plots", "markdown", "all"]),
    default="all",
    help="Output format",
)
def main(results_dir, output_dir, output_format):
    """Format benchmark results for publication."""
    results_dir = Path(results_dir)
    output_dir = Path(output_dir) if output_dir else results_dir / "formatted"

    logger.info(f"Loading results from {results_dir}")
    results = load_results(results_dir)

    total = sum(len(v) for v in results.values())
    logger.info(f"Found {total} result files")

    if output_format in ["latex", "all"]:
        logger.info("Generating LaTeX tables...")
        generate_latex_tables(results, output_dir / "latex")

    if output_format in ["plots", "all"]:
        logger.info("Generating plots...")
        generate_plots(results, output_dir / "plots")

    if output_format in ["markdown", "all"]:
        logger.info("Generating markdown tables...")
        # TODO: Implement markdown generation

    logger.info(f"Results formatted in {output_dir}")


if __name__ == "__main__":
    main()
