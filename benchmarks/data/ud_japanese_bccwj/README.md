# UD Japanese-BCCWJ r2.16 for Sakurs Benchmarks

This directory contains data processing tools for the Universal Dependencies Japanese BCCWJ (Balanced Corpus of Contemporary Written Japanese) release 2.16.

## Dataset Overview

- **Source**: Universal Dependencies Japanese-BCCWJ r2.16
- **Size**: 57,109 sentences (1,273k words)
- **Documents**: 1,980 documents across 6 domains
- **Format**: CoNLL-U (original text not included due to license)
- **License**: CC BY-NC-SA 4.0
- **Download**: http://hdl.handle.net/11234/1-5901

## Important Note on Text Availability

Due to licensing restrictions, the UD Japanese-BCCWJ treebank does **not** include the original text. The CoNLL-U files contain:
- Word forms (tokens)
- Morphological annotations
- Dependency relations
- Sentence structure

But **not** the original running text. This impacts benchmarking as we need to reconstruct sentences from tokens.

## Options for Obtaining Original Text

### 1. For BCCWJ License Holders
If you have purchased the BCCWJ DVD edition, you can merge the original text:

```bash
# Download with merge option
python download.py --merge-bccwj /path/to/core_SUW.txt
```

The merge process requires the official script from the UD repository:
https://github.com/UniversalDependencies/UD_Japanese-BCCWJ

### 2. Token-based Reconstruction
Without the original text, we reconstruct sentences by concatenating tokens:
- This may not perfectly match the original formatting
- Whitespace information is lost
- Some punctuation handling may differ

### 3. Alternative Corpora
Consider using other Japanese corpora with full text:
- UD Japanese-GSD (includes original text)
- KWDLC (Kyoto University Web Document Leads Corpus)
- Custom annotated data

## Data Splits

| Split | Documents | Sentences | Description |
|-------|-----------|-----------|-------------|
| Train | ~1,500 | ~40,000 | Training data |
| Dev | ~240 | ~8,500 | Development data |
| Test | ~240 | ~8,500 | Test data |
| **Total** | **1,980** | **57,109** | **Full dataset** |

## Usage

### Download and Process Data

```bash
# Download UD Japanese-BCCWJ r2.16
python download.py

# Force re-download
python download.py --force

# With BCCWJ merge (requires license)
python download.py --merge-bccwj /path/to/core_SUW.txt
```

### Python API

```python
from ud_japanese_bccwj import is_available, load_corpus, load_sample

# Check if data is available
if is_available():
    print("UD Japanese-BCCWJ data is ready!")

# Load full corpus
corpus = load_corpus()

# Load sample for testing
sample = load_sample()
```

## File Structure

```
ud_japanese_bccwj/
├── __init__.py         # Python module
├── download.py         # Download and processing script
├── loader.py          # Data loading utilities
├── README.md          # This file
├── Makefile           # Build commands
└── cache/             # Processed data
    └── ud_japanese_bccwj.json
```

## Data Processing

The processing pipeline:

1. **Download**: Fetches UD r2.16 release archive (~625MB)
2. **Extract**: Extracts UD_Japanese-BCCWJ treebank
3. **Parse**: Processes CoNLL-U files (train/dev/test splits)
4. **Reconstruct**: Attempts to reconstruct sentences from tokens
5. **Convert**: Transforms to sakurs-compatible JSON format
6. **Validate**: Ensures data integrity

### Token Reconstruction Challenges

- **No spaces**: Japanese doesn't use spaces between words
- **Particles**: Grammatical particles may be separate tokens
- **Punctuation**: May be tokenized differently
- **Multi-word expressions**: Split into components

## Comparison with Other Japanese Corpora

| Corpus | Sentences | Original Text | License | Domain |
|--------|-----------|---------------|---------|--------|
| UD Japanese-BCCWJ | 57,109 | No* | CC BY-NC-SA | Various |
| UD Japanese-GSD | 8,071 | Yes | CC BY-SA | News/Web |
| UD Japanese-Modern | 822 | Yes | CC BY-SA | Literature |

*Original text available only for BCCWJ license holders

## Japanese-Specific Considerations

### Character Types
- **Hiragana** (ひらがな): Phonetic characters
- **Katakana** (カタカナ): Phonetic characters for foreign words
- **Kanji** (漢字): Chinese characters
- **Romaji**: Latin alphabet
- **Numbers**: Arabic and Japanese numerals

### Punctuation
- Sentence endings: 。(full stop) ！ ？
- Pauses: 、(comma) ・ (middle dot)
- Quotes: 「」『』
- Parentheses: （）【】

### Challenges for Sentence Segmentation
1. No explicit word boundaries (no spaces)
2. Multiple valid segmentation points
3. Embedded sentences in quotes
4. Complex honorific expressions
5. Mixed scripts (Japanese + English)

## Citation

If you use this corpus, please cite:

```bibtex
@inproceedings{omura-etal-2023-ud,
    title = "UD Japanese-BCCWJ: Universal Dependencies Annotation for the Balanced Corpus of Contemporary Written Japanese",
    author = "Omura, Mai  and
      Asahara, Masayuki  and
      Miyao, Yusuke",
    booktitle = "Proceedings of the Universal Dependencies Workshop",
    year = "2023"
}
```

## License Notes

- The treebank annotations are licensed under CC BY-NC-SA 4.0
- The original BCCWJ text requires a separate license
- For commercial use, consider alternative corpora
- Attribution required for academic use

## Troubleshooting

### "Text not included" Error
This is expected behavior. The original text is not distributed with UD.

### Token Reconstruction Issues
If reconstructed sentences look incorrect:
1. Check if tokens are being concatenated properly
2. Verify character encoding (should be UTF-8)
3. Consider using alternative corpora with full text

### Download Failures
- Check internet connection
- Verify the UD release URL is accessible
- Try manual download from LINDAT repository

## Alternative Approaches

For full-text benchmarking, consider:
1. Using UD Japanese-GSD (smaller but includes text)
2. Creating custom annotated data
3. Using rule-based sentence splitters for comparison
4. Purchasing BCCWJ license for research purposes