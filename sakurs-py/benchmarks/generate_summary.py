#!/usr/bin/env python3
"""Generate markdown summary from pytest-benchmark JSON results."""

import json
import sys
from typing import Any, Final

# Constants for time formatting thresholds
MICROSECOND_THRESHOLD: Final[float] = 0.001
MILLISECOND_THRESHOLD: Final[float] = 1.0
MICROSECOND_MULTIPLIER: Final[int] = 1_000_000
MILLISECOND_MULTIPLIER: Final[int] = 1_000

# Constants for benchmark data processing
ENGLISH_LANGUAGE_KEY: Final[str] = "english"
JAPANESE_LANGUAGE_KEY: Final[str] = "japanese"
SAKURS_LIBRARY_KEY: Final[str] = "sakurs"
PYSBD_LIBRARY_KEY: Final[str] = "pysbd"
JA_SEGMENTER_LIBRARY_KEY: Final[str] = "ja_segmenter"
TEST_400_CHARS_KEY: Final[str] = "400_chars"
TEST_LARGE_KEY: Final[str] = "large"

# Multipliers for large text tests
ENGLISH_LARGE_MULTIPLIER: Final[int] = 550
JAPANESE_LARGE_MULTIPLIER: Final[int] = 200


def format_time(seconds: float) -> str:
    """Format time in appropriate units."""
    if seconds < MICROSECOND_THRESHOLD:
        return f"{seconds * MICROSECOND_MULTIPLIER:.2f} μs"
    elif seconds < MILLISECOND_THRESHOLD:
        return f"{seconds * MILLISECOND_MULTIPLIER:.2f} ms"
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
    data: dict[str, dict[str, Any]] = {}
    for benchmark in benchmarks:
        name = benchmark["name"]
        # Extract library name and test type from test name
        if SAKURS_LIBRARY_KEY in name:
            library = SAKURS_LIBRARY_KEY
        elif PYSBD_LIBRARY_KEY in name:
            library = PYSBD_LIBRARY_KEY
        elif JA_SEGMENTER_LIBRARY_KEY in name:
            library = JA_SEGMENTER_LIBRARY_KEY
        else:
            continue

        if "400" in name:
            test_type = TEST_400_CHARS_KEY
        elif "large" in name:
            test_type = TEST_LARGE_KEY
        else:
            continue

        if ENGLISH_LANGUAGE_KEY in name:
            language = ENGLISH_LANGUAGE_KEY
        elif JAPANESE_LANGUAGE_KEY in name:
            language = JAPANESE_LANGUAGE_KEY
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
    sakurs_key = f"{language}_{test_type}_{SAKURS_LIBRARY_KEY}"

    if language == ENGLISH_LANGUAGE_KEY:
        other_key = f"{language}_{test_type}_{PYSBD_LIBRARY_KEY}"
        other_name = "PySBD"
    else:
        other_key = f"{language}_{test_type}_{JA_SEGMENTER_LIBRARY_KEY}"
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


def generate_markdown_summary(json_file: str) -> str:
    """Generate markdown summary from benchmark results."""
    with open(json_file) as f:
        data = json.load(f)

    benchmarks = data.get("benchmarks", [])
    if not benchmarks:
        return "No benchmark data found!"

    # Extract benchmark data
    benchmark_data = extract_benchmark_data(benchmarks)

    # Build markdown output as a single string
    output_lines: list[str] = []

    # Header
    output_lines.append("# Benchmark Results\n")
    output_lines.append(
        "Performance comparison of sakurs against other popular sentence segmentation libraries.\n"
    )

    # English section
    output_lines.append("## English Sentence Segmentation\n")
    output_lines.append("Comparing sakurs against PySBD for English text processing.\n")

    output_lines.append("### 400 Character Text Performance\n")
    output_lines.append(
        generate_performance_table(
            benchmark_data, ENGLISH_LANGUAGE_KEY, TEST_400_CHARS_KEY
        )
    )

    # Extract segmentation results for English 400-char tests
    english_400_data: dict[str, dict[str, Any]] = {}
    for lib in [SAKURS_LIBRARY_KEY, PYSBD_LIBRARY_KEY]:
        key = f"{ENGLISH_LANGUAGE_KEY}_{TEST_400_CHARS_KEY}_{lib}"
        if key in benchmark_data and "extra_info" in benchmark_data[key]:
            english_400_data[lib] = benchmark_data[key]["extra_info"]

    if english_400_data:
        output_lines.append(format_sentences_comparison(english_400_data))

    output_lines.append("\n### Large Text Performance\n")
    output_lines.append(
        f"Performance on large text (400-char sample repeated {ENGLISH_LARGE_MULTIPLIER} times):\n"
    )
    output_lines.append(
        generate_performance_table(benchmark_data, ENGLISH_LANGUAGE_KEY, TEST_LARGE_KEY)
    )

    # Japanese section
    output_lines.append("\n## Japanese Sentence Segmentation\n")
    output_lines.append(
        "Comparing sakurs against ja_sentence_segmenter for Japanese text processing.\n"
    )

    output_lines.append("### 400 Character Text Performance\n")
    output_lines.append(
        generate_performance_table(
            benchmark_data, JAPANESE_LANGUAGE_KEY, TEST_400_CHARS_KEY
        )
    )

    # Extract segmentation results for Japanese 400-char tests
    japanese_400_data: dict[str, dict[str, Any]] = {}
    for lib in [SAKURS_LIBRARY_KEY, JA_SEGMENTER_LIBRARY_KEY]:
        key = f"{JAPANESE_LANGUAGE_KEY}_{TEST_400_CHARS_KEY}_{lib}"
        if key in benchmark_data and "extra_info" in benchmark_data[key]:
            japanese_400_data[lib] = benchmark_data[key]["extra_info"]

    if japanese_400_data:
        output_lines.append(format_sentences_comparison(japanese_400_data))

    output_lines.append("\n### Large Text Performance\n")
    output_lines.append(
        f"Performance on large text (Japanese sample repeated {JAPANESE_LARGE_MULTIPLIER} times):\n"
    )
    output_lines.append(
        generate_performance_table(
            benchmark_data, JAPANESE_LANGUAGE_KEY, TEST_LARGE_KEY
        )
    )

    output_lines.append("\n## Test Environment")
    output_lines.append(f"- Python: {data.get('python', 'Unknown')}")
    output_lines.append(f"- Platform: {data.get('platform', 'Unknown')}")
    output_lines.append(
        f"- CPU: {data.get('machine_info', {}).get('cpu', {}).get('brand_raw', 'Unknown')}"
    )
    output_lines.append(f"- Date: {data.get('datetime', 'Unknown')}")

    return "\n".join(output_lines)


def main() -> None:
    """Main entry point for the script."""
    if len(sys.argv) != 2:
        print("Usage: python generate_summary.py <benchmark_results.json>")
        sys.exit(1)

    summary = generate_markdown_summary(sys.argv[1])
    print(summary)


if __name__ == "__main__":
    main()
