"""Wrapper for ja_sentence_segmenter library."""

import logging
import sys

logger = logging.getLogger(__name__)


class JapaneseSentenceSegmenter:
    """Wrapper for ja_sentence_segmenter library."""

    def __init__(self, config: dict | None = None):
        """Initialize the segmenter.

        Args:
            config: Optional configuration dict
        """
        try:
            from ja_sentence_segmenter.common.pipeline import make_pipeline
            from ja_sentence_segmenter.concatenate.simple_concatenator import concatenate_matching
            from ja_sentence_segmenter.normalize.neologd_normalizer import normalize
            from ja_sentence_segmenter.split.simple_splitter import split_newline, split_punctuation

            self._available = True

            # Create pipeline with default configuration
            self.split_punc2 = make_pipeline(
                normalize, split_newline, concatenate_matching, split_punctuation
            )

            # Store for alternative configurations
            self._make_pipeline = make_pipeline
            self._normalize = normalize
            self._split_newline = split_newline
            self._split_punctuation = split_punctuation
            self._concatenate_matching = concatenate_matching

        except ImportError:
            logger.warning(
                "ja_sentence_segmenter not installed. Install with: pip install ja-sentence-segmenter"
            )
            self._available = False
            self.split_punc2 = None

    def is_available(self) -> bool:
        """Check if the segmenter is available."""
        return self._available

    def segment(self, text: str) -> list[str]:
        """Segment text into sentences.

        Args:
            text: Input text to segment

        Returns:
            List of sentences
        """
        if not self._available:
            raise RuntimeError("ja_sentence_segmenter not available. Please install it.")

        if not text:
            return []

        # Use the default pipeline
        sentences = list(self.split_punc2(text))

        # Filter out empty sentences
        sentences = [s.strip() for s in sentences if s.strip()]

        return sentences

    def segment_with_positions(self, text: str) -> list[tuple]:
        """Segment text and return sentences with positions.

        Args:
            text: Input text to segment

        Returns:
            List of (sentence, start_pos, end_pos) tuples
        """
        if not self._available:
            raise RuntimeError("ja_sentence_segmenter not available. Please install it.")

        sentences = self.segment(text)
        positions = []
        current_pos = 0

        for sentence in sentences:
            # Find the sentence in the original text
            start_pos = text.find(sentence, current_pos)
            if start_pos == -1:
                # Fallback if exact match not found
                start_pos = current_pos

            end_pos = start_pos + len(sentence)
            positions.append((sentence, start_pos, end_pos))
            current_pos = end_pos

        return positions

    def create_custom_pipeline(
        self, normalizer=True, newline_split=True, punctuation_split=True, concatenate=True
    ):
        """Create a custom processing pipeline.

        Args:
            normalizer: Use neologd normalizer
            newline_split: Split on newlines
            punctuation_split: Split on punctuation
            concatenate: Use concatenation rules

        Returns:
            Custom pipeline function
        """
        if not self._available:
            raise RuntimeError("ja_sentence_segmenter not available. Please install it.")

        components = []

        if normalizer:
            components.append(self._normalize)
        if newline_split:
            components.append(self._split_newline)
        if concatenate:
            components.append(self._concatenate_matching)
        if punctuation_split:
            components.append(self._split_punctuation)

        return self._make_pipeline(*components)

    def segment_batch(self, texts: list[str]) -> list[list[str]]:
        """Segment multiple texts.

        Args:
            texts: List of texts to segment

        Returns:
            List of sentence lists
        """
        return [self.segment(text) for text in texts]


# Convenience function for CLI compatibility
def segment_text(text: str) -> list[str]:
    """Segment Japanese text into sentences.

    Args:
        text: Input text

    Returns:
        List of sentences
    """
    segmenter = JapaneseSentenceSegmenter()
    if not segmenter.is_available():
        logger.error("ja_sentence_segmenter not installed")
        sys.exit(1)

    return segmenter.segment(text)
