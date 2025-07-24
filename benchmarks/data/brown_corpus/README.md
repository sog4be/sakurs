# Brown Corpus Data

This directory contains scripts and tools for downloading and processing the Brown Corpus for use in sakurs benchmarks.

## Overview

The Brown Corpus is a well-established corpus of American English containing approximately 1 million words from 500 sources, categorized by genre. It provides manually annotated sentence boundaries, making it ideal for evaluating sentence segmentation accuracy.

## Setup

1. Install Python dependencies:
```bash
uv pip install -r requirements.txt
```

2. Download and process the corpus:
```bash
python download.py
```

This will:
- Download the Brown Corpus from NLTK (if not cached)
- Parse sentence boundaries
- Convert to sakurs-compatible JSON format
- Save to `cache/brown_corpus.json`

## Files

- `download.py` - Downloads Brown Corpus from NLTK
- `parser.py` - Extracts sentences and boundaries
- `converter.py` - Converts to sakurs TestData format
- `requirements.txt` - Python dependencies
- `cache/` - Cached data files (git-ignored)

## Data Format

The processed data is saved as JSON with the following structure:
```json
{
  "name": "brown_corpus_full",
  "text": "The full text with spaces between sentences...",
  "boundaries": [45, 92, 156, ...],
  "metadata": {
    "source": "Brown Corpus",
    "sentences": 57340,
    "characters": 1161192,
    "genres": ["news", "fiction", "academic", ...]
  }
}
```

## Usage in Benchmarks

The processed data can be loaded in Rust benchmarks:
```rust
use sakurs_benchmarks::data::brown_corpus;

let corpus_data = brown_corpus::load_full_corpus()?;
// or
let subset = brown_corpus::load_genre("news")?;
```

## Notes

- The Brown Corpus uses linguistic sentence definitions which may differ slightly from sakurs' rule-based approach
- Sentence boundaries in the corpus are placed after the final punctuation and space
- Some preprocessing is done to handle special cases and ensure consistency