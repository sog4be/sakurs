# Sakurs Python Bindings

High-performance sentence boundary detection for Python using the Delta-Stack Monoid algorithm.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
  - [Functions](#functions)
  - [Classes](#classes)
- [Supported Languages](#supported-languages)
- [Performance Tips](#performance-tips)
- [Benchmarks](#benchmarks)
- [Error Handling](#error-handling)
- [Development](#development)
  - [Building from Source](#building-from-source)
  - [Development Workflow](#development-workflow)
  - [Troubleshooting](#troubleshooting)
- [License](#license)

## Installation

**Coming Soon** - Will be available on PyPI.

For now, build from source:
```bash
git clone https://github.com/sog4be/sakurs.git
cd sakurs/sakurs-py
pip install -e .
```

**Requirements**: Python 3.9 or later

## Quick Start

```python
import sakurs

# Simple sentence splitting
sentences = sakurs.split("Hello world. This is a test.")
print(sentences)  # ['Hello world.', 'This is a test.']

# Process files directly
sentences = sakurs.split("document.txt")  # Path as string
sentences = sakurs.split(Path("document.txt"))  # Path object

# Specify language
sentences = sakurs.split("これは日本語です。テストです。", language="ja")
print(sentences)  # ['これは日本語です。', 'テストです。']

# Get detailed output with offsets
results = sakurs.split(text, return_details=True)
for sentence in results:
    print(f"{sentence.text} [{sentence.start}:{sentence.end}]")

# Memory-efficient processing for large files
for sentence in sakurs.split_large_file("huge_corpus.txt", max_memory_mb=50):
    process(sentence)  # Process each sentence as it's found

# Responsive iteration (loads all, yields incrementally)  
for sentence in sakurs.iter_split("document.txt"):
    print(sentence)  # Get results as they're processed
```

## API Reference

### Functions

#### `sakurs.split(input, *, language=None, language_config=None, threads=None, chunk_size=None, parallel=False, execution_mode="adaptive", return_details=False, encoding="utf-8")`
Split text or file into sentences.

**Parameters:**
- `input` (str | Path | TextIO | BinaryIO): Text string, file path, or file-like object
- `language` (str, optional): Language code ("en", "ja")
- `language_config` (LanguageConfig, optional): Custom language configuration
- `threads` (int, optional): Number of threads (None for auto)
- `chunk_size` (int, optional): Chunk size in bytes for parallel processing
- `parallel` (bool): Force parallel processing even for small inputs
- `execution_mode` (str): "sequential", "parallel", or "adaptive" (default)
- `return_details` (bool): Return Sentence objects with metadata instead of strings
- `encoding` (str): Text encoding for file inputs (default: "utf-8")

**Returns:** List[str] or List[Sentence] if return_details=True

#### `sakurs.iter_split(input, *, language=None, language_config=None, threads=None, chunk_size=None, encoding="utf-8")`
Process input and return sentences as an iterator. Loads entire input but yields incrementally.

**Parameters:** Same as `split()` except no `return_details` parameter

**Returns:** Iterator[str] - Iterator yielding sentences

#### `sakurs.split_large_file(file_path, *, language=None, language_config=None, max_memory_mb=100, overlap_size=1024, encoding="utf-8")`
Process large files with limited memory usage.

**Parameters:**
- `file_path` (str | Path): Path to the file
- `language` (str, optional): Language code
- `language_config` (LanguageConfig, optional): Custom language configuration  
- `max_memory_mb` (int): Maximum memory to use in MB (default: 100)
- `overlap_size` (int): Bytes to overlap between chunks (default: 1024)
- `encoding` (str): File encoding (default: "utf-8")

**Returns:** Iterator[str] - Iterator yielding sentences

#### `sakurs.load(language, *, threads=None, chunk_size=None, execution_mode="adaptive")`
Create a processor instance for repeated use.

**Parameters:**
- `language` (str): Language code ("en" or "ja")
- `threads` (int, optional): Number of threads
- `chunk_size` (int, optional): Chunk size in bytes
- `execution_mode` (str): Processing mode

**Returns:** Processor instance

#### `sakurs.supported_languages()`
Get list of supported languages.

**Returns:** List[str] - Supported language codes

### Classes

#### `sakurs.Processor`
Main processor class for sentence boundary detection.

**Constructor Parameters:**
- `language` (str, optional): Language code
- `language_config` (LanguageConfig, optional): Custom language configuration
- `threads` (int, optional): Number of threads
- `chunk_size` (int, optional): Chunk size in bytes
- `execution_mode` (str): "sequential", "parallel", or "adaptive"
- `streaming` (bool): Enable streaming mode configuration
- `stream_chunk_size` (int): Chunk size for streaming mode

**Methods:**
- `split(input, *, return_details=False, encoding="utf-8")`: Split text or file into sentences
- `iter_split(input, *, encoding="utf-8")`: Return iterator over sentences
- `__enter__()` / `__exit__()`: Context manager support

#### `sakurs.Sentence`
Sentence with metadata (returned when `return_details=True`).

**Attributes:**
- `text` (str): The sentence text
- `start` (int): Character offset of sentence start
- `end` (int): Character offset of sentence end
- `confidence` (float): Confidence score (default: 1.0)
- `metadata` (dict): Additional metadata

#### `sakurs.LanguageConfig`
Language configuration for custom rules.

**Class Methods:**
- `from_toml(path)`: Load configuration from TOML file
- `to_toml(path)`: Save configuration to TOML file

**Attributes:**
- `code` (str): Language code
- `name` (str): Language name
- `terminators` (TerminatorConfig): Sentence terminator rules
- `ellipsis` (EllipsisConfig): Ellipsis handling rules
- `abbreviations` (AbbreviationConfig): Abbreviation rules
- `enclosures` (EnclosureConfig): Enclosure (quotes, parentheses) rules
- `suppression` (SuppressionConfig): Pattern suppression rules

## Supported Languages

- English (`en`, `english`)
- Japanese (`ja`, `japanese`)

## Performance Tips

1. **Choose the right function for your use case**:
   ```python
   # For small to medium texts - use split()
   sentences = sakurs.split(text)
   
   # For responsive processing - use iter_split()
   for sentence in sakurs.iter_split(document):
       process_immediately(sentence)
   
   # For huge files with memory constraints - use split_large_file()
   for sentence in sakurs.split_large_file("10gb_corpus.txt", max_memory_mb=100):
       index_sentence(sentence)
   ```

2. **Reuse Processor instances**: Create once, use many times
   ```python
   processor = sakurs.load("en", threads=4)
   for document in documents:
       sentences = processor.split(document)
   ```

3. **Configure for your workload**: 
   ```python
   # For CPU-bound batch processing
   processor = sakurs.load("en", threads=8, execution_mode="parallel")
   
   # For I/O-bound or interactive use
   processor = sakurs.load("en", threads=2, execution_mode="adaptive")
   
   # For memory-constrained environments
   processor = sakurs.Processor(language="en", streaming=True, stream_chunk_size=5*1024*1024)
   ```

4. **Adjust chunk size for document characteristics**:
   ```python
   # For texts with many short sentences
   sentences = sakurs.split(text, chunk_size=64*1024)
   
   # For texts with long sentences
   sentences = sakurs.split(text, chunk_size=512*1024)
   ```

## Benchmarks

Sakurs demonstrates significant performance improvements over existing Python sentence segmentation libraries. Benchmarks are run automatically in CI and results are displayed in GitHub Actions job summaries.

### Running Benchmarks Locally

To run performance benchmarks comparing sakurs with other libraries:

```bash
# Install benchmark dependencies
pip install -e ".[benchmark]"

# Run all benchmarks
pytest benchmarks/ --benchmark-only

# Run specific language benchmarks
pytest benchmarks/test_benchmark_english.py --benchmark-only
pytest benchmarks/test_benchmark_japanese.py --benchmark-only
```

### Benchmark Libraries

- **English**: Compared against [PySBD](https://github.com/nipunsadvilkar/pySBD)
- **Japanese**: Compared against [ja_sentence_segmenter](https://github.com/wwwcojp/ja_sentence_segmenter)

## Error Handling

```python
import sakurs

# Language errors
try:
    processor = sakurs.load("unsupported_language")
except sakurs.InvalidLanguageError as e:
    print(f"Language error: {e}")

# File errors
try:
    sentences = sakurs.split("nonexistent.txt")
except sakurs.FileNotFoundError as e:
    print(f"File error: {e}")

# Configuration errors
try:
    config = sakurs.LanguageConfig.from_toml("invalid.toml")
except sakurs.ConfigurationError as e:
    print(f"Config error: {e}")

# The library handles edge cases gracefully
sentences = sakurs.split("")  # Returns []
sentences = sakurs.split("No punctuation")  # Returns ["No punctuation"]
```

## Development

This package is built with PyO3 and maturin.

### Building from Source

For development, we recommend building and installing wheels rather than using editable installs:

```bash
# Build the wheel
maturin build --release --features extension-module

# Install the wheel (force reinstall to ensure updates)
pip install --force-reinstall target/wheels/*.whl
```

**Important Note**: Avoid using `pip install -e .` or `maturin develop` as they can lead to stale binaries that don't reflect Rust code changes. The editable install mechanism doesn't properly track changes in the compiled Rust extension module.

### Development Workflow

1. Make changes to the Rust code
2. Build the wheel: `maturin build --release --features extension-module`
3. Install the wheel: `pip install --force-reinstall target/wheels/*.whl`
4. Run tests: `python -m pytest tests/`

For convenience, you can use the Makefile from the project root:
```bash
make py-dev  # Builds and installs the wheel
make py-test # Builds, installs, and runs tests
```

### Troubleshooting

If your changes aren't reflected after rebuilding:
- Check if you have an editable install: `pip show sakurs` (look for "Editable project location")
- Uninstall completely: `pip uninstall sakurs -y`
- Reinstall from wheel as shown above
- Use `.venv/bin/python` directly instead of `uv run` to avoid automatic editable install restoration

## License

MIT License - see LICENSE file for details.