#!/usr/bin/env python3
"""Evaluate segmentation accuracy against ground truth."""

import json
import logging
from pathlib import Path

import click

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)


def read_sentences(file_path: Path) -> list[str]:
    """Read sentences from file (one per line)."""
    with open(file_path, encoding="utf-8") as f:
        return [line.strip() for line in f if line.strip()]


def align_sentences(predicted: list[str], reference: list[str]) -> tuple[list[int], list[int]]:
    """Align predicted and reference sentences to find boundaries.

    Returns:
        Tuple of (predicted_boundaries, reference_boundaries) as character positions
    """
    # Build text and track boundaries
    pred_text = ""
    pred_boundaries = []
    for sent in predicted:
        pred_text += sent
        pred_boundaries.append(len(pred_text))
        pred_text += " "  # Add space between sentences

    ref_text = ""
    ref_boundaries = []
    for sent in reference:
        ref_text += sent
        ref_boundaries.append(len(ref_text))
        ref_text += " "

    return pred_boundaries[:-1], ref_boundaries[:-1]  # Exclude last boundary


def calculate_metrics(predicted: list[str], reference: list[str]) -> dict[str, float]:
    """Calculate segmentation metrics.

    Args:
        predicted: List of predicted sentences
        reference: List of reference sentences

    Returns:
        Dictionary with precision, recall, f1, pk, and window_diff
    """
    # Basic boundary-based metrics
    pred_boundaries, ref_boundaries = align_sentences(predicted, reference)

    # Convert to sets for comparison
    pred_set = set(pred_boundaries)
    ref_set = set(ref_boundaries)

    # Calculate precision, recall, F1
    true_positive = len(pred_set & ref_set)
    false_positive = len(pred_set - ref_set)
    false_negative = len(ref_set - pred_set)

    precision = (
        true_positive / (true_positive + false_positive)
        if (true_positive + false_positive) > 0
        else 0
    )
    recall = (
        true_positive / (true_positive + false_negative)
        if (true_positive + false_negative) > 0
        else 0
    )
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0

    # Calculate Pk (probability of error)
    # Simplified version - would need full implementation for paper
    pk = calculate_pk(pred_boundaries, ref_boundaries)

    # Calculate WindowDiff
    window_diff = calculate_window_diff(pred_boundaries, ref_boundaries)

    return {
        "precision": precision,
        "recall": recall,
        "f1": f1,
        "pk": pk,
        "window_diff": window_diff,
        "predicted_sentences": len(predicted),
        "reference_sentences": len(reference),
    }


def calculate_pk(pred_boundaries: list[int], ref_boundaries: list[int], k: int = None) -> float:
    """Calculate Pk metric (Beeferman et al., 1999).

    Pk measures the probability that a randomly chosen pair of sentences
    separated by k sentences are incorrectly classified as being in the
    same segment or different segments.

    Args:
        pred_boundaries: List of predicted boundary positions (character indices)
        ref_boundaries: List of reference boundary positions (character indices)
        k: Window size (default: half of average segment length)

    Returns:
        Pk score (0 = perfect, 1 = worst)
    """
    # Get text length from the last boundary position
    # Add some buffer to ensure we cover the last segment
    if not ref_boundaries and not pred_boundaries:
        return 0.0

    text_length = (
        max(
            max(ref_boundaries) if ref_boundaries else 0,
            max(pred_boundaries) if pred_boundaries else 0,
        )
        + 100
    )  # Add buffer for last segment

    # Convert boundaries to segment assignments
    pred_segments = boundaries_to_segments(pred_boundaries, text_length)
    ref_segments = boundaries_to_segments(ref_boundaries, text_length)

    # Default k to half of average segment length
    if k is None:
        avg_segment_length = (
            text_length // (len(ref_boundaries) + 1) if ref_boundaries else text_length
        )
        k = max(1, avg_segment_length // 2)

    if text_length <= k:
        return 0.0

    errors = 0
    comparisons = text_length - k

    for i in range(comparisons):
        j = i + k
        pred_same = pred_segments[i] == pred_segments[j]
        ref_same = ref_segments[i] == ref_segments[j]

        if pred_same != ref_same:
            errors += 1

    return errors / comparisons if comparisons > 0 else 0.0


def calculate_window_diff(
    pred_boundaries: list[int], ref_boundaries: list[int], k: int = None
) -> float:
    """Calculate WindowDiff metric (Pevzner & Hearst, 2002).

    WindowDiff is similar to Pk but counts the difference in the number
    of boundaries within each window, making it more sensitive to
    near-miss errors.

    Args:
        pred_boundaries: List of predicted boundary positions (character indices)
        ref_boundaries: List of reference boundary positions (character indices)
        k: Window size (default: half of average segment length)

    Returns:
        WindowDiff score (0 = perfect, 1 = worst)
    """
    # Get text length from the last boundary position
    if not ref_boundaries and not pred_boundaries:
        return 0.0

    text_length = (
        max(
            max(ref_boundaries) if ref_boundaries else 0,
            max(pred_boundaries) if pred_boundaries else 0,
        )
        + 100
    )  # Add buffer for last segment

    # Default k to half of average segment length
    if k is None:
        avg_segment_length = (
            text_length // (len(ref_boundaries) + 1) if ref_boundaries else text_length
        )
        k = max(1, avg_segment_length // 2)

    if text_length <= k:
        return 0.0

    errors = 0
    comparisons = text_length - k

    for i in range(comparisons):
        window_end = i + k

        # Count boundaries in window for predicted
        pred_count = sum(1 for pos in pred_boundaries if i < pos <= window_end)

        # Count boundaries in window for reference
        ref_count = sum(1 for pos in ref_boundaries if i < pos <= window_end)

        if pred_count != ref_count:
            errors += 1

    return errors / comparisons if comparisons > 0 else 0.0


def boundaries_to_segments(boundaries: list[int], text_length: int) -> list[int]:
    """Convert boundary positions to segment assignments for each position.

    Args:
        boundaries: List of boundary positions (sorted)
        text_length: Total length of text

    Returns:
        List of segment IDs for each character position
    """
    segments = [0] * text_length
    current_segment = 0
    boundary_idx = 0

    sorted_boundaries = sorted(boundaries)

    for i in range(text_length):
        if boundary_idx < len(sorted_boundaries) and i >= sorted_boundaries[boundary_idx]:
            current_segment += 1
            boundary_idx += 1
        segments[i] = current_segment

    return segments


@click.command()
@click.option(
    "--predicted",
    "-p",
    required=True,
    type=click.Path(exists=True),
    help="Path to predicted sentences file",
)
@click.option(
    "--reference",
    "-r",
    required=True,
    type=click.Path(exists=True),
    help="Path to reference sentences file",
)
@click.option("--output", "-o", type=click.Path(), help="Output JSON file for results")
@click.option(
    "--format",
    "output_format",
    type=click.Choice(["json", "text"]),
    default="text",
    help="Output format",
)
@click.option("--with-ci/--no-ci", default=True, help="Include confidence intervals")
def main(predicted, reference, output, output_format, with_ci):
    """Evaluate segmentation accuracy."""
    # Read files
    pred_sentences = read_sentences(Path(predicted))
    ref_sentences = read_sentences(Path(reference))

    logger.info(f"Predicted sentences: {len(pred_sentences)}")
    logger.info(f"Reference sentences: {len(ref_sentences)}")

    # Calculate metrics
    metrics = calculate_metrics(pred_sentences, ref_sentences)

    # Add confidence intervals if requested
    if with_ci:
        # Import here to avoid circular dependency
        from statistical_analysis import add_confidence_intervals_to_metrics

        # For boundary-based metrics, use number of boundaries as sample size
        pred_boundaries, ref_boundaries = align_sentences(pred_sentences, ref_sentences)
        n_boundaries = len(ref_boundaries)

        # Enhanced metrics with CI
        enhanced_metrics = add_confidence_intervals_to_metrics(metrics, n_samples=n_boundaries)
        metrics = enhanced_metrics

    # Output results
    if output_format == "json" or output:
        result = {
            "predicted_file": str(predicted),
            "reference_file": str(reference),
            "metrics": metrics,
            "with_confidence_intervals": with_ci,
        }

        if output:
            with open(output, "w") as f:
                json.dump(result, f, indent=2)
            logger.info(f"Results saved to {output}")
        else:
            print(json.dumps(result, indent=2))
    else:
        # Text format
        print("\nSegmentation Accuracy Results")
        print("=" * 60)
        print(f"Predicted sentences: {metrics.get('predicted_sentences', 'N/A')}")
        print(f"Reference sentences: {metrics.get('reference_sentences', 'N/A')}")

        # Format metrics with or without CI
        def format_metric(name, key):
            if isinstance(metrics.get(key), dict):
                # With CI
                m = metrics[key]
                return f"{name}: {m['estimate']:.4f} [95% CI: {m['ci_lower']:.4f}, {m['ci_upper']:.4f}]"
            else:
                # Without CI
                return f"{name}: {metrics.get(key, 0):.4f}"

        print(format_metric("Precision", "precision"))
        print(format_metric("Recall", "recall"))
        print(format_metric("F1 Score", "f1"))
        print(format_metric("Pk", "pk"))
        print(format_metric("WindowDiff", "window_diff"))


if __name__ == "__main__":
    main()
