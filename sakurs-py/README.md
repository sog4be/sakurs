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

```bash
pip install sakurs
```

To build from source, build and install a wheel rather than an editable install (see
[Building from Source](#building-from-source) below for why):
```bash
git clone https://github.com/sog4be/sakurs.git
cd sakurs/sakurs-py
maturin build --release --features extension-module
WHEEL_FILE=$(ls -t ../target/wheels/*.whl | head -1)
uv pip install --force-reinstall "$WHEEL_FILE"
```

**Requirements**: Python 3.10 or later (tested through Python 3.14)

## Quick Start

```python
from pathlib import Path

import sakurs

# Simple sentence splitting
text = "Hello world. This is a test."
sentences = sakurs.split(text)
print(sentences)  # ['Hello world.', 'This is a test.']

# Process files directly (a string is read as a file only if it names an
# existing file on disk; otherwise it's treated as literal text)
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

### Table of Contents
- [Functions](#functions)
  - [`sakurs.split`](#sakurssplit)
  - [`sakurs.iter_split`](#sakursiter_split)
  - [`sakurs.split_large_file`](#sakurssplit_large_file)
  - [`sakurs.load`](#sakursload)
  - [`sakurs.supported_languages`](#sakurssupported_languages)
- [Classes](#classes)
  - [`SentenceSplitter`](#sakurssentencesplitter)
  - [`Sentence`](#sakurssentence)
  - [`LanguageConfig`](#sakurslanguageconfig)

### Functions

#### `sakurs.split`
Split text or file into sentences.

**Signature:**
```python
sakurs.split(
    input,
    *,
    language=None,
    language_config=None,
    threads=None,
    chunk_kb=None,
    parallel=False,
    execution_mode="adaptive",
    return_details=False,
    preserve_whitespace=False,
    encoding="utf-8"
)
```

**Parameters:**
- `input` (str | bytes | Path | TextIO | BinaryIO): Text string, file path, bytes, or file-like object
- `language` (str, optional): Language code ("en", "ja")
- `language_config` (LanguageConfig, optional): Custom language configuration
- `threads` (int, optional): Number of threads (None for auto)
- `chunk_kb` (int, optional): Chunk size in KB (default: 256) for parallel processing
- `parallel` (bool): Force parallel processing even for small inputs
- `execution_mode` (str): "sequential", "parallel", or "adaptive" (default)
- `return_details` (bool): Return Sentence objects with metadata instead of strings
- `preserve_whitespace` (bool): Keep leading/trailing whitespace on each sentence instead of trimming it
- `encoding` (str): Text encoding for file/bytes inputs (default: "utf-8")

**Returns:** List[str] or List[Sentence] if return_details=True

#### `sakurs.iter_split`
Process input and return sentences as an iterator. Loads entire input but yields incrementally.

**Signature:**
```python
sakurs.iter_split(
    input,
    *,
    language=None,
    language_config=None,
    threads=None,
    chunk_kb=None,
    encoding="utf-8"
)
```

**Parameters:** Same as `split()` except no `parallel`, `execution_mode`, `preserve_whitespace`, or `return_details` parameters

**Returns:** Iterator[str] - Iterator yielding sentences

#### `sakurs.split_large_file`
Process large files with limited memory usage.

**Signature:**
```python
sakurs.split_large_file(
    file_path,
    *,
    language=None,
    language_config=None,
    max_memory_mb=100,
    overlap_size=1024,
    encoding="utf-8"
)
```

**Parameters:**
- `file_path` (str | Path): Path to the file
- `language` (str, optional): Language code
- `language_config` (LanguageConfig, optional): Custom language configuration  
- `max_memory_mb` (int): Maximum memory to use in MB (default: 100)
- `overlap_size` (int): Bytes to overlap between chunks (default: 1024)
- `encoding` (str): File encoding (default: "utf-8")

**Returns:** Iterator[str] - Iterator yielding sentences

#### `sakurs.load`
Create a processor instance for repeated use.

**Signature:**
```python
sakurs.load(
    language,
    *,
    threads=None,
    chunk_kb=None,
    execution_mode="adaptive"
)
```

**Parameters:**
- `language` (str): Language code ("en" or "ja")
- `threads` (int, optional): Number of threads
- `chunk_kb` (int, optional): Chunk size in KB (default: 256)
- `execution_mode` (str): Processing mode

**Returns:** SentenceSplitter instance

#### `sakurs.supported_languages`
Get list of supported languages.

**Signature:**
```python
sakurs.supported_languages()
```

**Returns:** List[str] - Supported language codes

### Classes

#### `sakurs.SentenceSplitter`
Main sentence splitter class for sentence boundary detection.

**Constructor Parameters:**
- `language` (str, optional): Language code
- `language_config` (LanguageConfig, optional): Custom language configuration
- `threads` (int, optional): Number of threads
- `chunk_kb` (int, optional): Chunk size in KB (default: 256)
- `execution_mode` (str): "sequential", "parallel", or "adaptive"
- `streaming` (bool): Enable streaming mode configuration
- `stream_chunk_mb` (int): Chunk size in MB for streaming mode

**Methods:**
- `split(input, *, return_details=False, encoding="utf-8")`: Split text or file into sentences
- `iter_split(input, *, encoding="utf-8", preserve_whitespace=False)`: Return iterator over sentences
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
- `metadata` (MetadataConfig): Language code (`metadata.code`) and name (`metadata.name`)
- `terminators` (TerminatorConfig): Sentence terminator rules
- `ellipsis` (EllipsisConfig): Ellipsis handling rules
- `enclosures` (EnclosureConfig): Enclosure (quotes, parentheses) rules
- `suppression` (SuppressionConfig): Pattern suppression rules
- `abbreviations` (AbbreviationConfig): Abbreviation rules
- `sentence_starters` (SentenceStarterConfig | None): Words that can begin a sentence right
  after a terminator

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

2. **Reuse SentenceSplitter instances**: Create once, use many times
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
   processor = sakurs.SentenceSplitter(language="en", streaming=True, stream_chunk_mb=5)
   ```

4. **Leave the chunk size alone**: results are identical for every chunk size by design, and throughput is flat across a wide range — the 256KB default is right for almost all workloads.

## Benchmarks

Sakurs demonstrates significant performance improvements over existing Python sentence segmentation libraries. Benchmarks are run automatically in CI and results are displayed in GitHub Actions job summaries.

### Running Benchmarks Locally

To run performance benchmarks comparing sakurs with other libraries:

```bash
# Install benchmark dependencies
uv pip install -e ".[benchmark]"

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
from pathlib import Path

import sakurs

# Language errors
try:
    processor = sakurs.load("unsupported_language")
except sakurs.InvalidLanguageError as e:
    print(f"Language error: {e}")

# A plain string that doesn't name an existing file is treated as literal
# text, not an error — there is no sakurs.FileNotFoundError
sentences = sakurs.split("nonexistent.txt")  # -> ['nonexistent.', 'txt']

# Passing an explicit Path to a missing file raises the standard OSError
try:
    sentences = sakurs.split(Path("nonexistent.txt"))
except OSError as e:
    print(f"File error: {e}")

# Configuration errors (raised for a file that exists but fails to parse;
# a missing file raises FileNotFoundError instead, as above)
try:
    config = sakurs.LanguageConfig.from_toml("malformed.toml")
except sakurs.ConfigurationError as e:
    print(f"Config error: {e}")
except FileNotFoundError as e:
    print(f"File error: {e}")

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
WHEEL_FILE=$(ls -t ../target/wheels/*.whl | head -1)
uv pip install --force-reinstall "$WHEEL_FILE"
```

**Important Note**: Avoid using `pip install -e .` or `maturin develop` as they can lead to stale binaries that don't reflect Rust code changes. The editable install mechanism doesn't properly track changes in the compiled Rust extension module.

### Development Workflow

1. Make changes to the Rust code
2. Build the wheel: `maturin build --release --features extension-module`
3. Install the newest wheel from `../target/wheels/` with `uv pip install --force-reinstall`
4. Run tests: `python -m pytest tests/`

For convenience, you can use the Makefile from the project root:
```bash
make py-dev  # Builds and installs the wheel
make py-test # Builds, installs, and runs tests
```

### Troubleshooting

If your changes aren't reflected after rebuilding:
- Check if you have an editable install: `uv pip show sakurs` (look for "Editable project location")
- Uninstall completely: `uv pip uninstall sakurs -y`
- Reinstall from wheel as shown above
- Use `.venv/bin/python` directly instead of `uv run` to avoid automatic editable install restoration

## License

MIT License - see LICENSE file for details.
