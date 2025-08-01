#!/usr/bin/env python3
"""Profile Rust bindings performance to analyze scaling behavior."""

import time
import sakurs
import sys


def measure_processing_time(text, language="ja", iterations=3):
    """Measure average processing time for given text."""
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        _ = sakurs.split(text, language=language)
        end = time.perf_counter()
        times.append(end - start)
    
    avg_time = sum(times) / len(times)
    return avg_time * 1000  # Convert to milliseconds


def generate_test_text(base_sentence, multiplier):
    """Generate test text by repeating base sentence."""
    return (base_sentence + " ") * multiplier


def main():
    # Japanese test sentence
    base_sentence = "これは日本語のテスト文です。"
    
    # Test different text sizes
    sizes = [1, 10, 50, 100, 200, 500, 1000]
    
    print("Profiling Rust bindings scaling behavior...")
    print("=" * 60)
    print(f"{'Size':>6} | {'Text Length':>12} | {'Time (ms)':>10} | {'ms/char':>10} | {'Scaling':>10}")
    print("-" * 60)
    
    baseline_time = None
    baseline_size = None
    
    for size in sizes:
        text = generate_test_text(base_sentence, size)
        text_length = len(text)
        
        # Measure processing time
        avg_time = measure_processing_time(text, "ja")
        time_per_char = avg_time / text_length
        
        # Calculate scaling factor
        if baseline_time is None:
            baseline_time = avg_time
            baseline_size = size
            scaling = 1.0
        else:
            # Expected linear scaling
            expected_time = baseline_time * (size / baseline_size)
            scaling = avg_time / expected_time
        
        print(f"{size:>6}x | {text_length:>12} | {avg_time:>10.2f} | {time_per_char:>10.6f} | {scaling:>10.2f}x")
    
    print("=" * 60)
    
    # Additional detailed analysis for large text
    print("\nDetailed analysis for large text (200x):")
    large_text = generate_test_text(base_sentence, 200)
    
    # Time individual operations
    print("- Text length:", len(large_text), "characters")
    
    # Warm up
    _ = sakurs.split(large_text, language="ja")
    
    # Detailed timing
    start = time.perf_counter()
    sentences = sakurs.split(large_text, language="ja")
    total_time = time.perf_counter() - start
    
    print(f"- Total processing time: {total_time*1000:.2f}ms")
    print(f"- Number of sentences: {len(sentences)}")
    print(f"- Average time per sentence: {(total_time*1000)/len(sentences):.2f}ms")
    print(f"- Processing rate: {len(large_text)/(total_time*1000):.2f} chars/ms")


if __name__ == "__main__":
    main()