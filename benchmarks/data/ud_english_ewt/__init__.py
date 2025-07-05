"""UD English EWT data module for sakurs benchmarks."""

from .loader import is_available, load_corpus, load_sample

__all__ = ["is_available", "load_corpus", "load_sample"]
