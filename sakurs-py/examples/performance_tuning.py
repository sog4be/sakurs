#!/usr/bin/env python3
"""Performance tuning examples for sakurs."""

import time

import sakurs


def benchmark_split(text: str, processor: sakurs.Processor, name: str) -> list[str]:
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
    default_processor = sakurs.Processor("en")
    benchmark_split(base_text, default_processor, "Default")

    # Small chunks (more overhead, but may be better for memory)
    small_config = sakurs.ProcessorConfig(
        chunk_size=4096,
        overlap_size=128,
        num_threads=None,  # Auto
    )
    small_processor = sakurs.Processor("en", small_config)
    benchmark_split(base_text, small_processor, "Small chunks")

    # Large chunks (less overhead for long texts)
    large_config = sakurs.ProcessorConfig(
        chunk_size=32768, overlap_size=512, num_threads=None
    )
    large_processor = sakurs.Processor("en", large_config)
    benchmark_split(base_text, large_processor, "Large chunks")

    # Single-threaded (for comparison)
    single_config = sakurs.ProcessorConfig(
        chunk_size=8192, overlap_size=256, num_threads=1
    )
    single_processor = sakurs.Processor("en", single_config)
    benchmark_split(base_text, single_processor, "Single-threaded")

    # Multi-threaded
    multi_config = sakurs.ProcessorConfig(
        chunk_size=8192, overlap_size=256, num_threads=4
    )
    multi_processor = sakurs.Processor("en", multi_config)
    benchmark_split(base_text, multi_processor, "4 threads")

    print("\nTips:")
    print("- For short texts (<10KB), default settings are usually best")
    print("- For long texts, increase chunk_size to reduce overhead")
    print("- For batch processing, configure threads based on CPU cores")
    print("- For interactive use, limit threads to avoid UI freezing")


if __name__ == "__main__":
    main()
