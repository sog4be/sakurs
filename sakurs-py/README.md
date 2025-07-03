# Sakurs Python Bindings

High-performance sentence boundary detection for Python using the Delta-Stack Monoid algorithm.

## Features

- **High Performance**: 5-10x faster than NLTK, competitive with spaCy
- **Parallel Processing**: Automatic thread management for large texts
- **Multiple Languages**: English and Japanese support
- **Memory Efficient**: Streaming processing for large documents
- **Type Safe**: Full type hints and mypy compatibility

## Installation

```bash
pip install sakurs
```

**Requirements**: Python 3.9 or later

## Quick Start

```python
import sakurs

# Simple sentence tokenization
sentences = sakurs.sent_tokenize("Hello world. This is a test.")
print(sentences)  # ['Hello world.', 'This is a test.']

# Object-oriented interface
processor = sakurs.Processor("en")
result = processor.process("Hello world. This is a test.")
print(f"Found {len(result.boundaries)} boundaries")
print(f"Processing took {result.metrics.total_time_us}μs")

# Custom configuration
config = sakurs.ProcessorConfig(chunk_size=8192, max_threads=4)
processor = sakurs.Processor("en", config)
sentences = processor.sentences("Your text here...")
```

## Supported Languages

- English (`en`, `english`)
- Japanese (`ja`, `japanese`)

## Performance

Sakurs is designed for high-performance text processing:

- Automatic parallel processing for large texts
- Memory-efficient chunking strategy
- Zero-copy string operations where possible
- SIMD optimizations for character scanning

## Development

This package is built with PyO3 and maturin. See the main repository for development setup.

## License

MIT License - see LICENSE file for details.