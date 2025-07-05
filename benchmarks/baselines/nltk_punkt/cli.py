#!/usr/bin/env python3
"""CLI interface for NLTK Punkt sentence tokenizer that's compatible with sakurs benchmarks."""

import sys

import click

from .segmenter import create_segmenter


@click.command()
@click.option(
    "--input", "-i", type=click.Path(), help="Input file (use '-' for stdin, default: stdin)"
)
@click.option(
    "--output", "-o", type=click.Path(), help="Output file (default: stdout)"
)
@click.option("--encoding", "-e", default="utf-8", help="Text encoding")
@click.option("--format", "-f", type=click.Choice(["lines", "json"]), default="lines", help="Output format")
def main(input, output, encoding, format):
    """Segment text using NLTK Punkt tokenizer.
    
    This CLI interface is designed to be compatible with sakurs benchmarking.
    It reads text from a file or stdin and outputs sentences one per line.
    """
    # Initialize segmenter
    segmenter = create_segmenter()
    
    # Read input
    if input and input != '-':
        with open(input, encoding=encoding) as f:
            text = f.read()
    else:
        text = sys.stdin.read()
    
    # Segment text
    sentences = segmenter.extract_sentences(text)
    
    # Format output
    if format == "lines":
        output_text = "\n".join(sentences)
    else:  # json
        import json
        output_data = {
            "sentences": sentences,
            "count": len(sentences),
            "metadata": {"segmenter": "nltk_punkt", "encoding": encoding}
        }
        output_text = json.dumps(output_data, ensure_ascii=False, indent=2)
    
    # Write output
    if output:
        with open(output, "w", encoding=encoding) as f:
            f.write(output_text)
            if format == "lines":
                f.write("\n")  # Add final newline
    else:
        print(output_text)


if __name__ == "__main__":
    main()