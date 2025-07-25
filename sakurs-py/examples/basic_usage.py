#!/usr/bin/env python3
"""Basic usage examples for sakurs sentence splitter."""

import os
import tempfile

import sakurs


def main() -> None:  # noqa: PLR0915
    """Run basic usage examples."""
    # Example 1: Simple sentence splitting
    print("=== Example 1: Basic Usage ===")
    text = "Hello world. This is a test. How are you?"
    sentences = sakurs.split(text)
    for i, sentence in enumerate(sentences, 1):
        print(f"{i}. {sentence}")
    print()

    # Example 2: Japanese text
    print("=== Example 2: Japanese Text ===")
    japanese_text = (
        "これは日本語のテストです。文分割が正しく動作しますか？確認してみましょう。"  # noqa: RUF001
    )
    sentences = sakurs.split(japanese_text, language="ja")
    for i, sentence in enumerate(sentences, 1):
        print(f"{i}. {sentence}")
    print()

    # Example 3: Using SentenceSplitter for multiple texts
    print("=== Example 3: Reusing SentenceSplitter ===")
    processor = sakurs.SentenceSplitter(language="en")

    texts = [
        "First document. It has two sentences.",
        "Second document? Yes, it has questions!",
        "Third one... With ellipsis. And more.",
    ]

    for idx, text in enumerate(texts, 1):
        print(f"Document {idx}:")
        sentences = processor.split(text)
        for i, sentence in enumerate(sentences, 1):
            print(f"  {i}. {sentence}")
    print()

    # Example 4: Iterator-based processing with iter_split()
    print("=== Example 4: Iterator-based Processing ===")
    text = "First sentence. Second sentence. Third sentence."
    print("Using iter_split() for responsive processing:")
    for i, sentence in enumerate(sakurs.iter_split(text), 1):
        print(f"  {i}. {sentence}")
    print()

    # Example 5: Processing files
    print("=== Example 5: File Processing ===")
    with tempfile.NamedTemporaryFile(mode="w", suffix=".txt", delete=False) as f:
        f.write("File sentence one. File sentence two.\n")
        f.write("File sentence three on new line.")
        temp_path = f.name

    try:
        # Process file with regular split
        sentences = sakurs.split(temp_path)
        print(f"From file using split(): {len(sentences)} sentences")

        # Process file with iter_split
        print("From file using iter_split():")
        for i, sentence in enumerate(sakurs.iter_split(temp_path), 1):
            print(f"  {i}. {sentence}")
    finally:
        os.unlink(temp_path)
    print()

    # Example 6: Handling edge cases
    print("=== Example 6: Edge Cases ===")
    edge_cases = [
        "",  # Empty string
        "No punctuation",  # No sentence ending
        "Multiple... dots... here.",  # Multiple dots
        "Mr. Smith went to Dr. Brown's office.",  # Abbreviations
        "What?! Really?!",  # Multiple punctuation
    ]

    for text in edge_cases:
        print(f"Input: '{text}'")
        sentences = sakurs.split(text)
        print(f"Output: {sentences}")
        print()

    # Example 7: Using load() function
    print("=== Example 7: Using load() Function ===")
    nlp = sakurs.load("en")
    text = "Loaded with load(). Works great! Doesn't it?"
    sentences = nlp.split(text)
    for i, sentence in enumerate(sentences, 1):
        print(f"{i}. {sentence}")
    print()

    # Note about split_large_file()
    print("Note: For memory-efficient processing of very large files,")
    print("      use sakurs.split_large_file(). See streaming_demo.py for examples.")


if __name__ == "__main__":
    main()
