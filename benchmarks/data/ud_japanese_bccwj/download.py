#!/usr/bin/env python3
"""Download and process UD Japanese-BCCWJ r2.16 corpus."""

import logging
import sys
import tempfile
from pathlib import Path

import click
import requests
from tqdm import tqdm

logging.basicConfig(level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s")
logger = logging.getLogger(__name__)

# UD r2.16 release information
UD_VERSION = "2.16"
UD_RELEASE_URL = "https://lindat.mff.cuni.cz/repository/xmlui/bitstream/handle/11234/1-5901/ud-treebanks-v2.16.tgz"
JAPANESE_BCCWJ_DIR = "UD_Japanese-BCCWJ"

# Cache directory
CACHE_DIR = Path(__file__).parent / "cache"
CACHE_FILE = CACHE_DIR / "ud_japanese_bccwj.json"


def download_file(url: str, dest_path: Path, desc: str = "Downloading") -> None:
    """Download file with progress bar."""
    response = requests.get(url, stream=True)
    response.raise_for_status()

    total_size = int(response.headers.get("content-length", 0))

    with open(dest_path, "wb") as f:
        with tqdm(total=total_size, unit="B", unit_scale=True, desc=desc) as pbar:
            for chunk in response.iter_content(chunk_size=8192):
                f.write(chunk)
                pbar.update(len(chunk))


def extract_japanese_bccwj(archive_path: Path, extract_to: Path) -> Path:
    """Extract Japanese-BCCWJ from UD archive."""
    logger.info("Extracting UD Japanese-BCCWJ...")

    # First extract the main archive
    import tarfile

    with tarfile.open(archive_path, "r:gz") as tar:
        # Find Japanese-BCCWJ
        japanese_members = [m for m in tar.getmembers() if JAPANESE_BCCWJ_DIR in m.name]

        if not japanese_members:
            raise ValueError(f"Could not find {JAPANESE_BCCWJ_DIR} in archive")

        # Extract only Japanese-BCCWJ files
        for member in tqdm(japanese_members, desc="Extracting"):
            tar.extract(member, extract_to)

    # Find the extracted directory
    extracted_dir = None
    for item in extract_to.iterdir():
        if item.is_dir() and JAPANESE_BCCWJ_DIR in str(item):
            extracted_dir = item / JAPANESE_BCCWJ_DIR
            break

    if not extracted_dir or not extracted_dir.exists():
        # Try alternative path
        extracted_dir = extract_to / JAPANESE_BCCWJ_DIR

    if not extracted_dir.exists():
        raise ValueError(f"Extraction failed: {JAPANESE_BCCWJ_DIR} not found")

    return extracted_dir


def parse_conllu_to_json(conllu_dir: Path) -> dict:
    """Parse CoNLL-U files and convert to JSON format."""
    logger.info("Parsing CoNLL-U files...")

    documents = []
    metadata = {
        "corpus": "UD_Japanese-BCCWJ",
        "version": UD_VERSION,
        "language": "Japanese",
        "license": "CC BY-NC-SA 4.0",
        "note": "Original text not included due to license. See README for details.",
    }

    # Process train, dev, test splits
    for split in ["train", "dev", "test"]:
        conllu_file = conllu_dir / f"ja_bccwj-ud-{split}.conllu"
        if not conllu_file.exists():
            logger.warning(f"File not found: {conllu_file}")
            continue

        logger.info(f"Processing {split} split...")

        with open(conllu_file, encoding="utf-8") as f:
            current_sent_tokens = []
            current_sent_id = None
            doc_sentences = []
            doc_id = None

            for line in f:
                line = line.strip()

                if not line:  # Empty line = sentence boundary
                    if current_sent_tokens:
                        # Create sentence without original text
                        sentence = {
                            "id": current_sent_id,
                            "tokens": current_sent_tokens,
                            "text": "[Text not included - see README]",
                        }
                        doc_sentences.append(sentence)
                        current_sent_tokens = []
                        current_sent_id = None

                elif line.startswith("# sent_id"):
                    current_sent_id = line.split("=", 1)[1].strip()

                elif line.startswith("# newdoc id"):
                    # Save previous document if exists
                    if doc_sentences:
                        documents.append(
                            {
                                "id": doc_id,
                                "split": split,
                                "sentences": doc_sentences,
                                "text": "[Text not included - see README]",
                            }
                        )
                    doc_sentences = []
                    doc_id = line.split("=", 1)[1].strip()

                elif not line.startswith("#"):
                    # Token line
                    parts = line.split("\t")
                    if len(parts) >= 10 and "-" not in parts[0]:
                        token = {
                            "id": parts[0],
                            "form": parts[1],
                            "lemma": parts[2],
                            "pos": parts[3],
                            "features": parts[5],
                        }
                        current_sent_tokens.append(token)

            # Don't forget last document
            if doc_sentences:
                documents.append(
                    {
                        "id": doc_id,
                        "split": split,
                        "sentences": doc_sentences,
                        "text": "[Text not included - see README]",
                    }
                )

    return {"metadata": metadata, "documents": documents}


def create_sample_data() -> dict:
    """Create a small sample for testing when full corpus is not available."""
    return {
        "metadata": {
            "corpus": "UD_Japanese-BCCWJ",
            "version": UD_VERSION,
            "language": "Japanese",
            "license": "CC BY-NC-SA 4.0",
            "note": "Sample data for testing",
        },
        "documents": [
            {
                "id": "sample_001",
                "split": "test",
                "text": "これはテストです。日本語の文分割をテストしています。",
                "sentences": [
                    {"id": "sample_001_s1", "text": "これはテストです。", "tokens": []},
                    {
                        "id": "sample_001_s2",
                        "text": "日本語の文分割をテストしています。",
                        "tokens": [],
                    },
                ],
            }
        ],
    }


@click.command()
@click.option("--output", "-o", type=click.Path(), default=str(CACHE_FILE), help="Output file path")
@click.option("--force", "-f", is_flag=True, help="Force re-download even if cache exists")
@click.option(
    "--merge-bccwj",
    type=click.Path(exists=True),
    help="Path to BCCWJ core_SUW.txt for merging original text",
)
def main(output, force, merge_bccwj):
    """Download and process UD Japanese-BCCWJ corpus."""
    output_path = Path(output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Check cache
    if output_path.exists() and not force:
        logger.info(f"Corpus already downloaded: {output_path}")
        logger.info("Use --force to re-download")
        return

    # Download UD release
    with tempfile.TemporaryDirectory() as tmpdir:
        tmpdir = Path(tmpdir)
        archive_path = tmpdir / "ud-treebanks-v2.16.tgz"

        logger.info(f"Downloading UD {UD_VERSION} release...")
        logger.info("This may take several minutes (file size ~625MB)")

        try:
            download_file(UD_RELEASE_URL, archive_path, "Downloading UD release")
        except Exception as e:
            logger.error(f"Download failed: {e}")
            logger.info("Please check your internet connection and try again")
            sys.exit(1)

        # Extract Japanese-BCCWJ
        try:
            japanese_dir = extract_japanese_bccwj(archive_path, tmpdir)
        except Exception as e:
            logger.error(f"Extraction failed: {e}")
            sys.exit(1)

        # Parse CoNLL-U files
        corpus_data = parse_conllu_to_json(japanese_dir)

        # Merge with BCCWJ if provided
        if merge_bccwj:
            logger.info(f"Merging with BCCWJ from {merge_bccwj}")
            # This would require the merge script from the official repo
            logger.warning("BCCWJ merging not yet implemented")
            logger.info("Please use the official merge script from the UD repo")

        # Save to cache
        import json

        with open(output_path, "w", encoding="utf-8") as f:
            json.dump(corpus_data, f, ensure_ascii=False, indent=2)

        logger.info(f"Corpus saved to {output_path}")
        logger.info(f"Total documents: {len(corpus_data['documents'])}")

        # Print statistics
        splits = {}
        for doc in corpus_data["documents"]:
            split = doc["split"]
            splits[split] = splits.get(split, 0) + 1

        logger.info("Split distribution:")
        for split, count in splits.items():
            logger.info(f"  {split}: {count} documents")


if __name__ == "__main__":
    main()
