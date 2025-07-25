#!/usr/bin/env python3
"""Performance tuning examples for sakurs."""

import time

import sakurs


def benchmark_split(
    text: str, processor: sakurs.SentenceSplitter, name: str
) -> list[str]:
    """Benchmark sentence splitting and return results."""
    start = time.perf_counter()
    sentences = processor.split(text)
    end = time.perf_counter()

    elapsed_ms = (end - start) * 1000
    chars_per_sec = len(text) / (end - start) if end > start else 0

    print(
        f"{name:20} | Sentences: {len(sentences):4} | Time: {elapsed_ms:7.2f}ms | Speed: {chars_per_sec:,.0f} chars/sec"
    )
    return sentences


def main() -> None:
    """Run performance tuning examples."""
    # Create a sample long text
    base_text = (
        """The quick brown fox jumps over the lazy dog. This is a test sentence.
    Machine learning models require large amounts of data. Natural language processing
    is fascinating. Python is a great programming language. """
        * 100
    )

    print(f"Text length: {len(base_text):,} characters")
    print(f"{'Configuration':20} | {'Sentences':10} | {'Time':>10} | {'Speed':>15}")
    print("-" * 70)

    # Default configuration
    default_processor = sakurs.SentenceSplitter(language="en")
    benchmark_split(base_text, default_processor, "Default")

    # Small chunks (more overhead, but may be better for memory)
    small_processor = sakurs.SentenceSplitter(
        language="en", chunk_kb=4, execution_mode="adaptive"
    )
    benchmark_split(base_text, small_processor, "Small chunks")

    # Large chunks (less overhead for long texts)
    large_processor = sakurs.SentenceSplitter(
        language="en", chunk_kb=32, execution_mode="adaptive"
    )
    benchmark_split(base_text, large_processor, "Large chunks")

    # Sequential mode (single-threaded)
    sequential_processor = sakurs.SentenceSplitter(
        language="en", chunk_kb=8, execution_mode="sequential"
    )
    benchmark_split(base_text, sequential_processor, "Sequential")

    # Parallel mode with 4 threads
    parallel_processor = sakurs.SentenceSplitter(
        language="en", chunk_kb=8, threads=4, execution_mode="parallel"
    )
    benchmark_split(base_text, parallel_processor, "Parallel (4 threads)")

    # Streaming mode (for large files)
    streaming_processor = sakurs.SentenceSplitter(
        language="en",
        streaming=True,
        stream_chunk_mb=1,  # 1MB
    )
    benchmark_split(base_text, streaming_processor, "Streaming mode")

    print("\nPerformance Comparison using split() function:")
    print("-" * 70)

    # Direct split() function with different modes
    start = time.perf_counter()
    _ = sakurs.split(base_text, execution_mode="sequential")
    sequential_time = (time.perf_counter() - start) * 1000
    print(f"Sequential mode:     {sequential_time:7.2f}ms")

    start = time.perf_counter()
    _ = sakurs.split(base_text, execution_mode="parallel", threads=4)
    parallel_time = (time.perf_counter() - start) * 1000
    print(f"Parallel (4 threads): {parallel_time:7.2f}ms")

    start = time.perf_counter()
    _ = sakurs.split(base_text, execution_mode="adaptive")
    adaptive_time = (time.perf_counter() - start) * 1000
    print(f"Adaptive mode:       {adaptive_time:7.2f}ms")

    print("\nTips:")
    print("- For short texts (<10KB), default settings are usually best")
    print("- For long texts, increase chunk_kb to reduce overhead")
    print("- For batch processing, use parallel mode with appropriate thread count")
    print("- For interactive use, use adaptive mode for automatic optimization")
    print("- For memory-constrained environments, use streaming mode")


if __name__ == "__main__":
    main()
