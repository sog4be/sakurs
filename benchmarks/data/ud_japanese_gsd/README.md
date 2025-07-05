# UD Japanese GSD r2.16 for Sakurs Benchmarks

This directory contains data processing tools for the Universal Dependencies Japanese GSD Treebank release 2.16.

## Dataset Overview

- **Source**: Universal Dependencies Japanese GSD r2.16
- **Size**: 8,071 sentences, 232,753 words (tokens)
- **Genres**: Wikipedia articles, news articles, blog posts
- **Format**: CoNLL-U (converted to sakurs-compatible JSON)
- **License**: CC BY-SA-NC 4.0
- **Download**: http://hdl.handle.net/11234/1-5901
- **Advantage**: Full text available (unlike UD Japanese-BCCWJ)

## Data Splits

| Split | Sentences | Words | Description |
|-------|-----------|-------|-------------|
| Train | 7,027 | 203,018 | Training data (87%) |
| Dev | 501 | 14,470 | Development data (6%) |
| Test | 543 | 15,265 | Test data (7%) |
| **Total** | **8,071** | **232,753** | **Combined dataset** |

## Usage

### Download and Process Data

```bash
# Download and process UD Japanese GSD r2.16
python download.py

# Force re-download
python download.py --force

# Specify output location
python download.py --output custom/path/ud_japanese_gsd.json
```

### Python API

```python
from ud_japanese_gsd import is_available, load_sample

# Check if data is available
if is_available():
    print("UD Japanese GSD data is ready!")

# Load sample data for testing
sample = load_sample()
```

## File Structure

```
ud_japanese_gsd/
├── __init__.py          # Python module
├── download.py          # Download and processing script
├── loader.py           # Data loading utilities
├── README.md           # This file
├── Makefile            # Build automation
└── cache/              # Processed data
    └── ud_japanese_gsd.json
```

## Data Processing

The processing pipeline:

1. **Download**: Fetches UD r2.16 release archive (~625MB)
2. **Extract**: Extracts UD_Japanese-GSD treebank
3. **Parse**: Processes CoNLL-U files (train/dev/test splits)
4. **Combine**: Merges all splits into single dataset
5. **Convert**: Transforms to sakurs-compatible JSON format
6. **Validate**: Ensures data integrity

### CoNLL-U Processing Details

- **Sentence boundaries**: Detected from empty lines
- **Word tokenization**: Japanese text without spaces between words
- **Multi-word tokens**: Properly handled
- **Comments**: Filtered out (#-prefixed lines)
- **Word forms**: Extracted from FORM field (column 2)


## Genre Characteristics

- **Wikipedia**: Japanese Wikipedia articles (encyclopedic content)
- **News**: News articles from various sources
- **Blog**: Blog posts and web content

This provides diverse Japanese text suitable for evaluating sentence segmentation accuracy.

## Dependencies

- `click`: Command-line interface
- `tqdm`: Progress bars
- `requests`: HTTP downloads

## Notes

- First download may take several minutes (downloading ~625MB archive)
- Processed data is cached for subsequent use
- Temporary files are automatically cleaned up
- Compatible with sakurs benchmark infrastructure
- Japanese text is processed without spaces between words (as is standard)

## Alternative Download Method

If the automatic download fails, you can manually process data from GitHub:

```bash
# Clone the GitHub repository
git clone https://github.com/UniversalDependencies/UD_Japanese-GSD.git

# Process the cloned data
# (This functionality can be added in future updates)
```

## Key Advantages

1. **Full text availability**: GSD includes complete text for accurate benchmarking
2. **No manual steps**: Fully automated download and processing
3. **Permissive licensing**: CC BY-SA-NC 4.0 allows free use for research
4. **Benchmark accuracy**: Can accurately measure sentence segmentation performance