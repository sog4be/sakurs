"""Data loading utilities for UD English EWT corpus."""

import json
from pathlib import Path
from typing import Any


def is_available() -> bool:
    """Check if UD English EWT data is available."""
    cache_file = Path(__file__).parent / "cache" / "ud_english_ewt.json"
    test_cache_file = Path(__file__).parent / "cache" / "test_ud_english_ewt.json"
    return cache_file.exists() or test_cache_file.exists()


def load_corpus() -> dict[str, Any]:
    """Load the full UD English EWT corpus."""
    cache_file = Path(__file__).parent / "cache" / "ud_english_ewt.json"
    test_cache_file = Path(__file__).parent / "cache" / "test_ud_english_ewt.json"
    
    # Try full corpus first
    if cache_file.exists():
        with open(cache_file, "r", encoding="utf-8") as f:
            data = json.load(f)
            # Convert old format to new format if needed
            if "text" in data and "documents" not in data:
                # Old format: single text field
                return {
                    "metadata": data.get("metadata", {}),
                    "documents": [{
                        "text": data["text"],
                        "sentences": data["text"].split("\n") if "\n" in data["text"] else [data["text"]],
                        "id": "full_corpus"
                    }]
                }
            return data
    
    # Fall back to test set
    if test_cache_file.exists():
        with open(test_cache_file, "r", encoding="utf-8") as f:
            return json.load(f)
    
    raise FileNotFoundError("No UD English EWT data found. Please run download.py first.")


def load_sample() -> dict[str, Any]:
    """Load UD English EWT data for benchmarking.
    
    This loads the full corpus if available, otherwise a hardcoded sample.
    """
    try:
        # Try to load the full corpus
        return load_corpus()
    except FileNotFoundError:
        # Return hardcoded sample as fallback
        return {
            "name": "ud_english_ewt_sample",
            "metadata": {
                "source": "UD English EWT Sample",
                "sentences": 2,
                "words": 17,
                "genres": ["sample"],
            },
            "documents": [{
                "text": "From the AP comes this story: President Bush met with congressional leaders today. The discussion focused on economic policy issues.",
                "sentences": [
                    "From the AP comes this story: President Bush met with congressional leaders today.",
                    "The discussion focused on economic policy issues."
                ],
                "id": "sample_doc"
            }]
        }
