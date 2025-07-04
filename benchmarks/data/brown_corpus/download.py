#!/usr/bin/env python3
"""Download and process Brown Corpus data for sakurs benchmarks."""

import json
import sys
from pathlib import Path

import click
import nltk
from tqdm import tqdm

# Add parent directory to path to import schema
sys.path.insert(0, str(Path(__file__).parent.parent))
from schema import validate_corpus_data


def ensure_nltk_data() -> None:
    """Ensure NLTK Brown Corpus data is downloaded."""
    try:
        nltk.data.find("corpora/brown")
        click.echo("✓ Brown Corpus already downloaded")
    except LookupError:
        click.echo("📥 Downloading Brown Corpus...")
        nltk.download("brown", quiet=True)
        click.echo("✓ Brown Corpus downloaded successfully")


def extract_sentences_and_boundaries() -> tuple[str, list[int]]:
    """Extract sentences and their boundaries from Brown Corpus.

    Returns:
        Tuple of (text, boundaries) where boundaries are character positions
        after each sentence (including space).
    """
    from nltk.corpus import brown

    click.echo("🔍 Extracting sentences from Brown Corpus...")

    # Get all sentences
    sentences = brown.sents()

    # Build text with boundary tracking
    text_parts = []
    boundaries = []
    current_pos = 0

    # Use tqdm for progress bar
    for sent in tqdm(sentences, desc="Processing sentences"):
        # Join words with spaces and add trailing space
        sent_text = " ".join(sent) + " "
        text_parts.append(sent_text)
        current_pos += len(sent_text)
        # Boundary should be at the space after punctuation (not the next character)
        # Sakurs detects boundaries at the space position, not the next character
        boundaries.append(current_pos - 1)

    # Combine all text
    full_text = "".join(text_parts)

    # Remove the last boundary (end of text)
    # The last boundary points past the end of text, so remove it
    if boundaries:
        boundaries.pop()

    return full_text, boundaries


def save_corpus_data(text: str, boundaries: list[int], output_path: Path) -> None:
    """Save corpus data in sakurs-compatible JSON format."""
    # Calculate metadata
    metadata = {
        "source": "Brown Corpus",
        "sentences": len(boundaries),
        "characters": len(text),
        "words": len(text.split()),
    }

    # Create data structure
    corpus_data = {
        "name": "brown_corpus_full",
        "text": text,
        "boundaries": boundaries,
        "metadata": metadata,
    }

    # Validate before saving
    try:
        validate_corpus_data(corpus_data)
        click.echo("✅ Data validation passed")
    except ValueError as e:
        click.echo(f"❌ Data validation failed: {e}", err=True)
        sys.exit(1)

    # Save to JSON
    click.echo(f"💾 Saving to {output_path}...")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(corpus_data, f, ensure_ascii=False, indent=2)

    # Print summary
    click.echo("\n✨ Processing complete!")
    click.echo(f"   Sentences: {metadata['sentences']:,}")
    click.echo(f"   Characters: {metadata['characters']:,}")
    click.echo(f"   Words: {metadata['words']:,}")
    click.echo(f"   Output: {output_path}")


@click.command()
@click.option(
    "--output",
    "-o",
    type=click.Path(path_type=Path),
    default="cache/brown_corpus.json",
    help="Output file path",
)
@click.option(
    "--force",
    "-f",
    is_flag=True,
    help="Force re-download and re-process even if cache exists",
)
def main(output: Path, force: bool) -> None:
    """Download and process Brown Corpus for sakurs benchmarks."""
    # Ensure output directory exists
    output.parent.mkdir(parents=True, exist_ok=True)

    # Check if already exists
    if output.exists() and not force:
        click.echo(f"✓ Cached data already exists at {output}")
        click.echo("  Use --force to re-process")
        return

    try:
        # Download NLTK data if needed
        ensure_nltk_data()

        # Extract sentences and boundaries
        text, boundaries = extract_sentences_and_boundaries()

        # Save processed data
        save_corpus_data(text, boundaries, output)

    except Exception as e:
        click.echo(f"❌ Error: {e}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
