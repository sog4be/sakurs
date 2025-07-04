#!/usr/bin/env python3
"""Benchmark NLTK Punkt sentence tokenizer with Brown Corpus data."""

import json
import sys
from pathlib import Path
from typing import Any

# Add parent directories to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from baselines.nltk_punkt.segmenter import create_segmenter
from data.brown_corpus.loader import is_available, load_full_corpus, load_subset


def calculate_metrics(predicted: list[int], actual: list[int], text_len: int) -> dict[str, float]:
    """Calculate accuracy metrics for boundary detection.

    Args:
        predicted: Predicted boundary positions
        actual: Actual boundary positions
        text_len: Length of the text

    Returns:
        Dictionary with precision, recall, and F1 score
    """
    predicted_set = set(predicted)
    actual_set = set(actual)

    true_positives = len(predicted_set & actual_set)
    false_positives = len(predicted_set - actual_set)
    false_negatives = len(actual_set - predicted_set)

    precision = true_positives / (true_positives + false_positives) if predicted else 0.0
    recall = true_positives / (true_positives + false_negatives) if actual else 0.0
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0.0

    return {
        "precision": precision,
        "recall": recall,
        "f1_score": f1,
        "true_positives": true_positives,
        "false_positives": false_positives,
        "false_negatives": false_negatives,
    }


DEFAULT_WARMUP_RUNS = 3  # Should match Rust config


def benchmark_punkt(
    subset_size: int = None, warmup_runs: int = DEFAULT_WARMUP_RUNS
) -> dict[str, Any]:
    """Run benchmark on NLTK Punkt with Brown Corpus data.

    Args:
        subset_size: Number of sentences to use (None for full corpus)
        warmup_runs: Number of warmup runs before timing

    Returns:
        Dictionary with benchmark results
    """
    # Load corpus data
    if subset_size:
        corpus_data = load_subset(subset_size)
        dataset_name = f"brown_corpus_subset_{subset_size}"
    else:
        corpus_data = load_full_corpus()
        dataset_name = "brown_corpus_full"

    # Initialize segmenter
    segmenter = create_segmenter()

    # Warmup runs
    for _ in range(warmup_runs):
        _ = segmenter.segment(corpus_data["text"])

    # Timed run
    boundaries, processing_time = segmenter.segment_with_timing(corpus_data["text"])

    # Calculate metrics
    metrics = calculate_metrics(boundaries, corpus_data["boundaries"], len(corpus_data["text"]))

    # Calculate throughput
    num_sentences = len(corpus_data["boundaries"])
    sentences_per_sec = num_sentences / processing_time if processing_time > 0 else 0
    chars_per_sec = len(corpus_data["text"]) / processing_time if processing_time > 0 else 0

    return {
        "tool": "nltk_punkt",
        "dataset": dataset_name,
        "text_length": len(corpus_data["text"]),
        "num_sentences": num_sentences,
        "processing_time_seconds": processing_time,
        "sentences_per_second": sentences_per_sec,
        "characters_per_second": chars_per_sec,
        "metrics": metrics,
        "predicted_boundaries": len(boundaries),
        "actual_boundaries": len(corpus_data["boundaries"]),
    }


def main():
    """Run benchmarks and output results as JSON."""
    import argparse

    parser = argparse.ArgumentParser(description="Benchmark NLTK Punkt tokenizer")
    parser.add_argument(
        "--subset", type=int, default=None, help="Use subset of N sentences (default: full corpus)"
    )
    parser.add_argument(
        "--output", type=str, default=None, help="Output file for results (default: stdout)"
    )

    args = parser.parse_args()

    # Check if Brown Corpus is available
    if not is_available():
        print("Error: Brown Corpus data not available.", file=sys.stderr)
        print("Please run: cd benchmarks/data/brown_corpus && make download", file=sys.stderr)
        sys.exit(1)

    # Run benchmark
    try:
        results = benchmark_punkt(subset_size=args.subset)

        # Output results
        json_output = json.dumps(results, indent=2)
        if args.output:
            with open(args.output, "w") as f:
                f.write(json_output)
        else:
            print(json_output)

    except Exception as e:
        print(f"Error running benchmark: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
