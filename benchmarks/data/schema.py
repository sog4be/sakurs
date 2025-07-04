#!/usr/bin/env python3
"""Schema definitions for benchmark data formats.

This module defines the data structures shared between Python data processors
and Rust benchmark consumers to ensure compatibility.
"""

from typing import List, Dict, Any, TypedDict


class CorpusMetadata(TypedDict):
    """Metadata about a corpus."""
    source: str
    sentences: int
    characters: int
    words: int


class CorpusData(TypedDict):
    """Standard corpus data format for sakurs benchmarks."""
    name: str
    text: str
    boundaries: List[int]
    metadata: CorpusMetadata


def validate_corpus_data(data: Dict[str, Any]) -> CorpusData:
    """Validate that data conforms to the CorpusData schema.
    
    Args:
        data: Dictionary to validate
        
    Returns:
        The validated data as CorpusData
        
    Raises:
        ValueError: If data doesn't conform to schema
    """
    # Check required top-level fields
    required_fields = {"name", "text", "boundaries", "metadata"}
    missing_fields = required_fields - set(data.keys())
    if missing_fields:
        raise ValueError(f"Missing required fields: {missing_fields}")
    
    # Validate types
    if not isinstance(data["name"], str):
        raise ValueError("Field 'name' must be a string")
    
    if not isinstance(data["text"], str):
        raise ValueError("Field 'text' must be a string")
    
    if not isinstance(data["boundaries"], list):
        raise ValueError("Field 'boundaries' must be a list")
    
    if not all(isinstance(b, int) for b in data["boundaries"]):
        raise ValueError("All boundaries must be integers")
    
    if not isinstance(data["metadata"], dict):
        raise ValueError("Field 'metadata' must be a dictionary")
    
    # Validate metadata
    metadata = data["metadata"]
    metadata_fields = {"source", "sentences", "characters", "words"}
    missing_metadata = metadata_fields - set(metadata.keys())
    if missing_metadata:
        raise ValueError(f"Missing metadata fields: {missing_metadata}")
    
    # Validate metadata types
    if not isinstance(metadata["source"], str):
        raise ValueError("Metadata 'source' must be a string")
    
    for field in ["sentences", "characters", "words"]:
        if not isinstance(metadata[field], int):
            raise ValueError(f"Metadata '{field}' must be an integer")
    
    # Validate boundary consistency
    if len(data["boundaries"]) != metadata["sentences"]:
        raise ValueError(
            f"Boundary count ({len(data['boundaries'])}) doesn't match "
            f"metadata sentences ({metadata['sentences']})"
        )
    
    if len(data["text"]) != metadata["characters"]:
        raise ValueError(
            f"Text length ({len(data['text'])}) doesn't match "
            f"metadata characters ({metadata['characters']})"
        )
    
    # Validate boundaries are sorted and in range
    prev_boundary = -1
    for boundary in data["boundaries"]:
        if boundary <= prev_boundary:
            raise ValueError("Boundaries must be strictly increasing")
        if boundary >= len(data["text"]):
            raise ValueError(f"Boundary {boundary} is out of range for text length {len(data['text'])}")
        prev_boundary = boundary
    
    return data  # type: ignore