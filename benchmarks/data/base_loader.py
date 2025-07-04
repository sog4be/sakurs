"""Base loader interface for unified corpus handling."""

import logging
from abc import ABC, abstractmethod
from collections.abc import Iterator
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)


class CorpusLoader(ABC):
    """Abstract base class for corpus loaders."""

    def __init__(self, cache_dir: Path | None = None):
        """Initialize loader with optional cache directory."""
        self.cache_dir = cache_dir or Path(__file__).parent / "cache"
        self.cache_dir.mkdir(parents=True, exist_ok=True)

    @abstractmethod
    def download(self, force: bool = False) -> Path:
        """Download corpus data.

        Args:
            force: Force re-download even if cached

        Returns:
            Path to downloaded/cached data
        """
        pass

    @abstractmethod
    def load(self) -> dict[str, Any]:
        """Load corpus data.

        Returns:
            Dictionary with corpus data and metadata
        """
        pass

    @abstractmethod
    def iter_documents(self) -> Iterator[tuple[str, list[str]]]:
        """Iterate over documents with ground truth.

        Yields:
            Tuple of (document_text, sentence_list)
        """
        pass

    @abstractmethod
    def to_plain_text(self, output_path: Path) -> Path:
        """Convert to plain text format.

        Args:
            output_path: Path to save plain text

        Returns:
            Path to output file
        """
        pass

    @abstractmethod
    def to_sentences_file(self, output_path: Path) -> Path:
        """Convert to one-sentence-per-line format.

        Args:
            output_path: Path to save sentences

        Returns:
            Path to output file
        """
        pass

    def get_statistics(self) -> dict[str, int]:
        """Get corpus statistics.

        Returns:
            Dictionary with statistics (documents, sentences, tokens)
        """
        doc_count = 0
        sent_count = 0
        token_count = 0

        for doc_text, sentences in self.iter_documents():
            doc_count += 1
            sent_count += len(sentences)
            token_count += len(doc_text.split())

        return {"documents": doc_count, "sentences": sent_count, "tokens": token_count}

    def validate(self) -> bool:
        """Validate corpus integrity.

        Returns:
            True if valid, False otherwise
        """
        try:
            stats = self.get_statistics()
            if stats["documents"] == 0 or stats["sentences"] == 0:
                logger.error("Empty corpus detected")
                return False
            return True
        except Exception as e:
            logger.error(f"Validation failed: {e}")
            return False


class ConllULoader(CorpusLoader):
    """Base loader for CoNLL-U format corpora."""

    def parse_conllu(self, file_path: Path) -> list[tuple[str, list[str]]]:
        """Parse CoNLL-U format file.

        Args:
            file_path: Path to CoNLL-U file

        Returns:
            List of (document_text, sentence_list) tuples
        """
        documents = []
        current_sentences = []
        current_tokens = []

        with open(file_path, encoding="utf-8") as f:
            for line in f:
                line = line.strip()

                if not line:  # Empty line = sentence boundary
                    if current_tokens:
                        sentence = " ".join(current_tokens)
                        current_sentences.append(sentence)
                        current_tokens = []

                elif line.startswith("#"):  # Comment line
                    if line.startswith("# newdoc"):
                        if current_sentences:
                            doc_text = " ".join(current_sentences)
                            documents.append((doc_text, current_sentences))
                            current_sentences = []

                else:  # Token line
                    parts = line.split("\t")
                    if len(parts) >= 2 and "-" not in parts[0]:
                        # Skip multi-word tokens (e.g., "1-2")
                        current_tokens.append(parts[1])  # FORM field

        # Don't forget last sentence/document
        if current_tokens:
            sentence = " ".join(current_tokens)
            current_sentences.append(sentence)

        if current_sentences:
            doc_text = " ".join(current_sentences)
            documents.append((doc_text, current_sentences))

        return documents


class WikipediaLoader(CorpusLoader):
    """Base loader for Wikipedia dump processing."""

    def __init__(self, language: str, size_mb: int = 500, **kwargs):
        """Initialize Wikipedia loader.

        Args:
            language: Language code (en, ja, etc.)
            size_mb: Target size in megabytes
        """
        super().__init__(**kwargs)
        self.language = language
        self.size_mb = size_mb

    def extract_articles(self, dump_path: Path, target_size: int) -> list[str]:
        """Extract articles from Wikipedia dump up to target size.

        Args:
            dump_path: Path to Wikipedia dump file
            target_size: Target size in bytes

        Returns:
            List of article texts
        """
        # Implementation depends on Wikipedia dump format
        # This is a placeholder for the actual implementation
        raise NotImplementedError("Wikipedia extraction to be implemented")
