"""Unified metrics measurement for benchmarks."""

import json
import os
import subprocess
import tempfile
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

import psutil


@dataclass
class ThroughputMetrics:
    """Throughput measurement results."""

    duration_seconds: float
    data_size_mb: float
    throughput_mbps: float
    num_threads: int


@dataclass
class MemoryMetrics:
    """Memory usage measurement results."""

    peak_memory_mb: float
    initial_memory_mb: float
    memory_delta_mb: float


@dataclass
class AccuracyMetrics:
    """Accuracy metrics for sentence segmentation."""

    precision: float
    recall: float
    f1_score: float
    pk: Optional[float] = None
    window_diff: Optional[float] = None


@dataclass
class BenchmarkResult:
    """Complete benchmark result for a single run."""

    tool: str
    language: str
    dataset: str
    num_threads: int
    throughput: Optional[ThroughputMetrics] = None
    memory: Optional[MemoryMetrics] = None
    accuracy: Optional[AccuracyMetrics] = None
    timestamp: str = ""

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization."""
        result = asdict(self)
        # Clean up None values
        if result.get("throughput") is None:
            del result["throughput"]
        if result.get("memory") is None:
            del result["memory"]
        if result.get("accuracy") is None:
            del result["accuracy"]
        return result


@dataclass
class BenchmarkMetrics:
    """Container for benchmark measurement results (legacy compatibility)."""

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

    def _find_gnu_time(self) -> Optional[str]:
        """Find GNU time command (gtime on macOS, time on Linux)."""
        for cmd in ["gtime", "/usr/bin/time"]:
            try:
                subprocess.run([cmd, "--version"], capture_output=True, check=False)
                return cmd
            except FileNotFoundError:
                continue
        return None

    def _measure_with_gnu_time(
        self, command: List[str], input_file: Optional[str], initial_memory: float
    ) -> MemoryMetrics:
        """Measure memory using GNU time."""
        time_cmd = self._find_gnu_time()

        with tempfile.NamedTemporaryFile(mode="w", delete=False) as tf:
            time_output = tf.name

        try:
            # Run command with GNU time
            full_cmd = [time_cmd, "-v", "-o", time_output] + command

            if input_file:
                with open(input_file, "rb") as f:
                    result = subprocess.run(full_cmd, stdin=f, capture_output=True)
            else:
                result = subprocess.run(full_cmd, capture_output=True)

            # Parse time output
            with open(time_output, "r") as f:
                time_data = f.read()

            # Extract maximum resident set size
            peak_memory = 0
            for line in time_data.split("\n"):
                if "Maximum resident set size" in line:
                    # Value is in KB on Linux, bytes on macOS
                    value = int(line.split(":")[1].strip())
                    # Check if macOS (value would be very large if in bytes)
                    if value > 1000000:
                        peak_memory = value / (1024 * 1024)  # bytes to MB
                    else:
                        peak_memory = value / 1024  # KB to MB
                    break

            memory_delta = peak_memory - initial_memory

            return MemoryMetrics(
                peak_memory_mb=peak_memory,
                initial_memory_mb=initial_memory,
                memory_delta_mb=memory_delta,
            )

        finally:
            os.unlink(time_output)

    def _measure_with_psutil(
        self, command: List[str], input_file: Optional[str], initial_memory: float
    ) -> MemoryMetrics:
        """Measure memory using psutil (less accurate fallback)."""
        # Start subprocess
        if input_file:
            with open(input_file, "rb") as f:
                proc = subprocess.Popen(
                    command, stdin=f, stdout=subprocess.PIPE, stderr=subprocess.PIPE
                )
        else:
            proc = subprocess.Popen(
                command, stdout=subprocess.PIPE, stderr=subprocess.PIPE
            )

        # Monitor memory usage
        peak_memory = initial_memory
        try:
            ps_proc = psutil.Process(proc.pid)
            while proc.poll() is None:
                try:
                    mem_info = ps_proc.memory_info()
                    current_memory = mem_info.rss / (1024 * 1024)  # MB
                    peak_memory = max(peak_memory, current_memory)

                    # Check children too
                    for child in ps_proc.children(recursive=True):
                        try:
                            child_mem = child.memory_info().rss / (1024 * 1024)
                            current_memory += child_mem
                        except (psutil.NoSuchProcess, psutil.AccessDenied):
                            pass

                    peak_memory = max(peak_memory, current_memory)
                    time.sleep(0.01)  # 10ms sampling

                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    break

        except Exception:
            pass

        # Wait for completion
        proc.wait()

        memory_delta = peak_memory - initial_memory

        return MemoryMetrics(
            peak_memory_mb=peak_memory,
            initial_memory_mb=initial_memory,
            memory_delta_mb=memory_delta,
        )

    def enhanced_measure_memory_peak(
        self, command: List[str], input_file: Optional[str] = None
    ) -> MemoryMetrics:
        """Measure peak memory usage during command execution.

        Args:
            command: Command to execute as list of strings
            input_file: Optional input file to pass to stdin

        Returns:
            MemoryMetrics object
        """
        # Get initial memory
        initial_memory = self.process.memory_info().rss / (1024 * 1024)  # MB

        # Use GNU time if available for more accurate measurement
        time_cmd = self._find_gnu_time()
        if time_cmd:
            return self._measure_with_gnu_time(command, input_file, initial_memory)
        else:
            return self._measure_with_psutil(command, input_file, initial_memory)

    def run_throughput_benchmark(
        self,
        command: List[str],
        input_file: str,
        num_threads: int = 1,
        warmup_runs: int = 1,
        test_runs: int = 3,
    ) -> ThroughputMetrics:
        """Run throughput benchmark with warmup and multiple runs.

        Args:
            command: Command to execute
            input_file: Input file path
            num_threads: Number of threads to use
            warmup_runs: Number of warmup runs
            test_runs: Number of test runs to average

        Returns:
            ThroughputMetrics with averaged results
        """
        # Get file size
        file_size_bytes = os.path.getsize(input_file)
        file_size_mb = file_size_bytes / (1024 * 1024)

        # Warmup runs
        for _ in range(warmup_runs):
            with open(input_file, "rb") as f:
                subprocess.run(
                    command, stdin=f, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
                )

        # Test runs
        durations = []
        for _ in range(test_runs):
            start_time = time.time()
            with open(input_file, "rb") as f:
                result = subprocess.run(
                    command, stdin=f, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
                )
            end_time = time.time()

            if result.returncode == 0:
                durations.append(end_time - start_time)

        # Calculate average
        if durations:
            avg_duration = sum(durations) / len(durations)
            throughput = file_size_mb / avg_duration if avg_duration > 0 else 0
            return ThroughputMetrics(
                duration_seconds=avg_duration,
                data_size_mb=file_size_mb,
                throughput_mbps=throughput,
                num_threads=num_threads,
            )
        else:
            raise RuntimeError("All benchmark runs failed")

    def save_results(self, results: List[BenchmarkResult], output_file: str):
        """Save benchmark results to JSON file.

        Args:
            results: List of BenchmarkResult objects
            output_file: Output file path
        """
        data = {
            "results": [r.to_dict() for r in results],
            "metadata": {
                "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
                "platform": os.uname().sysname,
                "python_version": subprocess.check_output(["python", "--version"])
                .decode()
                .strip(),
            },
        }

        output_path = Path(output_file)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, "w") as f:
            json.dump(data, f, indent=2)

    def load_results(self, input_file: str) -> List[BenchmarkResult]:
        """Load benchmark results from JSON file.

        Args:
            input_file: Input file path

        Returns:
            List of BenchmarkResult objects
        """
        with open(input_file, "r") as f:
            data = json.load(f)

        results = []
        for r in data["results"]:
            # Reconstruct nested dataclasses
            if "throughput" in r:
                r["throughput"] = ThroughputMetrics(**r["throughput"])
            if "memory" in r:
                r["memory"] = MemoryMetrics(**r["memory"])
            if "accuracy" in r:
                r["accuracy"] = AccuracyMetrics(**r["accuracy"])

            results.append(BenchmarkResult(**r))

        return results