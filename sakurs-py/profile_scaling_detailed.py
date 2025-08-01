#!/usr/bin/env python3
"""Detailed profiling to analyze O(N²) scaling behavior with abbreviations."""

import time
import sakurs
import random
import string


def generate_text_with_abbreviations(num_sentences, abbr_density=0.3):
    """Generate text with controlled abbreviation density."""
    # Common sentence patterns with abbreviations
    patterns = [
        "Dr. Smith works at the university.",
        "The company was founded in the U.S.A. by entrepreneurs.",
        "Ph.D. students study for many years.",
        "Mr. Johnson met Mrs. Davis at the conference.",
        "The Inc. reported profits of $1.5 million.",
        "Prof. Williams teaches at M.I.T. every semester.",
        "The meeting is scheduled for 3 p.m. today.",
        "She graduated with a B.A. in psychology.",
        "The Co. specializes in software development.",
        "Lt. Colonel Brown served in the military.",
    ]
    
    # Non-abbreviation sentences
    regular_sentences = [
        "The weather today is particularly nice.",
        "Technology continues to advance rapidly.",
        "Many people enjoy reading books.",
        "The conference was very successful.",
        "Innovation drives economic growth.",
        "Education is important for society.",
        "The project deadline is approaching.",
        "Teamwork leads to better results.",
        "Research requires careful planning.",
        "Success comes from hard work.",
    ]
    
    sentences = []
    for i in range(num_sentences):
        if random.random() < abbr_density:
            sentences.append(random.choice(patterns))
        else:
            sentences.append(random.choice(regular_sentences))
    
    return " ".join(sentences)


def measure_with_warmup(text, language="en", warmup=2, iterations=5):
    """Measure processing time with warmup runs."""
    # Warmup runs
    for _ in range(warmup):
        _ = sakurs.split(text, language=language)
    
    # Actual measurements
    times = []
    for _ in range(iterations):
        start = time.perf_counter()
        _ = sakurs.split(text, language=language)
        end = time.perf_counter()
        times.append(end - start)
    
    return sum(times) / len(times) * 1000  # Return average in ms


def main():
    print("Detailed O(N²) Scaling Analysis with Abbreviations")
    print("=" * 80)
    
    # Test with increasing text sizes
    sentence_counts = [10, 50, 100, 200, 500, 1000, 2000, 5000]
    
    print(f"{'Sentences':>10} | {'Chars':>10} | {'Periods':>8} | {'Time (ms)':>10} | {'ms/char':>10} | {'Scaling':>10}")
    print("-" * 80)
    
    baseline_time = None
    baseline_chars = None
    
    for num_sentences in sentence_counts:
        # Generate text with ~30% abbreviation density
        text = generate_text_with_abbreviations(num_sentences, abbr_density=0.3)
        
        # Count periods (potential abbreviation checks)
        period_count = text.count('.')
        char_count = len(text)
        
        # Measure processing time
        avg_time = measure_with_warmup(text, "en")
        time_per_char = avg_time / char_count
        
        # Calculate scaling
        if baseline_time is None:
            baseline_time = avg_time
            baseline_chars = char_count
            scaling = 1.0
        else:
            # Expected linear scaling
            expected_time = baseline_time * (char_count / baseline_chars)
            scaling = avg_time / expected_time
        
        print(f"{num_sentences:>10} | {char_count:>10} | {period_count:>8} | {avg_time:>10.2f} | {time_per_char:>10.6f} | {scaling:>10.2f}x")
    
    print("=" * 80)
    
    # Additional analysis with very high abbreviation density
    print("\nExtreme case: Text with very high abbreviation density")
    print("-" * 80)
    
    for num_sentences in [100, 500, 1000]:
        # Generate text with 90% abbreviation density
        text = generate_text_with_abbreviations(num_sentences, abbr_density=0.9)
        period_count = text.count('.')
        char_count = len(text)
        
        avg_time = measure_with_warmup(text, "en")
        
        print(f"Sentences: {num_sentences}, Chars: {char_count}, Periods: {period_count}, Time: {avg_time:.2f}ms")
    
    # Test pure scaling without abbreviations
    print("\nControl: Text without abbreviations")
    print("-" * 80)
    
    for num_sentences in [100, 500, 1000]:
        # Generate text with 0% abbreviation density
        text = generate_text_with_abbreviations(num_sentences, abbr_density=0.0)
        period_count = text.count('.')
        char_count = len(text)
        
        avg_time = measure_with_warmup(text, "en")
        
        print(f"Sentences: {num_sentences}, Chars: {char_count}, Periods: {period_count}, Time: {avg_time:.2f}ms")


if __name__ == "__main__":
    main()