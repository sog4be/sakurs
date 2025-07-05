# UD English EWT r2.16 for Sakurs Benchmarks

This directory contains data processing tools for the Universal Dependencies English Web Treebank (UD English EWT) release 2.16.

## Dataset Overview

- **Source**: Universal Dependencies English Web Treebank r2.16
- **Size**: 16,622 sentences, 254,820 words
- **Genres**: Weblogs, newsgroups, emails, reviews, Yahoo! answers
- **Format**: CoNLL-U (converted to sakurs-compatible JSON)
- **License**: CC BY-SA 4.0
- **Download**: http://hdl.handle.net/11234/1-5901

## Data Splits

| Split | Sentences | Words | Description |
|-------|-----------|-------|-------------|
| Train | 12,544 | 204,576 | Training data (75%) |
| Dev | 2,001 | 25,149 | Development data (12%) |
| Test | 2,077 | 25,094 | Test data (13%) |
| **Total** | **16,622** | **254,820** | **Combined dataset** |

## Usage

### Download and Process Data

```bash
# Download and process UD English EWT r2.16
python download.py

# Force re-download
python download.py --force

# Specify output location
python download.py --output custom/path/ud_english_ewt.json
```

### Generate CLI Format Files

CLI format files are generated dynamically by the prepare_data script:

```bash
# From benchmarks directory
cd ../..
uv run python cli/scripts/prepare_data.py
```

This will create:
- `cli_format/ewt_plain.txt` - Plain text for processing
- `cli_format/ewt_sentences.txt` - One sentence per line for evaluation

**Note**: These files are not stored in the repository and must be generated locally.

### Python API

```python
from ud_english_ewt import is_available, load_sample

# Check if data is available
if is_available():
    print("UD English EWT data is ready!")

# Load sample data for testing
sample = load_sample()
```

## File Structure

```
ud_english_ewt/
├── __init__.py          # Python module
├── download.py          # Download and processing script
├── loader.py           # Data loading utilities
├── README.md           # This file
└── cache/              # Processed data
    └── ud_english_ewt.json
```

## Data Processing

The processing pipeline:

1. **Download**: Fetches UD r2.16 release archive (~625MB)
2. **Extract**: Extracts UD_English-EWT treebank
3. **Parse**: Processes CoNLL-U files (train/dev/test splits)
4. **Combine**: Merges all splits into single dataset
5. **Convert**: Transforms to sakurs-compatible JSON format
6. **Validate**: Ensures data integrity

### CoNLL-U Processing Details

- **Sentence boundaries**: Detected from empty lines
- **Multi-word tokens**: Properly handled (e.g., "can't" → "ca" + "n't")
- **Comments**: Filtered out (#-prefixed lines)
- **Word forms**: Extracted from FORM field (column 2)

## Comparison with Brown Corpus

| Dataset | Sentences | Words | Era | Genre |
|---------|-----------|-------|-----|-------|
| Brown Corpus | 57,339 | 1,161,192 | 1960s | News articles |
| UD English EWT | 16,622 | 254,820 | 2000s+ | Web media |

## Genre Characteristics

- **Weblogs**: Personal blogs and online diaries
- **Newsgroups**: Internet discussion forums
- **Emails**: Email communications
- **Reviews**: Product and service reviews
- **Yahoo! Answers**: Q&A platform content

This provides more informal, conversational text compared to Brown Corpus news articles.

## Dependencies

- `click`: Command-line interface
- `tqdm`: Progress bars
- `requests`: HTTP downloads (optional)

## Notes

- First download may take several minutes (downloading ~625MB archive)
- Processed data is cached for subsequent use
- Temporary files are automatically cleaned up
- Compatible with sakurs benchmark infrastructure

## Alternative Download Method

If the automatic download fails, you can manually process data from GitHub:

```bash
# Clone the GitHub repository
git clone https://github.com/UniversalDependencies/UD_English-EWT.git

# Process the cloned data
# (This functionality can be added in future updates)
```