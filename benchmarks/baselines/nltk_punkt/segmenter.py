"""NLTK Punkt sentence tokenizer wrapper for sakurs benchmarks."""

import os
import time
from typing import List, Tuple, Optional

import nltk
from nltk.tokenize import punkt


class PunktSegmenter:
    """Wrapper for NLTK Punkt sentence tokenizer with sakurs-compatible interface."""
    
    def __init__(self, language: str = 'english'):
        """Initialize the Punkt tokenizer.
        
        Args:
            language: Language model to use (default: 'english')
        """
        self.language = language
        self._ensure_punkt_data()
        self.tokenizer = punkt.PunktSentenceTokenizer()
        
    def _ensure_punkt_data(self) -> None:
        """Ensure Punkt tokenizer data is downloaded."""
        try:
            nltk.data.find('tokenizers/punkt')
        except LookupError:
            print("Downloading NLTK Punkt data...")
            nltk.download('punkt', quiet=True)
    
    def segment(self, text: str) -> List[int]:
        """Segment text into sentences and return boundary positions.
        
        This method returns boundary positions compatible with sakurs format:
        boundaries are placed at the position after the sentence-ending punctuation.
        
        Args:
            text: Text to segment
            
        Returns:
            List of boundary positions (0-indexed character positions)
        """
        # Get sentence spans from NLTK
        spans = list(self.tokenizer.span_tokenize(text))
        
        boundaries = []
        for i in range(len(spans) - 1):  # Process all but the last sentence
            _, end = spans[i]
            next_start, _ = spans[i + 1]
            
            # The boundary should be at the start of the next sentence
            # This matches the Brown Corpus format
            boundaries.append(next_start)
        
        return boundaries
    
    def segment_with_timing(self, text: str) -> Tuple[List[int], float]:
        """Segment text and return boundaries with processing time.
        
        Args:
            text: Text to segment
            
        Returns:
            Tuple of (boundaries, processing_time_seconds)
        """
        start_time = time.perf_counter()
        boundaries = self.segment(text)
        processing_time = time.perf_counter() - start_time
        
        return boundaries, processing_time
    
    def extract_sentences(self, text: str) -> List[str]:
        """Extract sentences from text.
        
        Args:
            text: Text to segment
            
        Returns:
            List of sentences
        """
        return self.tokenizer.tokenize(text)


def create_segmenter(language: str = 'english') -> PunktSegmenter:
    """Factory function to create a Punkt segmenter.
    
    Args:
        language: Language model to use
        
    Returns:
        Configured PunktSegmenter instance
    """
    return PunktSegmenter(language)


if __name__ == "__main__":
    # Test the segmenter
    segmenter = create_segmenter()
    
    test_text = "Hello world. This is a test! How are you?"
    boundaries = segmenter.segment(test_text)
    sentences = segmenter.extract_sentences(test_text)
    
    print(f"Text: {test_text}")
    print(f"Boundaries: {boundaries}")
    print(f"Sentences: {sentences}")
    
    # Verify boundaries
    for i, boundary in enumerate(boundaries):
        if i == 0:
            sentence = test_text[:boundary]
        else:
            sentence = test_text[boundaries[i-1]:boundary]
        print(f"Sentence {i+1}: '{sentence.strip()}'")