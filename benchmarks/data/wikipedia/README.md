# Wikipedia Corpus for Performance Benchmarking

This directory contains tools for creating fixed-size Wikipedia samples using Hugging Face datasets for large-scale performance benchmarking of sentence segmentation.

## Overview

This implementation uses the official Hugging Face `wikimedia/wikipedia` dataset, which provides:
- Pre-processed and cleaned Wikipedia text (no Wiki markup)
- Monthly snapshots for reproducibility
- Support for 300+ languages
- Streaming support for memory-efficient processing

## Supported Languages

- **English** (`en`): English Wikipedia
- **Japanese** (`ja`): Japanese Wikipedia
- All other Wikipedia languages supported by the dataset

## Usage

### Quick Start

```python
from wikipedia import create_loader, is_available

# Check if data is available
if is_available('en'):
    print("English Wikipedia sample is ready!")

# Create a loader with specific date for reproducibility
loader = create_loader('en', size_mb=500, date='20231101')

# Download and prepare sample (if not cached)
sample_path = loader.download()

# Load sample data
data = loader.load()
print(f"Loaded {data['metadata']['articles']} articles from {data['metadata']['date']}")
```

### Command Line

```bash
# Prepare Wikipedia samples via CLI scripts
cd ../../cli
python scripts/prepare_data.py

# Or use Python directly
python -c "from data.wikipedia import create_loader; create_loader('en', 500).download()"
python -c "from data.wikipedia import create_loader; create_loader('ja', 500).download()"
```

## File Structure

```
wikipedia/
├── __init__.py         # Module interface
├── loader.py           # Hugging Face dataset loader
├── README.md           # This file
└── cache/              # Cached samples
    └── wikipedia_*.txt # Processed samples
```

## Data Processing Pipeline

1. **Load from Hugging Face**: Stream the dataset without downloading everything
2. **Sample Creation**: Extract articles until target size is reached
3. **Caching**: Store processed samples for reuse

### Benefits of Hugging Face Dataset

- **Pre-cleaned**: Wiki markup, templates, and references already removed
- **Consistent**: Professional processing by Wikimedia team
- **Versioned**: Use specific dates (e.g., `20231101`) for reproducibility
- **Efficient**: Streaming support for large datasets
- **Updated**: Monthly snapshots available

### Sample Format

Each sample file contains articles with clear boundaries:
```
===== Article 1: Title =====

Article text goes here...

===== Article 2: Another Title =====

More article text...
```

## Performance Considerations

### Memory Usage
- Streaming processing - no need to load entire dataset
- Fixed-size samples (default 500MB)
- Efficient caching system

### Network
- Downloads only what's needed via streaming
- Hugging Face's CDN for fast downloads
- Automatic caching by HF datasets library

## Benchmark Scenarios

### Throughput Testing
```bash
# Test processing speed on 500MB sample
hyperfine --warmup 3 \
    'sakurs process --input wikipedia_en_500mb.txt --format text'
```

### Scalability Testing
```bash
# Test with different thread counts
hyperfine --warmup 3 \
    -L threads 1,2,4,8 \
    'sakurs process --input wikipedia_en_500mb.txt --threads {threads}'
```

### Memory Profiling
```bash
# Monitor memory usage
/usr/bin/time -l sakurs process --input wikipedia_en_500mb.txt
```

## Language-Specific Notes

### English Wikipedia
- Clean, well-structured text
- No special processing needed
- Mix of article types and topics

### Japanese Wikipedia
- Pre-processed by Hugging Face
- Handles mixed scripts correctly
- No additional cleaning required

## Reproducibility

To ensure reproducible benchmarks:
1. **Always specify date**: Use `date='20231101'` not default/latest
2. **Document version**: Record the exact dataset configuration used
3. **Fixed seeds**: Sample creation is deterministic for same parameters
4. **Cache samples**: Reuse same samples across benchmark runs

Example for paper citations:
```
We used Wikipedia samples from the Hugging Face wikimedia/wikipedia dataset
(snapshot: 20231101) with 500MB samples for each language.
```

## Migration from Custom Implementation

This implementation replaces our previous custom Wikipedia downloader and parser with the Hugging Face dataset. Benefits include:
- 80% less code to maintain
- Professional text cleaning
- Regular updates without code changes
- Better reproducibility with versioned datasets

## Troubleshooting

### Dataset Not Found
- Check internet connection
- Verify language code is correct (use ISO 639-1)
- Ensure datasets library is installed: `uv pip install datasets>=2.14.0`

### Memory Issues
- Use streaming mode (already default)
- Reduce sample size if needed
- Close other applications

### Slow Downloads
- Hugging Face automatically uses CDN
- First download caches the data
- Subsequent runs use cached data

## References

- [Hugging Face Wikipedia Dataset](https://huggingface.co/datasets/wikimedia/wikipedia)
- [Wikimedia Downloads](https://dumps.wikimedia.org/)
- [Hugging Face Datasets Docs](https://huggingface.co/docs/datasets)

## License

Wikipedia content is available under Creative Commons Attribution-ShareAlike License.
This tool is part of the Sakurs project.