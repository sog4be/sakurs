#!/usr/bin/env python3
"""Brown Corpus data loader for Python benchmarks."""

import json
import os
from pathlib import Path
from typing import Dict, Any


def get_data_path() -> Path:
    """Get the path to the Brown Corpus data directory."""
    return Path(__file__).parent / "data"


def is_available() -> bool:
    """Check if Brown Corpus data is available."""
    data_path = get_data_path()
    return (
        data_path.exists() and
        (data_path / "sentences_all.json").exists()
    )


def load_subset(size: int) -> Dict[str, Any]:
    """Load a subset of the Brown Corpus data.
    
    Args:
        size: Number of sentences to load
        
    Returns:
        Dictionary with 'text' and 'boundaries' keys
    """
    data_path = get_data_path()
    json_file = data_path / f"sentences_{size}.json"
    
    if not json_file.exists():
        # Fall back to loading from full dataset
        return load_sentences_subset(size)
    
    with open(json_file, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    # Convert to expected format
    text = ''.join(sent['text'] for sent in data['sentences'])
    boundaries = []
    pos = 0
    
    for sent in data['sentences']:
        pos += len(sent['text'])
        if pos < len(text):  # Don't add boundary at the very end
            boundaries.append(pos)
    
    return {
        'text': text,
        'boundaries': boundaries
    }


def load_full_corpus() -> Dict[str, Any]:
    """Load the full Brown Corpus data.
    
    Returns:
        Dictionary with 'text' and 'boundaries' keys
    """
    data_path = get_data_path()
    json_file = data_path / "sentences_all.json"
    
    if not json_file.exists():
        raise FileNotFoundError(f"Brown Corpus data not found at {json_file}")
    
    with open(json_file, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    # Convert to expected format
    text = ''.join(sent['text'] for sent in data['sentences'])
    boundaries = []
    pos = 0
    
    for sent in data['sentences'][:-1]:  # All but last sentence
        pos += len(sent['text'])
        boundaries.append(pos)
    
    return {
        'text': text,
        'boundaries': boundaries
    }


def load_sentences_subset(size: int) -> Dict[str, Any]:
    """Load a subset from the full corpus.
    
    Args:
        size: Number of sentences to load
        
    Returns:
        Dictionary with 'text' and 'boundaries' keys
    """
    full_data = load_full_corpus()
    
    # Find the boundary for the subset
    if size >= len(full_data['boundaries']):
        return full_data
    
    # Get text up to the nth boundary
    end_pos = full_data['boundaries'][size - 1] if size > 0 else 0
    subset_text = full_data['text'][:end_pos]
    subset_boundaries = full_data['boundaries'][:size - 1] if size > 1 else []
    
    return {
        'text': subset_text,
        'boundaries': subset_boundaries
    }