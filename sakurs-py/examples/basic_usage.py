#!/usr/bin/env python3
"""Basic usage examples for sakurs sentence splitter."""

import sakurs


def main() -> None:
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

    # Example 3: Using Processor for multiple texts
    print("=== Example 3: Reusing Processor ===")
    processor = sakurs.Processor(language="en")

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

    # Example 4: Handling edge cases
    print("=== Example 4: Edge Cases ===")
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


if __name__ == "__main__":
    main()
