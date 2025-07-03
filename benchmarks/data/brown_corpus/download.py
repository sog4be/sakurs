#!/usr/bin/env python3
"""Download and process Brown Corpus data for sakurs benchmarks."""

import json
import os
import sys
from pathlib import Path
from typing import List, Tuple

import click
import nltk
from tqdm import tqdm


def ensure_nltk_data() -> None:
    """Ensure NLTK Brown Corpus data is downloaded."""
    try:
        nltk.data.find("corpora/brown")
        click.echo("✓ Brown Corpus already downloaded")
    except LookupError:
        click.echo("📥 Downloading Brown Corpus...")
        nltk.download("brown", quiet=True)
        click.echo("✓ Brown Corpus downloaded successfully")


def extract_sentences_and_boundaries() -> Tuple[str, List[int]]:
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
        boundaries.append(current_pos)
    
    # Combine all text
    full_text = "".join(text_parts)
    
    # Remove the last boundary (end of text)
    if boundaries and boundaries[-1] == len(full_text):
        boundaries.pop()
    
    return full_text, boundaries


def save_corpus_data(text: str, boundaries: List[int], output_path: Path) -> None:
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
    
    # Save to JSON
    click.echo(f"💾 Saving to {output_path}...")
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(corpus_data, f, ensure_ascii=False, indent=2)
    
    # Print summary
    click.echo(f"\n✨ Processing complete!")
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