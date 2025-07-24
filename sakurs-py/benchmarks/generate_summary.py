#!/usr/bin/env python3
"""Generate markdown summary from pytest-benchmark JSON results."""

import json
import os
import sys
from typing import Any


def format_time(seconds: float) -> str:
    """Format time in appropriate units."""
    if seconds < 0.001:
        return f"{seconds * 1_000_000:.2f} μs"
    elif seconds < 1:
        return f"{seconds * 1_000:.2f} ms"
    else:
        return f"{seconds:.2f} s"


def calculate_ratio(base_time: float, compare_time: float) -> str:
    """Calculate performance ratio."""
    if compare_time == 0:
        return "N/A"
    ratio = base_time / compare_time
    if ratio >= 1:
        return f"{ratio:.2f}x faster"
    else:
        return f"{1 / ratio:.2f}x slower"


def extract_benchmark_data(
    benchmarks: list[dict[str, Any]],
) -> dict[str, dict[str, Any]]:
    """Extract and organize benchmark data by test name."""
    data = {}
    for benchmark in benchmarks:
        name = benchmark["name"]
        # Extract library name and test type from test name
        if "sakurs" in name:
            library = "sakurs"
        elif "pysbd" in name:
            library = "pysbd"
        elif "ja_segmenter" in name:
            library = "ja_segmenter"
        else:
            continue

        if "400" in name:
            test_type = "400_chars"
        elif "large" in name:
            test_type = "large"
        else:
            continue

        if "english" in name:
            language = "english"
        elif "japanese" in name:
            language = "japanese"
        else:
            continue

        key = f"{language}_{test_type}_{library}"
        data[key] = {
            "mean": benchmark["stats"]["mean"],
            "stddev": benchmark["stats"]["stddev"],
            "min": benchmark["stats"]["min"],
            "max": benchmark["stats"]["max"],
            "rounds": benchmark["stats"]["rounds"],
            "iterations": benchmark["stats"]["iterations"],
        }

        # Include extra_info if available (contains segmentation results)
        if "extra_info" in benchmark:
            data[key]["extra_info"] = benchmark["extra_info"]

    return data


def generate_performance_table(
    data: dict[str, dict[str, Any]], language: str, test_type: str
) -> str:
    """Generate performance comparison table."""
    sakurs_key = f"{language}_{test_type}_sakurs"

    if language == "english":
        other_key = f"{language}_{test_type}_pysbd"
        other_name = "PySBD"
    else:
        other_key = f"{language}_{test_type}_ja_segmenter"
        other_name = "ja_sentence_segmenter"

    if sakurs_key not in data or other_key not in data:
        return "No data available for this comparison.\n"

    sakurs_data = data[sakurs_key]
    other_data = data[other_key]

    table = f"| Metric | sakurs | {other_name} | Ratio |\n"
    table += "|--------|---------|---------|-------|\n"

    # Mean time
    sakurs_mean = format_time(sakurs_data["mean"])
    other_mean = format_time(other_data["mean"])
    ratio = calculate_ratio(other_data["mean"], sakurs_data["mean"])
    table += f"| Mean time | {sakurs_mean} | {other_mean} | sakurs is {ratio} |\n"

    # Std deviation
    sakurs_std = format_time(sakurs_data["stddev"])
    other_std = format_time(other_data["stddev"])
    table += f"| Std dev | ±{sakurs_std} | ±{other_std} | - |\n"

    # Min/Max
    sakurs_min = format_time(sakurs_data["min"])
    sakurs_max = format_time(sakurs_data["max"])
    other_min = format_time(other_data["min"])
    other_max = format_time(other_data["max"])
    table += f"| Min time | {sakurs_min} | {other_min} | - |\n"
    table += f"| Max time | {sakurs_max} | {other_max} | - |\n"

    # Rounds and Iterations
    table += f"| Rounds | {sakurs_data['rounds']} | {other_data['rounds']} | - |\n"
    table += f"| Iterations | {sakurs_data['iterations']} | {other_data['iterations']} | - |\n"

    return table


def format_sentences_comparison(segmentation_data: dict[str, dict[str, Any]]) -> str:
    """Format sentence segmentation comparison from benchmark data."""
    if not segmentation_data:
        return ""

    output = "\n#### Segmentation Results\n\n"

    # Get library names and data
    libraries = list(segmentation_data.keys())
    if len(libraries) < 2:
        return ""

    for lib_name in libraries:
        data = segmentation_data[lib_name]
        if "segmentation" in data:
            seg_info = data["segmentation"]
            output += f"**{lib_name}** (found {seg_info['count']} sentences):\n"
            for i, sent in enumerate(seg_info["sentences"], 1):
                output += f"{i}. {sent}\n"
            output += "\n"

    return output


def generate_markdown_summary(json_file: str) -> None:
    """Generate markdown summary from benchmark results."""
    with open(json_file) as f:
        data = json.load(f)

    benchmarks = data.get("benchmarks", [])
    if not benchmarks:
        print("No benchmark data found!")
        return

    # Extract benchmark data
    benchmark_data = extract_benchmark_data(benchmarks)

    # Generate markdown
    print("# Benchmark Results\n")
    print(
        "Performance comparison of sakurs against other popular sentence segmentation libraries.\n"
    )

    # English section
    print("## English Sentence Segmentation\n")
    print("Comparing sakurs against PySBD for English text processing.\n")

    print("### 400 Character Text Performance\n")
    print(generate_performance_table(benchmark_data, "english", "400_chars"))

    # Extract segmentation results for English 400-char tests
    english_400_data = {}
    for lib in ["sakurs", "pysbd"]:
        key = f"english_400_chars_{lib}"
        if key in benchmark_data and "extra_info" in benchmark_data[key]:
            english_400_data[lib] = benchmark_data[key]["extra_info"]

    if english_400_data:
        print(format_sentences_comparison(english_400_data))

    print("\n### Large Text Performance\n")
    print("Performance on large text (400-char sample repeated 550 times):\n")
    print(generate_performance_table(benchmark_data, "english", "large"))

    # Japanese section
    print("\n## Japanese Sentence Segmentation\n")
    print(
        "Comparing sakurs against ja_sentence_segmenter for Japanese text processing.\n"
    )

    print("### 400 Character Text Performance\n")
    print(generate_performance_table(benchmark_data, "japanese", "400_chars"))

    # Extract segmentation results for Japanese 400-char tests
    japanese_400_data = {}
    for lib in ["sakurs", "ja_segmenter"]:
        key = f"japanese_400_chars_{lib}"
        if key in benchmark_data and "extra_info" in benchmark_data[key]:
            japanese_400_data[lib] = benchmark_data[key]["extra_info"]

    if japanese_400_data:
        print(format_sentences_comparison(japanese_400_data))

    print("\n### Large Text Performance\n")
    print("Performance on large text (Japanese sample repeated 200 times):\n")
    print(generate_performance_table(benchmark_data, "japanese", "large"))

    print("\n## Test Environment")
    print(f"- Python: {data.get('python', 'Unknown')}")
    print(f"- Platform: {data.get('platform', 'Unknown')}")
    print(
        f"- CPU: {data.get('machine_info', {}).get('cpu', {}).get('brand_raw', 'Unknown')}"
    )
    print(f"- Date: {data.get('datetime', 'Unknown')}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python generate_summary.py <benchmark_results.json>")
        sys.exit(1)

    generate_markdown_summary(sys.argv[1])
