"""Data loading utilities for UD Japanese GSD corpus."""

import json
from pathlib import Path
from typing import Any


def is_available() -> bool:
    """Check if UD Japanese GSD data is available."""
    cache_file = Path(__file__).parent / "cache" / "ud_japanese_gsd.json"
    test_cache_file = Path(__file__).parent / "cache" / "test_ud_japanese_gsd.json"
    return cache_file.exists() or test_cache_file.exists()


def load_corpus() -> dict[str, Any]:
    """Load the full UD Japanese GSD corpus."""
    cache_file = Path(__file__).parent / "cache" / "ud_japanese_gsd.json"
    test_cache_file = Path(__file__).parent / "cache" / "test_ud_japanese_gsd.json"

    # Try full corpus first
    if cache_file.exists():
        with open(cache_file, "r", encoding="utf-8") as f:
            data = json.load(f)
            # Convert old format to new format if needed
            if "text" in data and "documents" not in data:
                # Old format: single text field
                # For Japanese, sentences are separated by spaces in our format
                sentences = data["text"].split(" ") if " " in data["text"] else [data["text"]]
                return {
                    "metadata": data.get("metadata", {}),
                    "documents": [
                        {"text": data["text"], "sentences": sentences, "id": "full_corpus"}
                    ],
                }
            return data

    # Fall back to test set
    if test_cache_file.exists():
        with open(test_cache_file, "r", encoding="utf-8") as f:
            return json.load(f)

    raise FileNotFoundError("No UD Japanese GSD data found. Please run download.py first.")


def load_sample() -> dict[str, Any]:
    """Load UD Japanese GSD data for benchmarking.

    This loads the full corpus if available, otherwise a hardcoded sample.
    """
    try:
        # Try to load the full corpus
        return load_corpus()
    except FileNotFoundError:
        # Return hardcoded sample as fallback
        return {
            "name": "ud_japanese_gsd_sample",
            "metadata": {
                "source": "UD Japanese GSD Sample",
                "sentences": 2,
                "words": 24,  # In Japanese, count characters
                "genres": ["sample"],
            },
            "documents": [
                {
                    "text": "今日は天気が良いです。明日も晴れるでしょう。",
                    "sentences": ["今日は天気が良いです。", "明日も晴れるでしょう。"],
                    "id": "sample_doc",
                }
            ],
        }

