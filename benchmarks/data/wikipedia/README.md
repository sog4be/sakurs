# Wikipedia Corpus for Performance Benchmarking

This directory contains tools for downloading and processing Wikipedia dumps for large-scale performance benchmarking of sentence segmentation.

## Overview

Wikipedia provides an excellent source of real-world text in multiple languages for performance testing. We use fixed-size samples (default: 500MB) to ensure reproducible benchmarks.

## Supported Languages

- **English** (`en`): English Wikipedia
- **Japanese** (`ja`): Japanese Wikipedia

## Usage

### Quick Start

```python
from wikipedia import create_loader, is_available

# Check if data is available
if is_available('en'):
    print("English Wikipedia sample is ready!")

# Create a loader
loader = create_loader('en', size_mb=500)

# Download and prepare sample (if not cached)
sample_path = loader.download()

# Load sample data
data = loader.load()
print(f"Loaded {data['metadata']['articles']} articles")
```

### Command Line

```bash
# Download Wikipedia dump
python download.py --language en --date latest

# Show dump information
python download.py --language ja --info-only

# Create samples of different sizes
python -m wikipedia.sampler --language en --sizes 10,50,100,500
```

## File Structure

```
wikipedia/
├── __init__.py         # Module interface
├── download.py         # Dump downloader
├── extractor.py        # XML parser and text cleaner
├── sampler.py          # Sample generator
├── loader.py           # Unified loader interface
├── README.md           # This file
└── cache/              # Downloaded dumps and samples
    ├── dumps/          # Raw Wikipedia dumps
    └── wikipedia_*.txt # Processed samples
```

## Data Processing Pipeline

1. **Download**: Fetch compressed XML dumps from Wikimedia
2. **Extract**: Parse XML and clean Wiki markup
3. **Sample**: Create fixed-size samples (e.g., 500MB)
4. **Cache**: Store processed samples for reuse

### Text Cleaning

The extractor removes:
- Wiki markup (`[[links]]`, `{{templates}}`)
- HTML tags and comments
- Tables and infoboxes
- References and citations
- Redirect pages
- Very short articles (<100 chars)

### Sampling Strategy

- **Sequential sampling**: Articles in order until size reached
- **Reproducible**: Fixed random seed for consistency
- **Representative**: Includes various article types and lengths

## Performance Considerations

### Memory Usage
- Streaming XML parsing for large dumps
- Iterative processing to avoid loading entire dump
- Configurable chunk sizes

### Disk Space
- Raw dumps: Several GB compressed
- Samples: Configurable (10MB to 500MB+)
- Automatic cleanup of temporary files

### Network
- Respects Wikimedia rate limits
- Resume support for interrupted downloads
- Local caching to avoid re-downloads

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
- Average article length: ~3,000 characters
- Mix of article types: biographies, events, concepts
- Includes various English variants (US, UK, etc.)

### Japanese Wikipedia
- No spaces between words
- Mixed scripts (hiragana, katakana, kanji, romaji)
- Special handling for ruby annotations (furigana)
- Normalized punctuation (。、「」)

## Reproducibility

To ensure reproducible benchmarks:
1. Use specific dump dates (not "latest")
2. Fixed random seeds for sampling
3. Document Wikipedia dump version
4. Cache samples for consistent testing

## Troubleshooting

### Download Failures
- Check internet connection
- Verify Wikimedia servers are accessible
- Try alternative mirror sites
- Use `--date` to specify older dumps

### Memory Issues
- Reduce sample size
- Process dumps in streaming mode
- Increase system swap space
- Use cloud/server for processing

### Parsing Errors
- Ensure dump file is not corrupted
- Check for incomplete downloads
- Verify bz2 decompression works
- Report malformed XML issues

## References

- [Wikimedia Downloads](https://dumps.wikimedia.org/)
- [Database Download](https://en.wikipedia.org/wiki/Wikipedia:Database_download)
- [XML Schema](https://www.mediawiki.org/xml/export-0.10.xsd)

## License

Wikipedia content is available under Creative Commons Attribution-ShareAlike License.
Processing tools in this directory are part of the Sakurs project.