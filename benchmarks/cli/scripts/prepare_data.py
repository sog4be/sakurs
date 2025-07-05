#!/usr/bin/env python3
"""Prepare benchmark data for CLI benchmarks."""

import logging
import sys
from pathlib import Path

import click

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from data.brown_corpus import is_available as brown_available
from data.ud_english_ewt import is_available as ewt_available
from data.ud_english_ewt import load_sample as load_ewt
from data.ud_japanese_bccwj import is_available as bccwj_available
from data.ud_japanese_bccwj import load_sample as load_bccwj
from data.wikipedia import create_loader as create_wikipedia_loader

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)


def prepare_ud_english_ewt():
    """Prepare UD English EWT data."""
    logger.info("Checking UD English EWT data...")

    if not ewt_available():
        logger.error("UD English EWT data not available. Please run:")
        logger.error("  cd ../../data/ud_english_ewt && python download.py")
        return False

    # Create plain text versions for CLI benchmarking
    output_dir = Path(__file__).parent.parent.parent / "data" / "ud_english_ewt" / "cli_format"
    output_dir.mkdir(exist_ok=True)

    # Load and convert to plain text
    data = load_ewt()

    # Count test set sentences
    test_sentence_count = 0
    test_word_count = 0
    
    # Save as plain text (all documents concatenated)
    plain_text_path = output_dir / "ewt_plain.txt"
    with open(plain_text_path, "w", encoding="utf-8") as f:
        for doc in data["documents"]:
            f.write(doc["text"] + "\n\n")
            # Count test set statistics
            if doc.get("split") == "test" or "test" in doc.get("id", "").lower():
                test_sentence_count += len(doc.get("sentences", []))
                test_word_count += len(doc["text"].split())

    # Save ground truth (one sentence per line)
    sentences_path = output_dir / "ewt_sentences.txt"
    with open(sentences_path, "w", encoding="utf-8") as f:
        for doc in data["documents"]:
            for sent in doc["sentences"]:
                # Handle both string and dict formats
                if isinstance(sent, str):
                    f.write(sent + "\n")
                elif isinstance(sent, dict) and "text" in sent:
                    f.write(sent["text"] + "\n")

    # Display dataset statistics
    logger.info(f"UD English EWT prepared: {plain_text_path}")
    if data.get("metadata"):
        logger.info(f"  Version: {data['metadata'].get('version', 'Unknown')}")
        logger.info(f"  Total sentences: {data['metadata'].get('sentences', 'Unknown')}")
        
    # If we couldn't get test set info from documents, try from metadata
    if test_sentence_count == 0 and "metadata" in data:
        # For simple loader, we might not have split information
        # Display total stats instead
        logger.info(f"  Dataset info: {data['metadata'].get('sentences', 'Unknown')} sentences")
    else:
        logger.info(f"  Test set: {test_sentence_count} sentences, {test_word_count} words")
    
    return True


def prepare_brown_corpus():
    """Prepare Brown Corpus data."""
    logger.info("Checking Brown Corpus data...")

    if not brown_available():
        logger.error("Brown Corpus data not available. Please run:")
        logger.error("  cd ../../data/brown_corpus && make download")
        return False

    # Brown corpus is already in the right format
    logger.info("Brown Corpus data available")
    return True


def prepare_wikipedia_samples():
    """Prepare Wikipedia samples using Hugging Face datasets."""
    logger.info("Preparing Wikipedia samples...")

    success = True
    date = "20231101"  # Using November 2023 dump available from Hugging Face

    # Prepare English Wikipedia
    try:
        logger.info("Checking English Wikipedia sample...")
        en_loader = create_wikipedia_loader("en", size_mb=500, date=date)

        if not en_loader.is_cached():
            logger.info("Creating English Wikipedia sample from Hugging Face dataset...")
            logger.info("This may take a while on first run, but will be cached for future use.")
            en_loader.download()
            logger.info("English Wikipedia sample ready")
        else:
            logger.info("English Wikipedia sample already cached")

    except Exception as e:
        logger.error(f"Failed to prepare English Wikipedia: {e}")
        logger.error("Make sure 'datasets' package is installed: pip install datasets>=2.14.0")
        success = False

    # Prepare Japanese Wikipedia
    try:
        logger.info("Checking Japanese Wikipedia sample...")
        ja_loader = create_wikipedia_loader("ja", size_mb=500, date=date)

        if not ja_loader.is_cached():
            logger.info("Creating Japanese Wikipedia sample from Hugging Face dataset...")
            logger.info("This may take a while on first run, but will be cached for future use.")
            ja_loader.download()
            logger.info("Japanese Wikipedia sample ready")
        else:
            logger.info("Japanese Wikipedia sample already cached")

    except Exception as e:
        logger.error(f"Failed to prepare Japanese Wikipedia: {e}")
        logger.error("Make sure 'datasets' package is installed: pip install datasets>=2.14.0")
        success = False

    return success


def prepare_japanese_data():
    """Prepare Japanese data."""
    logger.info("Checking UD Japanese-BCCWJ data...")

    if not bccwj_available():
        logger.error("UD Japanese-BCCWJ data not available. Please run:")
        logger.error("  cd ../../data/ud_japanese_bccwj && python download.py")
        logger.info("Note: Original text may not be included due to licensing")
        return False

    # Create plain text versions for CLI benchmarking
    output_dir = Path(__file__).parent.parent.parent / "data" / "ud_japanese_bccwj" / "cli_format"
    output_dir.mkdir(exist_ok=True)

    try:
        # Load and convert to plain text
        data = load_bccwj()

        # Count test set sentences
        test_sentence_count = 0
        test_char_count = 0
        total_sentence_count = 0

        # Save as plain text (all documents concatenated)
        plain_text_path = output_dir / "bccwj_plain.txt"
        with open(plain_text_path, "w", encoding="utf-8") as f:
            for doc in data["documents"]:
                # Note: text might be reconstructed from tokens
                doc_text = doc.get("text", "")
                if doc_text and doc_text != "[Text not included - see README]":
                    f.write(doc_text + "\n\n")
                else:
                    # Reconstruct from sentences if available
                    for sent in doc.get("sentences", []):
                        sent_text = sent.get("text", "")
                        if sent_text:
                            f.write(sent_text)
                    f.write("\n\n")
                
                # Count statistics
                sentences = doc.get("sentences", [])
                total_sentence_count += len(sentences)
                if doc.get("split") == "test":
                    test_sentence_count += len(sentences)
                    for sent in sentences:
                        sent_text = sent.get("text", "")
                        if sent_text and sent_text != "[Text not included - see README]":
                            test_char_count += len(sent_text)

        # Save ground truth (one sentence per line)
        sentences_path = output_dir / "bccwj_sentences.txt"
        with open(sentences_path, "w", encoding="utf-8") as f:
            for doc in data["documents"]:
                for sent in doc.get("sentences", []):
                    sent_text = sent.get("text", "")
                    if sent_text and sent_text.strip():
                        f.write(sent_text.strip() + "\n")

        logger.info(f"UD Japanese-BCCWJ prepared: {plain_text_path}")
        if data.get("metadata"):
            logger.info(f"  Version: {data['metadata'].get('version', 'Unknown')}")
            logger.info(f"  Total documents: {len(data.get('documents', []))}")
            logger.info(f"  Total sentences: {total_sentence_count}")
        
        if test_sentence_count > 0:
            logger.info(f"  Test set: {test_sentence_count} sentences, {test_char_count} characters")
        
        logger.warning("Note: Text may be reconstructed from tokens if original is not available")
        return True

    except Exception as e:
        logger.error(f"Failed to prepare Japanese data: {e}")
        return False


@click.command()
@click.option("--force", is_flag=True, help="Force data preparation even if exists")
def main(force):
    """Prepare all benchmark data."""
    logger.info("Preparing benchmark data...")

    success = True

    # Prepare English data
    if not prepare_ud_english_ewt():
        success = False

    if not prepare_brown_corpus():
        success = False

    # Prepare Wikipedia samples
    if not prepare_wikipedia_samples():
        success = False

    # Prepare Japanese data (Phase 2)
    if not prepare_japanese_data():
        logger.warning("Japanese data not ready (Phase 2)")

    if success:
        logger.info("Data preparation complete!")
    else:
        logger.error("Some data preparation failed")
        sys.exit(1)


if __name__ == "__main__":
    main()
