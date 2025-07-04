#!/usr/bin/env python3
"""Generate comprehensive comparison report."""

import logging
import sys
from datetime import datetime
from pathlib import Path

import click

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from scripts.format_results import format_accuracy_table, format_performance_table, load_results

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)


def generate_html_report(results_dir: Path, output_path: Path):
    """Generate HTML report from benchmark results."""
    results = load_results(results_dir)

    html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Sakurs CLI Benchmark Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        h2 {{ color: #666; margin-top: 30px; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .timestamp {{ color: #999; font-size: 0.9em; }}
        .note {{ background-color: #fffbea; padding: 10px; border-left: 3px solid #ffc107; margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>Sakurs CLI Benchmark Report</h1>
    <p class="timestamp">Generated: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}</p>

    <h2>Accuracy Results</h2>
    """

    if results["accuracy"]:
        df = format_accuracy_table(results["accuracy"])
        html_content += df.to_html(index=False)
    else:
        html_content += "<p>No accuracy results found.</p>"

    html_content += """
    <h2>Performance Results</h2>
    """

    if results["performance"]:
        df = format_performance_table(results["performance"])
        html_content += df.to_html(index=False)
    else:
        html_content += """
        <div class="note">
            Performance benchmarks will be available in Phase 3.
        </div>
        """

    html_content += """
    <h2>Comparison with Baselines</h2>
    <div class="note">
        Baseline comparisons will be added in Phase 2 (Japanese) and Phase 4 (full comparison).
    </div>

    <h2>Notes</h2>
    <ul>
        <li>All benchmarks run with Hyperfine for consistent measurement</li>
        <li>Accuracy metrics: Precision, Recall, F1, Pk, WindowDiff</li>
        <li>Performance metrics: Throughput (MB/s), Latency</li>
        <li>Results are reproducible with fixed random seeds</li>
    </ul>
</body>
</html>
    """

    with open(output_path, "w") as f:
        f.write(html_content)

    logger.info(f"HTML report saved to {output_path}")


@click.command()
@click.option(
    "--results-dir",
    "-r",
    type=click.Path(exists=True),
    help="Directory containing benchmark results",
)
@click.option("--output", "-o", type=click.Path(), help="Output path for HTML report")
def main(results_dir, output):
    """Generate comprehensive benchmark report."""
    # Default paths
    if not results_dir:
        results_dir = Path(__file__).parent.parent / "results"

    if not output:
        timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
        output = results_dir / f"benchmark_report_{timestamp}.html"

    results_dir = Path(results_dir)
    output = Path(output)

    if not results_dir.exists():
        logger.error(f"Results directory not found: {results_dir}")
        sys.exit(1)

    generate_html_report(results_dir, output)

    # Also generate formatted results
    logger.info("Generating additional formatted outputs...")

    # Call format_results with appropriate arguments
    import subprocess

    subprocess.run(
        [
            sys.executable,
            str(Path(__file__).parent.parent / "scripts" / "format_results.py"),
            "--results-dir",
            str(results_dir),
            "--format",
            "all",
        ]
    )


if __name__ == "__main__":
    main()
