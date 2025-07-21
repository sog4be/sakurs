#!/usr/bin/env python3
"""Demonstration of sakurs streaming APIs for different use cases.

This example shows how to use:
1. iter_split() - For responsive iteration (loads all data, yields incrementally)
2. split_large_file() - For true memory-efficient processing of large files
"""

import os
import tempfile
import time

import sakurs


def demo_iter_split() -> None:
    """Demonstrate iter_split() for responsive processing."""
    print("=== iter_split() Demo ===")
    print("This loads all data into memory but yields results incrementally.")
    print()

    # Example 1: Processing a string
    text = """This is the first sentence. Here's the second one!
    And a third? Finally, the fourth sentence with some complexity:
    it includes a colon. The fifth one ends here."""

    print("Processing text string:")
    start_time = time.time()

    for i, sentence in enumerate(sakurs.iter_split(text), 1):
        print(f"  Sentence {i}: {sentence!r}")
        # Simulate some processing time
        time.sleep(0.1)

    elapsed = time.time() - start_time
    print(f"Total time: {elapsed:.2f}s")
    print()

    # Example 2: Processing a file
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        f.write("First line sentence. Second line sentence.\n")
        f.write("Third sentence spans\nmultiple lines. Fourth sentence.")
        temp_path = f.name

    try:
        print("Processing file:")
        for i, sentence in enumerate(sakurs.iter_split(temp_path), 1):
            print(f"  Sentence {i}: {sentence!r}")
    finally:
        os.unlink(temp_path)

    print()


def demo_split_large_file() -> None:
    """Demonstrate split_large_file() for memory-efficient processing."""
    print("=== split_large_file() Demo ===")
    print("This processes files in chunks with limited memory usage.")
    print()

    # Create a moderately large test file
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        print("Creating test file with 1000 sentences...")
        for i in range(1000):
            f.write(f"This is sentence number {i}. ")
            if i % 10 == 0:
                f.write("\n")  # Add some line breaks
        temp_path = f.name

    file_size_mb = os.path.getsize(temp_path) / (1024 * 1024)
    print(f"File size: {file_size_mb:.2f} MB")
    print()

    try:
        # Process with very limited memory (1MB)
        print("Processing with max_memory_mb=1:")
        start_time = time.time()
        sentence_count = 0

        for _sentence in sakurs.split_large_file(temp_path, max_memory_mb=1):
            sentence_count += 1
            # Show progress every 100 sentences
            if sentence_count % 100 == 0:
                print(f"  Processed {sentence_count} sentences...")

        elapsed = time.time() - start_time
        print(f"Total sentences: {sentence_count}")
        print(f"Processing time: {elapsed:.2f}s")
        print(f"Processing rate: {sentence_count / elapsed:.0f} sentences/second")

    finally:
        os.unlink(temp_path)

    print()


def demo_language_support() -> None:
    """Demonstrate streaming with different languages."""
    print("=== Language Support Demo ===")
    print()

    # Japanese example
    japanese_text = """
    これは最初の文です。二番目の文はこちらです！
    質問文もありますか？もちろん、複雑な文章も処理できます：
    例えば、このような文章です。最後の文です。
    """  # noqa: RUF001

    print("Japanese text processing:")
    for i, sentence in enumerate(sakurs.iter_split(japanese_text, language="ja"), 1):
        print(f"  文 {i}: {sentence!r}")

    print()


def demo_use_case_comparison() -> None:
    """Compare when to use each API."""
    print("=== Use Case Comparison ===")
    print()

    # Create test files of different sizes
    small_text = "Small file. " * 100  # ~1KB
    medium_text = "Medium file. " * 10000  # ~120KB

    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        f.write(small_text)
        small_file = f.name

    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        f.write(medium_text)
        medium_file = f.name

    try:
        # For small files, iter_split() is fine
        print("Small file (1KB) - using iter_split():")
        start = time.time()
        sentences = list(sakurs.iter_split(small_file))
        print(f"  Loaded {len(sentences)} sentences in {time.time() - start:.3f}s")

        # For larger files where memory is a concern, use split_large_file()
        print("\nMedium file (120KB) - comparing both methods:")

        print("  Using iter_split() (loads all at once):")
        start = time.time()
        sentences = list(sakurs.iter_split(medium_file))
        iter_time = time.time() - start
        print(f"    Processed {len(sentences)} sentences in {iter_time:.3f}s")

        print("  Using split_large_file() (memory-efficient):")
        start = time.time()
        sentences = list(sakurs.split_large_file(medium_file, max_memory_mb=1))
        large_time = time.time() - start
        print(f"    Processed {len(sentences)} sentences in {large_time:.3f}s")

        print(f"\nTime difference: {abs(iter_time - large_time):.3f}s")
        print(
            "Note: split_large_file() may be slightly slower but uses much less memory."
        )

    finally:
        os.unlink(small_file)
        os.unlink(medium_file)


def main() -> None:
    """Run all demonstrations."""
    print("Sakurs Streaming API Demonstrations")
    print("=" * 40)
    print()

    demo_iter_split()
    demo_split_large_file()
    demo_language_support()
    demo_use_case_comparison()

    print("\nSummary:")
    print(
        "- Use iter_split() when you want responsive processing and memory is not a concern"
    )
    print(
        "- Use split_large_file() when processing very large files with limited memory"
    )
    print("- Both APIs support all sakurs language configurations")


if __name__ == "__main__":
    main()
