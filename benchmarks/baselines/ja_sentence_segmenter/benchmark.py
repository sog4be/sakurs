#!/usr/bin/env python3
"""CLI benchmark interface for ja_sentence_segmenter."""

import sys
import logging
import time
from pathlib import Path
import click

from .segmenter import JapaneseSentenceSegmenter

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


@click.command()
@click.argument('input_file', type=click.Path(exists=True))
@click.option('--output', '-o', type=click.Path(), help='Output file (default: stdout)')
@click.option('--format', '-f', type=click.Choice(['lines', 'json']), default='lines',
              help='Output format')
@click.option('--encoding', '-e', default='utf-8', help='Text encoding')
@click.option('--time', '-t', is_flag=True, help='Print timing information')
def main(input_file, output, format, encoding, time):
    """Segment Japanese text using ja_sentence_segmenter.
    
    This CLI interface is designed to be compatible with sakurs benchmarking.
    """
    # Initialize segmenter
    segmenter = JapaneseSentenceSegmenter()
    
    if not segmenter.is_available():
        logger.error("ja_sentence_segmenter not installed")
        logger.error("Install with: pip install ja-sentence-segmenter")
        sys.exit(1)
    
    # Read input file
    try:
        with open(input_file, 'r', encoding=encoding) as f:
            text = f.read()
    except Exception as e:
        logger.error(f"Failed to read input file: {e}")
        sys.exit(1)
    
    # Measure segmentation time
    start_time = time.time() if time else None
    
    try:
        sentences = segmenter.segment(text)
    except Exception as e:
        logger.error(f"Segmentation failed: {e}")
        sys.exit(1)
    
    if time:
        elapsed = time.time() - start_time
        logger.info(f"Segmentation took {elapsed:.3f} seconds")
        logger.info(f"Processed {len(text)} characters")
        logger.info(f"Found {len(sentences)} sentences")
        logger.info(f"Throughput: {len(text) / elapsed:.0f} chars/sec")
    
    # Format output
    if format == 'lines':
        output_text = '\n'.join(sentences)
    else:  # json
        import json
        output_data = {
            'sentences': sentences,
            'count': len(sentences),
            'metadata': {
                'segmenter': 'ja_sentence_segmenter',
                'encoding': encoding
            }
        }
        output_text = json.dumps(output_data, ensure_ascii=False, indent=2)
    
    # Write output
    if output:
        try:
            with open(output, 'w', encoding=encoding) as f:
                f.write(output_text)
                if format == 'lines':
                    f.write('\n')  # Add final newline
        except Exception as e:
            logger.error(f"Failed to write output: {e}")
            sys.exit(1)
    else:
        print(output_text)


if __name__ == '__main__':
    main()