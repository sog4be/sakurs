"""Parallel execution support for benchmarks."""

import json
import os
import subprocess
import tempfile
from pathlib import Path
from typing import Dict, List, Optional


class ParallelRunner:
    """Manages parallel execution of sakurs with different thread counts."""

    def __init__(self, sakurs_bin: Optional[str] = None):
        """Initialize the parallel runner.

        Args:
            sakurs_bin: Path to sakurs binary (default: use cargo run)
        """
        self.sakurs_bin = sakurs_bin or ["cargo", "run", "--release", "--bin", "sakurs", "--"]

    def run_with_threads(
        self,
        input_file: str,
        output_file: str,
        language: str,
        num_threads: int,
        config_override: Optional[Dict] = None,
    ) -> subprocess.CompletedProcess:
        """Run sakurs with specified number of threads.

        Args:
            input_file: Input file path
            output_file: Output file path
            language: Language (english/japanese)
            num_threads: Number of worker threads
            config_override: Additional config options

        Returns:
            Completed process result
        """
        # Create temporary config file with thread settings
        config = {
            "performance": {
                "worker_threads": num_threads,
                "parallel_threshold_mb": 0,  # Force parallel for benchmarking
            }
        }

        if config_override:
            # Merge additional config
            for key, value in config_override.items():
                if key not in config:
                    config[key] = value
                else:
                    config[key].update(value)

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as config_file:
            json.dump(config, config_file)
            config_path = config_file.name

        try:
            # Build command
            cmd = []
            if isinstance(self.sakurs_bin, list):
                cmd.extend(self.sakurs_bin)
            else:
                cmd.append(self.sakurs_bin)

            cmd.extend(
                [
                    "process",
                    "-i",
                    input_file,
                    "-o",
                    output_file,
                    "-l",
                    language,
                    "-c",
                    config_path,
                    "-p",  # Force parallel processing
                    "-q",  # Quiet mode
                ]
            )

            # Run with environment variable as fallback
            env = os.environ.copy()
            env["RAYON_NUM_THREADS"] = str(num_threads)

            result = subprocess.run(
                cmd,
                env=env,
                capture_output=True,
                text=True,
            )

            return result

        finally:
            # Clean up temp config
            os.unlink(config_path)

    def benchmark_threads(
        self,
        input_file: str,
        language: str,
        thread_counts: List[int] = [1, 2, 4, 8],
        runs_per_thread: int = 3,
    ) -> Dict[int, List[float]]:
        """Benchmark with different thread counts using hyperfine.

        Args:
            input_file: Input file path
            language: Language (english/japanese)
            thread_counts: List of thread counts to test
            runs_per_thread: Number of runs per thread count

        Returns:
            Dictionary mapping thread count to list of execution times
        """
        results = {}

        for threads in thread_counts:
            with tempfile.NamedTemporaryFile(delete=False) as output_file:
                output_path = output_file.name

            try:
                # Create config for this thread count
                config = {
                    "performance": {
                        "worker_threads": threads,
                        "parallel_threshold_mb": 0,
                    }
                }

                with tempfile.NamedTemporaryFile(
                    mode="w", suffix=".json", delete=False
                ) as config_file:
                    json.dump(config, config_file)
                    config_path = config_file.name

                # Build hyperfine command
                sakurs_cmd = []
                if isinstance(self.sakurs_bin, list):
                    sakurs_cmd.extend(self.sakurs_bin)
                else:
                    sakurs_cmd.append(self.sakurs_bin)

                sakurs_cmd.extend(
                    [
                        "process",
                        "-i",
                        input_file,
                        "-o",
                        output_path,
                        "-l",
                        language,
                        "-c",
                        config_path,
                        "-p",
                        "-q",
                    ]
                )

                hyperfine_cmd = [
                    "hyperfine",
                    "--warmup",
                    "1",
                    "--runs",
                    str(runs_per_thread),
                    "--export-json",
                    "-",
                    "--shell",
                    "none",
                    " ".join(sakurs_cmd) if isinstance(self.sakurs_bin, list) else sakurs_cmd,
                ]

                # Run hyperfine
                env = os.environ.copy()
                env["RAYON_NUM_THREADS"] = str(threads)

                result = subprocess.run(
                    hyperfine_cmd,
                    env=env,
                    capture_output=True,
                    text=True,
                )

                if result.returncode == 0:
                    # Parse hyperfine JSON output
                    data = json.loads(result.stdout)
                    times = data["results"][0]["times"]
                    results[threads] = times
                else:
                    print(f"Error running benchmark for {threads} threads: {result.stderr}")
                    results[threads] = []

                os.unlink(config_path)

            finally:
                if os.path.exists(output_path):
                    os.unlink(output_path)

        return results
