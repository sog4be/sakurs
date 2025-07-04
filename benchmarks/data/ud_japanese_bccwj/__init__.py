"""UD Japanese-BCCWJ corpus loader for benchmarking."""

from pathlib import Path
from .loader import UDJapaneseBCCWJLoader

# Convenience functions
def is_available() -> bool:
    """Check if UD Japanese-BCCWJ data is available."""
    loader = UDJapaneseBCCWJLoader()
    return loader.is_downloaded()

def load_corpus():
    """Load the full UD Japanese-BCCWJ corpus."""
    loader = UDJapaneseBCCWJLoader()
    return loader.load()

def load_sample():
    """Load a small sample for testing."""
    loader = UDJapaneseBCCWJLoader()
    return loader.load_sample()

__all__ = ['UDJapaneseBCCWJLoader', 'is_available', 'load_corpus', 'load_sample']