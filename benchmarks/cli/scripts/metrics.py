"""Unified metrics measurement for benchmarks."""

import time
from dataclasses import dataclass
from typing import Dict, List, Optional, Tuple

import psutil


@dataclass
class BenchmarkMetrics:
    """Container for benchmark measurement results."""

    throughput_mbps: Optional[float] = None
    memory_peak_mib: Optional[float] = None
    precision: Optional[float] = None
    recall: Optional[float] = None
    f1_score: Optional[float] = None
    pk_score: Optional[float] = None
    windowdiff_score: Optional[float] = None
    duration_seconds: Optional[float] = None
    data_size_mb: Optional[float] = None


class MetricsMeasurer:
    """Unified metrics measurement for benchmarks."""

    def __init__(self):
        """Initialize the metrics measurer."""
        self.process = psutil.Process()

    def measure_throughput(self, duration: float, data_size_mb: float) -> float:
        """Calculate throughput in MB/s.

        Args:
            duration: Time taken in seconds
            data_size_mb: Size of processed data in MB

        Returns:
            Throughput in MB/s
        """
        if duration <= 0:
            raise ValueError("Duration must be positive")
        return data_size_mb / duration

    def measure_memory_peak(self) -> float:
        """Get peak memory usage in MiB.

        Returns:
            Peak RSS memory in MiB
        """
        memory_info = self.process.memory_info()
        return memory_info.rss / (1024 * 1024)  # Convert bytes to MiB

    def calculate_precision_recall_f1(
        self, predicted: List[int], gold: List[int]
    ) -> Dict[str, float]:
        """Calculate precision, recall, and F1 score for sentence boundaries.

        Args:
            predicted: List of predicted sentence boundary positions
            gold: List of gold standard sentence boundary positions

        Returns:
            Dictionary with precision, recall, and f1 scores
        """
        predicted_set = set(predicted)
        gold_set = set(gold)

        true_positives = len(predicted_set & gold_set)
        false_positives = len(predicted_set - gold_set)
        false_negatives = len(gold_set - predicted_set)

        precision = (
            true_positives / (true_positives + false_positives)
            if (true_positives + false_positives) > 0
            else 0.0
        )
        recall = (
            true_positives / (true_positives + false_negatives)
            if (true_positives + false_negatives) > 0
            else 0.0
        )
        f1 = (
            2 * (precision * recall) / (precision + recall)
            if (precision + recall) > 0
            else 0.0
        )

        return {"precision": precision, "recall": recall, "f1": f1}

    def calculate_pk_windowdiff(
        self, reference: str, hypothesis: str, k: Optional[int] = None
    ) -> Dict[str, float]:
        """Calculate Pk and WindowDiff scores for text segmentation.

        Args:
            reference: Reference segmentation (1 for boundary, 0 for non-boundary)
            hypothesis: Hypothesis segmentation (1 for boundary, 0 for non-boundary)
            k: Window size (default: half of average segment length)

        Returns:
            Dictionary with pk and windowdiff scores
        """
        if len(reference) != len(hypothesis):
            raise ValueError("Reference and hypothesis must have the same length")

        # Calculate k if not provided
        if k is None:
            num_segments = reference.count("1") + 1
            k = max(1, len(reference) // (2 * num_segments))

        # Pk calculation
        pk_errors = 0
        for i in range(len(reference) - k):
            ref_boundaries = reference[i : i + k + 1].count("1")
            hyp_boundaries = hypothesis[i : i + k + 1].count("1")
            if (ref_boundaries > 0) != (hyp_boundaries > 0):
                pk_errors += 1

        pk_score = pk_errors / (len(reference) - k) if len(reference) > k else 0.0

        # WindowDiff calculation
        windowdiff_errors = 0
        for i in range(len(reference) - k):
            ref_boundaries = reference[i : i + k + 1].count("1")
            hyp_boundaries = hypothesis[i : i + k + 1].count("1")
            if ref_boundaries != hyp_boundaries:
                windowdiff_errors += 1

        windowdiff_score = (
            windowdiff_errors / (len(reference) - k) if len(reference) > k else 0.0
        )

        return {"pk": pk_score, "windowdiff": windowdiff_score}

    def measure_with_timer(self, func, *args, **kwargs) -> Tuple[float, any]:
        """Measure execution time of a function.

        Args:
            func: Function to measure
            *args: Positional arguments for func
            **kwargs: Keyword arguments for func

        Returns:
            Tuple of (duration_seconds, function_result)
        """
        start_time = time.perf_counter()
        result = func(*args, **kwargs)
        duration = time.perf_counter() - start_time
        return duration, result