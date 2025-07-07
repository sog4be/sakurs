#!/usr/bin/env python3
"""Error handling examples for sakurs."""

import sakurs


def main() -> None:
    """Run error handling examples."""
    print("=== Sakurs Error Handling Examples ===\n")

    # Example 1: Unsupported language
    print("1. Handling unsupported language:")
    try:
        processor = sakurs.Processor("fr")  # French is not supported
    except sakurs.SakursError as e:
        print(f"   Error caught: {e}")
    print()

    # Example 2: Invalid language code
    print("2. Handling invalid language code:")
    try:
        sentences = sakurs.split("Hello world.", language="invalid")
    except sakurs.SakursError as e:
        print(f"   Error caught: {e}")
    print()

    # Example 3: Empty and None inputs
    print("3. Handling edge case inputs:")
    edge_cases = [
        ("Empty string", ""),
        ("Whitespace only", "   \n\t  "),
        ("Single word", "Hello"),
        ("No punctuation", "This text has no sentence ending punctuation"),
    ]

    for name, text in edge_cases:
        sentences = sakurs.split(text)
        print(f"   {name}: {sentences}")
    print()

    # Example 4: Using fallback for language detection
    print("4. Language fallback pattern:")

    def split_with_fallback(text: str, preferred_lang: str = "en") -> list[str]:
        """Split text with automatic fallback to English."""
        try:
            return sakurs.split(text, language=preferred_lang)
        except sakurs.SakursError:
            print(
                f"   Language '{preferred_lang}' not supported, falling back to English"
            )
            return sakurs.split(text, language="en")

    # This will fallback to English
    sentences = split_with_fallback("Hello world.", preferred_lang="es")
    print(f"   Result: {sentences}")
    print()

    # Example 5: Deprecated parameter warning
    print("5. Handling deprecated parameters:")
    processor = sakurs.Processor("en")
    print("   Calling split with deprecated 'threads' parameter...")
    # This will trigger a deprecation warning
    sentences = processor.split("Test text.", threads=4)
    print(f"   Result: {sentences}")
    print("   (Check above for deprecation warning)")


if __name__ == "__main__":
    main()
