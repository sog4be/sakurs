#!/usr/bin/env python3
"""Analyze and format benchmark results for various output formats."""

import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

import click
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import seaborn as sns

# Import statistical analysis utilities
sys.path.insert(0, str(Path(__file__).parent))
from statistical_analysis import bootstrap_confidence_interval

# Set style for plots
plt.style.use("seaborn-v0_8-darkgrid")
sns.set_palette("husl")


def load_hyperfine_results(json_path: Path) -> dict[str, Any]:
    """Load Hyperfine benchmark results from JSON."""
    with open(json_path) as f:
        return json.load(f)


def load_accuracy_results(json_path: Path) -> dict[str, Any]:
    """Load accuracy evaluation results from JSON."""
    with open(json_path) as f:
        return json.load(f)


def format_time(seconds: float) -> str:
    """Format time in seconds to human-readable string."""
    if seconds < 0.001:
        return f"{seconds * 1000000:.0f}μs"
    elif seconds < 1:
        return f"{seconds * 1000:.1f}ms"
    else:
        return f"{seconds:.2f}s"


def generate_performance_plot(
    results: list[dict[str, Any]], output_path: Path, with_ci: bool = True
):
    """Generate performance comparison plot with confidence intervals."""
    # Extract data for plotting
    data = []
    for result in results:
        if "performance" in result:
            perf = result["performance"]

            # Check if we have timing data for bootstrap CI
            if with_ci and "times" in perf:
                ci = bootstrap_confidence_interval(perf["times"])
                error_lower = ci.estimate - ci.lower
                error_upper = ci.upper - ci.estimate
                yerr = [[error_lower], [error_upper]]
            else:
                # Fallback to standard deviation
                yerr = perf.get("stddev", 0)

            data.append(
                {
                    "Language": result["language"],
                    "Mean Time (s)": perf["mean_time"],
                    "Error": yerr,
                    "CI_text": f"95% CI [{ci.lower:.3f}, {ci.upper:.3f}]"
                    if with_ci and "times" in perf
                    else None,
                }
            )

    if not data:
        return

    df = pd.DataFrame(data)

    # Create bar plot with error bars
    fig, ax = plt.subplots(figsize=(10, 6))
    x = range(len(df))

    # Handle asymmetric error bars for CI
    yerr_values = []
    for err in df["Error"]:
        if isinstance(err, list):
            yerr_values.append(err)
        else:
            yerr_values.append([[err], [err]])

    yerr_array = np.array(yerr_values).T

    bars = ax.bar(x, df["Mean Time (s)"], yerr=yerr_array, capsize=10, alpha=0.7, edgecolor="black")

    # Customize plot
    ax.set_xlabel("Language", fontsize=12)
    ax.set_ylabel("Processing Time (seconds)", fontsize=12)
    title = "Sakurs Performance Benchmark Results"
    if with_ci:
        title += " (with 95% Confidence Intervals)"
    ax.set_title(title, fontsize=14, fontweight="bold")
    ax.set_xticks(x)
    ax.set_xticklabels(df["Language"])

    # Add value labels on bars
    for i, (bar, row) in enumerate(zip(bars, df.itertuples())):
        height = bar.get_height()
        # Position text above error bar
        y_pos = height + yerr_array[1][i] if isinstance(row.Error, list) else height + row.Error
        label = f"{format_time(row._2)}"  # Mean Time
        if row.CI_text:
            label += f"\n{row.CI_text}"
        ax.text(
            bar.get_x() + bar.get_width() / 2.0, y_pos, label, ha="center", va="bottom", fontsize=9
        )

    plt.tight_layout()
    plt.savefig(output_path, dpi=300, bbox_inches="tight")
    plt.close()


def generate_accuracy_plot(results: list[dict[str, Any]], output_path: Path, with_ci: bool = True):
    """Generate accuracy metrics comparison plot with confidence intervals."""
    # Extract accuracy data
    metrics = ["precision", "recall", "f1"]
    data = {metric: {"values": [], "ci_lower": [], "ci_upper": []} for metric in metrics}
    languages = []

    for result in results:
        if "accuracy" in result:
            languages.append(result["language"])
            acc = result["accuracy"]

            # Get sample size if available
            n_samples = result.get("n_samples", result.get("n_boundaries", 1000))

            for metric in metrics:
                value = acc.get(metric, 0)
                data[metric]["values"].append(value)

                # Calculate confidence intervals
                if with_ci:
                    from statistical_analysis import proportion_confidence_interval

                    successes = int(value * n_samples)
                    ci = proportion_confidence_interval(successes, n_samples)
                    data[metric]["ci_lower"].append(value - ci.lower)
                    data[metric]["ci_upper"].append(ci.upper - value)
                else:
                    # No CI
                    data[metric]["ci_lower"].append(0)
                    data[metric]["ci_upper"].append(0)

    if not languages:
        return

    # Create grouped bar plot
    fig, ax = plt.subplots(figsize=(10, 6))
    x = np.arange(len(languages))
    width = 0.25

    # Plot bars for each metric with error bars
    for i, (metric, metric_data) in enumerate(data.items()):
        offset = (i - 1) * width
        yerr = [metric_data["ci_lower"], metric_data["ci_upper"]] if with_ci else None
        bars = ax.bar(
            x + offset,
            metric_data["values"],
            width,
            yerr=yerr,
            capsize=5,
            label=metric.capitalize(),
            alpha=0.8,
        )

        # Add value labels
        for bar, value in zip(bars, metric_data["values"]):
            height = bar.get_height()
            ax.text(
                bar.get_x() + bar.get_width() / 2.0,
                height + 0.01,
                f"{value:.3f}",
                ha="center",
                va="bottom",
                fontsize=9,
            )

    # Customize plot
    ax.set_xlabel("Language", fontsize=12)
    ax.set_ylabel("Score", fontsize=12)
    ax.set_title("Sakurs Accuracy Benchmark Results", fontsize=14, fontweight="bold")
    ax.set_xticks(x)
    ax.set_xticklabels(languages)
    ax.set_ylim(0, 1.1)
    ax.legend()
    ax.grid(axis="y", alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_path, dpi=300, bbox_inches="tight")
    plt.close()


def format_latex_table(results: list[dict[str, Any]]) -> str:
    """Format results as LaTeX table."""
    latex = []
    latex.append("\\begin{table}[htbp]")
    latex.append("\\centering")
    latex.append("\\caption{Sakurs Benchmark Results}")
    latex.append("\\label{tab:sakurs-benchmarks}")
    latex.append("\\begin{tabular}{lcccccc}")
    latex.append("\\toprule")
    latex.append("Language & Dataset & F1 & Precision & Recall & Time (s) & Throughput \\\\")
    latex.append("\\midrule")

    for result in results:
        lang = result["language"]
        dataset = result.get("dataset", "N/A")

        if "accuracy" in result:
            f1 = f"{result['accuracy']['f1']:.4f}"
            prec = f"{result['accuracy']['precision']:.4f}"
            rec = f"{result['accuracy']['recall']:.4f}"
        else:
            f1 = prec = rec = "N/A"

        if "performance" in result:
            time = f"{result['performance']['mean_time']:.3f} $\\pm$ {result['performance']['stddev']:.3f}"
            if "chars_per_second" in result["performance"]:
                throughput = f"{result['performance']['chars_per_second']:,} chars/s"
            else:
                throughput = "N/A"
        else:
            time = throughput = "N/A"

        latex.append(f"{lang} & {dataset} & {f1} & {prec} & {rec} & {time} & {throughput} \\\\")

    latex.append("\\bottomrule")
    latex.append("\\end{tabular}")
    latex.append("\\end{table}")

    return "\n".join(latex)


def format_markdown_report(results: list[dict[str, Any]]) -> str:
    """Format results as Markdown report."""
    md = []
    md.append("# Sakurs Benchmark Results")
    md.append("")
    md.append(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    md.append("")

    # Summary table
    md.append("## Summary")
    md.append("")
    md.append("| Language | Dataset | F1 Score | Time (s) | Status |")
    md.append("|----------|---------|----------|----------|---------|")

    for result in results:
        lang = result["language"]
        dataset = result.get("dataset", "N/A")

        if "accuracy" in result:
            f1 = f"{result['accuracy']['f1']:.4f}"
            status = "✅ Pass" if result["accuracy"]["f1"] > 0.85 else "⚠️ Low"
        else:
            f1 = "N/A"
            status = "❌ Failed"

        if "performance" in result:
            time = f"{result['performance']['mean_time']:.3f}"
        else:
            time = "N/A"

        md.append(f"| {lang} | {dataset} | {f1} | {time} | {status} |")

    md.append("")
    md.append("## Detailed Results")

    for result in results:
        md.append("")
        md.append(f"### {result['language']}")
        md.append("")

        if "dataset" in result:
            md.append(f"**Dataset**: {result['dataset']}")
            md.append("")

        if "accuracy" in result:
            md.append("**Accuracy Metrics:**")
            md.append(f"- F1 Score: {result['accuracy']['f1']:.4f}")
            md.append(f"- Precision: {result['accuracy']['precision']:.4f}")
            md.append(f"- Recall: {result['accuracy']['recall']:.4f}")
            md.append(f"- Sentences: {result['accuracy']['reference_sentences']:,}")
            md.append("")

        if "performance" in result:
            md.append("**Performance Metrics:**")
            md.append(f"- Mean Time: {result['performance']['mean_time']:.3f}s")
            md.append(f"- Std Dev: {result['performance']['stddev']:.3f}s")
            md.append(f"- Min Time: {result['performance']['min']:.3f}s")
            md.append(f"- Max Time: {result['performance']['max']:.3f}s")

            if "chars_per_second" in result["performance"]:
                md.append(f"- Throughput: {result['performance']['chars_per_second']:,} chars/s")

            md.append("")

    return "\n".join(md)


@click.command()
@click.option(
    "--input",
    "-i",
    "input_files",
    multiple=True,
    required=True,
    type=click.Path(exists=True),
    help="Input JSON result files",
)
@click.option(
    "--output-dir",
    "-o",
    type=click.Path(),
    default=".",
    help="Output directory for generated files",
)
@click.option(
    "--format",
    "-f",
    "output_formats",
    multiple=True,
    type=click.Choice(["latex", "markdown", "plots", "all"]),
    default=["all"],
    help="Output formats to generate",
)
def main(input_files, output_dir, output_formats):
    """Analyze and format benchmark results."""
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    # Load all results
    results = []
    for input_file in input_files:
        with open(input_file) as f:
            data = json.load(f)

            # Extract language from filename or data
            filename = Path(input_file).stem
            if "english" in filename.lower():
                data["language"] = "English"
            elif "japanese" in filename.lower():
                data["language"] = "Japanese"
            else:
                data["language"] = "Unknown"

            results.append(data)

    # Generate outputs based on requested formats
    if "all" in output_formats or "plots" in output_formats:
        print("Generating plots...")
        generate_performance_plot(results, output_path / "performance_comparison.png")
        generate_accuracy_plot(results, output_path / "accuracy_comparison.png")
        print(f"  ✓ Plots saved to {output_path}")

    if "all" in output_formats or "latex" in output_formats:
        print("Generating LaTeX table...")
        latex_content = format_latex_table(results)
        with open(output_path / "benchmark_results.tex", "w") as f:
            f.write(latex_content)
        print(f"  ✓ LaTeX table saved to {output_path / 'benchmark_results.tex'}")

    if "all" in output_formats or "markdown" in output_formats:
        print("Generating Markdown report...")
        md_content = format_markdown_report(results)
        with open(output_path / "benchmark_report.md", "w") as f:
            f.write(md_content)
        print(f"  ✓ Markdown report saved to {output_path / 'benchmark_report.md'}")

    # Also save combined JSON for future processing
    combined = {"timestamp": datetime.now().isoformat(), "results": results}
    with open(output_path / "combined_results.json", "w") as f:
        json.dump(combined, f, indent=2)

    print("\nAnalysis complete!")


if __name__ == "__main__":
    main()
