# Brown Corpus data processing module

from .loader import (
    get_data_path,
    is_available,
    load_full_corpus,
    load_sentences_subset,
    load_subset,
)

__all__ = [
    "get_data_path",
    "is_available",
    "load_full_corpus",
    "load_sentences_subset",
    "load_subset",
]
