"""Wikipedia loader using Hugging Face datasets."""

import logging
import sys
from collections.abc import Iterator
from datetime import datetime
from pathlib import Path
from typing import Any

from datasets import IterableDataset, load_dataset

# Add parent directory for base_loader import
sys.path.insert(0, str(Path(__file__).parent.parent))
from base_loader import CorpusLoader

from .version_manager import WikipediaVersionManager

logger = logging.getLogger(__name__)


class WikipediaLoader(CorpusLoader):
    """Loader for Wikipedia data from Hugging Face."""

    # Default to November 2023 snapshot for reproducibility
    # Available dates can be found at: https://huggingface.co/datasets/wikimedia/wikipedia
    DEFAULT_DATE = "20231101"

    def __init__(
        self,
        language: str = "en",
        size_mb: int = 500,
        cache_dir: Path | None = None,
        date: str = None,
    ):
        """Initialize the Wikipedia loader.

        Args:
            language: Language code ('en', 'ja', etc.)
            size_mb: Target sample size in MB
            cache_dir: Cache directory for samples
            date: Wikipedia dump date (default: 20231101)
        """
        super().__init__(cache_dir)
        self.language = language
        self.size_mb = size_mb
        self.date = date or self.DEFAULT_DATE
        self.dataset_name = f"{self.date}.{language}"
        self.sample_file = self.cache_dir / f"wikipedia_{language}_{size_mb}mb_{self.date}.txt"
        self.version_manager = WikipediaVersionManager(self.cache_dir)

    def is_cached(self) -> bool:
        """Check if sample is already cached."""
        return self.sample_file.exists()

    def download(self, force: bool = False) -> Path:
        """Create Wikipedia sample from Hugging Face dataset.

        Args:
            force: Force re-creation even if cached

        Returns:
            Path to prepared sample
        """
        if self.is_cached() and not force:
            logger.info(f"Using cached sample: {self.sample_file}")
            return self.sample_file

        logger.info(f"Creating {self.size_mb}MB sample from HF Wikipedia dataset")
        logger.info(f"Dataset: wikimedia/wikipedia, Configuration: {self.dataset_name}")

        try:
            # Load dataset in streaming mode
            dataset = load_dataset(
                "wikimedia/wikipedia", self.dataset_name, split="train", streaming=True
            )

            # Create sample
            self._create_sample(dataset)

        except Exception as e:
            raise RuntimeError(f"Failed to load Wikipedia dataset: {e}")

        return self.sample_file

    def _create_sample(self, dataset: IterableDataset):
        """Create fixed-size sample from streaming dataset."""
        target_size = self.size_mb * 1024 * 1024
        current_size = 0
        article_count = 0

        self.sample_file.parent.mkdir(parents=True, exist_ok=True)

        with open(self.sample_file, "w", encoding="utf-8") as f:
            for article in dataset:
                # Skip very short articles
                if len(article["text"]) < 100:
                    continue

                # Write article with clear boundaries
                title = article["title"]
                text = article["text"]

                article_header = f"===== Article {article_count + 1}: {title} =====\n\n"
                article_content = text + "\n\n"

                # Check if adding this article would exceed target size
                article_bytes = len((article_header + article_content).encode("utf-8"))
                if current_size + article_bytes > target_size and article_count > 0:
                    break

                f.write(article_header)
                f.write(article_content)

                current_size += article_bytes
                article_count += 1

                if article_count % 100 == 0:
                    logger.info(
                        f"Processed {article_count} articles, {current_size / 1024 / 1024:.1f}MB"
                    )

        actual_size_mb = current_size / 1024 / 1024
        logger.info(f"Created sample: {article_count} articles, {actual_size_mb:.1f}MB")

        # Save metadata
        self.version_manager.save_metadata(
            language=self.language,
            size_mb=self.size_mb,
            date=self.date,
            article_count=article_count,
            actual_size_mb=actual_size_mb,
            download_timestamp=datetime.now(),
            additional_info={
                "dataset_source": "Hugging Face wikimedia/wikipedia",
                "sample_file": str(self.sample_file.name),
            },
        )

    def load(self) -> dict[str, Any]:
        """Load the Wikipedia sample.

        Returns:
            Dictionary with sample data and metadata
        """
        if not self.is_cached():
            logger.info("Sample not found, creating...")
            self.download()

        # Read the sample file
        with open(self.sample_file, encoding="utf-8") as f:
            content = f.read()

        # Split into articles
        articles = []
        current_article = {"title": "", "text": ""}

        for line in content.split("\n"):
            if line.startswith("===== Article"):
                if current_article["text"]:
                    articles.append(current_article)
                # Extract title from header
                title = line.split(":", 1)[1].strip().rstrip(" =====") if ":" in line else "Unknown"
                current_article = {"title": title, "text": ""}
            else:
                current_article["text"] += line + "\n"

        if current_article["text"]:
            articles.append(current_article)

        # Load version metadata if available
        version_metadata = self.version_manager.load_metadata(
            self.language, self.size_mb, self.date
        )

        return {
            "metadata": {
                "corpus": f"Wikipedia-{self.language.upper()}",
                "language": self.language,
                "size_mb": self.size_mb,
                "articles": len(articles),
                "date": self.date,
                "source": "Hugging Face wikimedia/wikipedia",
                "version_info": version_metadata,
            },
            "articles": articles,
        }

    def iter_documents(self) -> Iterator[tuple[str, list[str]]]:
        """Iterate over articles.

        Note: Wikipedia samples don't have sentence boundaries,
        so we return each article as a single "sentence".

        Yields:
            Tuple of (article_text, [article_text])
        """
        data = self.load()

        for article in data["articles"]:
            text = article["text"].strip()
            if text:
                # Return article as both document and single sentence
                # Real sentence segmentation will be done by the benchmarked tools
                yield text, [text]

    def to_plain_text(self, output_path: Path) -> Path:
        """Copy sample to output path.

        Args:
            output_path: Path to save plain text

        Returns:
            Path to output file
        """
        if not self.is_cached():
            self.download()

        # Copy the sample file
        import shutil

        shutil.copy2(self.sample_file, output_path)

        return output_path

    def to_sentences_file(self, output_path: Path) -> Path:
        """Convert to one-article-per-line format.

        Args:
            output_path: Path to save sentences

        Returns:
            Path to output file
        """
        data = self.load()

        with open(output_path, "w", encoding="utf-8") as f:
            for article in data["articles"]:
                text = article["text"].strip()
                if text:
                    # Replace newlines with spaces for one-line format
                    one_line = " ".join(text.split())
                    f.write(one_line + "\n")

        return output_path

    def get_statistics(self) -> dict[str, Any]:
        """Get Wikipedia sample statistics."""
        data = self.load()

        total_chars = sum(len(article["text"]) for article in data["articles"])
        total_words = sum(len(article["text"].split()) for article in data["articles"])

        stats = {
            "corpus": data["metadata"]["corpus"],
            "language": self.language,
            "size_mb": self.size_mb,
            "articles": len(data["articles"]),
            "total_characters": total_chars,
            "total_words": total_words,
            "avg_article_length": total_chars / len(data["articles"]) if data["articles"] else 0,
            "date": self.date,
            "source": "Hugging Face wikimedia/wikipedia",
        }

        # Language-specific stats
        if self.language == "ja":
            # Count character types for Japanese
            hiragana = katakana = kanji = 0
            for article in data["articles"]:
                for char in article["text"]:
                    if "\u3040" <= char <= "\u309f":
                        hiragana += 1
                    elif "\u30a0" <= char <= "\u30ff":
                        katakana += 1
                    elif "\u4e00" <= char <= "\u9fff":
                        kanji += 1

            stats.update(
                {"hiragana_count": hiragana, "katakana_count": katakana, "kanji_count": kanji}
            )

        return stats
