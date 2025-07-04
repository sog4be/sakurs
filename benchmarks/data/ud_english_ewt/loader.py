"""Data loading utilities for UD English EWT corpus."""

import json
from pathlib import Path
from typing import Dict, Any

def is_available() -> bool:
    """Check if UD English EWT data is available."""
    cache_file = Path(__file__).parent / "cache" / "ud_english_ewt.json"
    test_cache_file = Path(__file__).parent / "cache" / "test_ud_english_ewt.json"
    return cache_file.exists() or test_cache_file.exists()

def load_sample() -> Dict[str, Any]:
    """Load a small sample of UD English EWT data for testing."""
    return {
        "name": "ud_english_ewt_sample",
        "text": "From the AP comes this story: President Bush met with congressional leaders today. The discussion focused on economic policy issues.",
        "boundaries": [32, 84],  # After "story: " and "today. "
        "metadata": {
            "source": "UD English EWT Sample",
            "sentences": 2,
            "words": 17,
            "genres": ["sample"]
        }
    }