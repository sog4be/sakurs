#!/usr/bin/env python3
"""Example of using custom language configurations with Sakurs."""

from pathlib import Path

import sakurs


def create_minimal_config() -> None:
    """Create a minimal language configuration."""
    print("=== Minimal Language Config ===")

    # Create the most basic configuration
    config = sakurs.LanguageConfig(
        metadata=sakurs.MetadataConfig("minimal", "Minimal Language"),
        terminators=sakurs.TerminatorConfig(["."]),  # Only period as terminator
        ellipsis=sakurs.EllipsisConfig(),
        enclosures=sakurs.EnclosureConfig([]),
        suppression=sakurs.SuppressionConfig(),
        abbreviations=sakurs.AbbreviationConfig(),
        sentence_starters=None,
    )

    # Test with simple text
    text = "Hello world. This is a test! Will this split?"
    sentences = sakurs.split(text, language_config=config)

    print(f"Text: {text}")
    print(f"Sentences: {sentences}")
    print()


def create_custom_abbreviations() -> None:
    """Create a config with custom abbreviations."""
    print("=== Custom Abbreviations ===")

    # Create config with medical abbreviations
    config = sakurs.LanguageConfig(
        metadata=sakurs.MetadataConfig("medical", "Medical English"),
        terminators=sakurs.TerminatorConfig([".", "!", "?"]),
        ellipsis=sakurs.EllipsisConfig(),
        enclosures=sakurs.EnclosureConfig(
            [
                sakurs.EnclosurePair("(", ")"),
            ]
        ),
        suppression=sakurs.SuppressionConfig(),
        abbreviations=sakurs.AbbreviationConfig(
            medical=["Dr", "M.D", "Ph.D", "R.N"],
            units=["mg", "ml", "cc", "i.v"],
            common=["etc", "vs", "e.g", "i.e"],
        ),
        sentence_starters=None,
    )

    # Test with medical text
    text = "Dr. Smith prescribed 50mg i.v. The patient (R.N. Jones) agreed. Treatment vs. placebo?"
    sentences = sakurs.split(text, language_config=config)

    print(f"Text: {text}")
    print("Sentences:")
    for i, sent in enumerate(sentences, 1):
        print(f"  {i}. {sent}")
    print()


def create_programming_language_config() -> None:
    """Create a config for processing programming documentation."""
    print("=== Programming Documentation Config ===")

    # Create config optimized for code documentation
    config = sakurs.LanguageConfig(
        metadata=sakurs.MetadataConfig("code_docs", "Code Documentation"),
        terminators=sakurs.TerminatorConfig(
            chars=[".", "!", "?"],
            patterns=[
                sakurs.TerminatorPattern("...", "ellipsis_terminator"),
            ],
        ),
        ellipsis=sakurs.EllipsisConfig(
            treat_as_boundary=False,  # Don't treat ... as boundary in code
            patterns=["...", "…"],
        ),
        enclosures=sakurs.EnclosureConfig(
            [
                sakurs.EnclosurePair("(", ")"),
                sakurs.EnclosurePair("[", "]"),
                sakurs.EnclosurePair("{", "}"),
                sakurs.EnclosurePair('"', '"', symmetric=True),
                sakurs.EnclosurePair("'", "'", symmetric=True),
            ]
        ),
        suppression=sakurs.SuppressionConfig(
            fast_patterns=[
                sakurs.FastPattern(
                    ".", before="digit", after="digit"
                ),  # Decimal numbers
            ]
        ),
        abbreviations=sakurs.AbbreviationConfig(
            languages=["C", "C++", "C#", "F#"],
            common=["e.g", "i.e", "etc", "vs"],
            file_extensions=["js", "py", "rs", "go", "cpp"],
        ),
        sentence_starters=None,
    )

    # Test with code documentation
    text = """The function returns 3.14 (a float). Use array[0] to access.
Languages like C++ and F# are supported. See main.py for examples.
The range is [0...100]. Call foo() vs. bar()!"""

    sentences = sakurs.split(text, language_config=config)

    print(f"Text: {text}")
    print("\nSentences:")
    for i, sent in enumerate(sentences, 1):
        print(f"  {i}. {sent.strip()}")
    print()


def save_and_load_config() -> None:
    """Demonstrate saving and loading configurations."""
    print("=== Save and Load Config ===")

    # Create a custom config
    config = sakurs.LanguageConfig(
        metadata=sakurs.MetadataConfig("legal", "Legal English"),
        terminators=sakurs.TerminatorConfig([".", ";", ":"]),
        ellipsis=sakurs.EllipsisConfig(),
        enclosures=sakurs.EnclosureConfig(
            [
                sakurs.EnclosurePair("(", ")"),
                sakurs.EnclosurePair('"', '"'),
            ]
        ),
        suppression=sakurs.SuppressionConfig(),
        abbreviations=sakurs.AbbreviationConfig(
            legal=["Inc", "Corp", "Ltd", "LLC"],
            titles=["Hon", "Esq"],
            common=["v", "vs", "et al"],
        ),
        sentence_starters=sakurs.SentenceStarterConfig(
            legal_terms=["Whereas", "Therefore", "Provided"],
            common=["The", "This", "That"],
        ),
    )

    # Save to temporary file
    config_path = Path("legal_english.toml")
    config.to_toml(config_path)
    print(f"Saved config to: {config_path}")

    # Load it back
    loaded_config = sakurs.LanguageConfig.from_toml(config_path)
    print(f"Loaded config: {loaded_config}")

    # Test with legal text
    text = (
        "Apple Inc. vs. Samsung Corp.; The case proceeds. Whereas: Both parties agreed."
    )
    sentences = sakurs.split(text, language_config=loaded_config)

    print(f"\nText: {text}")
    print("Sentences:")
    for i, sent in enumerate(sentences, 1):
        print(f"  {i}. {sent.strip()}")

    # Clean up
    config_path.unlink()
    print("\nCleaned up temporary file")


def main() -> None:
    """Run all examples."""
    create_minimal_config()
    create_custom_abbreviations()
    create_programming_language_config()
    save_and_load_config()


if __name__ == "__main__":
    main()
