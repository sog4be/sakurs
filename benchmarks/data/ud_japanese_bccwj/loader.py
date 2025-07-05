"""Loader for UD Japanese-BCCWJ corpus."""

import json
import logging
import sys
from collections.abc import Iterator
from pathlib import Path

# Add parent directory for base_loader import
sys.path.insert(0, str(Path(__file__).parent.parent))
from base_loader import ConllULoader

logger = logging.getLogger(__name__)


class UDJapaneseBCCWJLoader(ConllULoader):
    """Loader for UD Japanese-BCCWJ corpus."""

    def __init__(self, cache_dir: Path | None = None):
        """Initialize the loader."""
        super().__init__(cache_dir)
        self.corpus_name = "UD_Japanese-BCCWJ"
        self.cache_file = self.cache_dir / "ud_japanese_bccwj.json"

    def is_downloaded(self) -> bool:
        """Check if corpus data is downloaded."""
        return self.cache_file.exists()

    def download(self, force: bool = False) -> Path:
        """Download corpus data.

        Args:
            force: Force re-download even if cached

        Returns:
            Path to downloaded/cached data
        """
        if self.is_downloaded() and not force:
            logger.info(f"Using cached data: {self.cache_file}")
            return self.cache_file

        # Run download script
        import subprocess

        download_script = Path(__file__).parent / "download.py"

        cmd = [sys.executable, str(download_script)]
        if force:
            cmd.append("--force")

        logger.info("Downloading UD Japanese-BCCWJ...")
        result = subprocess.run(cmd, capture_output=True, text=True)

        if result.returncode != 0:
            raise RuntimeError(f"Download failed: {result.stderr}")

        return self.cache_file

    def load(self) -> dict:
        """Load corpus data.

        Returns:
            Dictionary with corpus data and metadata
        """
        if not self.is_downloaded():
            logger.info("Corpus not found, downloading...")
            self.download()

        with open(self.cache_file, encoding="utf-8") as f:
            return json.load(f)

    def load_sample(self) -> dict:
        """Load a small sample for testing."""
        # Try to load from cache first
        if self.is_downloaded():
            data = self.load()
            # Return first few documents
            sample = {
                "metadata": data["metadata"],
                "documents": data["documents"][:5],  # First 5 documents
            }
            return sample

        # Otherwise return hardcoded sample
        return {
            "metadata": {
                "name": "UD_Japanese-BCCWJ",
                "version": "2.16",
                "url": "https://github.com/UniversalDependencies/UD_Japanese-BCCWJ",
                "license": "CC BY-NC-SA 4.0",
                "sentences": 2,
                "tokens": 20,
                "documents": 1,
            },
            "documents": [{
                "id": "sample_doc",
                "text": "今日は良い天気です。公園で散歩しました。",
                "sentences": [
                    {"text": "今日は良い天気です。", "id": "1"},
                    {"text": "公園で散歩しました。", "id": "2"}
                ],
                "split": "sample"
            }]
        }

    def iter_documents(self) -> Iterator[tuple[str, list[str]]]:
        """Iterate over documents with ground truth.

        Yields:
            Tuple of (document_text, sentence_list)
        """
        data = self.load()

        for doc in data["documents"]:
            # Extract sentences
            sentences = []

            # Check if we have original text
            if doc["text"] == "[Text not included - see README]":
                # Reconstruct from tokens if possible
                for sent in doc["sentences"]:
                    if "tokens" in sent and sent["tokens"]:
                        # Join token forms
                        text = "".join(token["form"] for token in sent["tokens"])
                        sentences.append(text)
                    else:
                        # No tokens available
                        sentences.append(sent.get("text", ""))
            else:
                # Use provided sentences
                sentences = [sent["text"] for sent in doc["sentences"]]

            # Create document text
            doc_text = "".join(sentences)

            yield doc_text, sentences

    def to_plain_text(self, output_path: Path) -> Path:
        """Convert to plain text format.

        Args:
            output_path: Path to save plain text

        Returns:
            Path to output file
        """
        with open(output_path, "w", encoding="utf-8") as f:
            for doc_text, _ in self.iter_documents():
                f.write(doc_text + "\n\n")

        return output_path

    def to_sentences_file(self, output_path: Path) -> Path:
        """Convert to one-sentence-per-line format.

        Args:
            output_path: Path to save sentences

        Returns:
            Path to output file
        """
        with open(output_path, "w", encoding="utf-8") as f:
            for _, sentences in self.iter_documents():
                for sent in sentences:
                    if sent.strip():  # Skip empty sentences
                        f.write(sent.strip() + "\n")

        return output_path

    def get_statistics(self) -> dict[str, int]:
        """Get corpus statistics with Japanese-specific metrics."""
        stats = super().get_statistics()

        # Add Japanese-specific statistics
        total_chars = 0
        hiragana_count = 0
        katakana_count = 0
        kanji_count = 0

        for doc_text, _ in self.iter_documents():
            total_chars += len(doc_text)
            for char in doc_text:
                if "\u3040" <= char <= "\u309f":
                    hiragana_count += 1
                elif "\u30a0" <= char <= "\u30ff":
                    katakana_count += 1
                elif "\u4e00" <= char <= "\u9fff":
                    kanji_count += 1

        stats.update(
            {
                "total_characters": total_chars,
                "hiragana_count": hiragana_count,
                "katakana_count": katakana_count,
                "kanji_count": kanji_count,
                "hiragana_ratio": hiragana_count / total_chars if total_chars > 0 else 0,
                "katakana_ratio": katakana_count / total_chars if total_chars > 0 else 0,
                "kanji_ratio": kanji_count / total_chars if total_chars > 0 else 0,
            }
        )

        return stats
