# Sakurs Python Bindings

High-performance sentence boundary detection for Python using the Delta-Stack Monoid algorithm.

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

# Specify language
sentences = sakurs.split("これは日本語です。テストです。", language="ja")
print(sentences)  # ['これは日本語です。', 'テストです。']

# Object-oriented interface for repeated use
processor = sakurs.Processor("en")
sentences = processor.split("Hello world. This is a test.")
print(sentences)  # ['Hello world.', 'This is a test.']

# Custom configuration
config = sakurs.ProcessorConfig(
    chunk_size=16384,      # Chunk size in bytes
    overlap_size=512,      # Overlap between chunks
    num_threads=4          # Number of threads (None for auto)
)
processor = sakurs.Processor("en", config)
sentences = processor.split("Your text here...")
```

## API Reference

### Functions

#### `sakurs.split(text, language="en", config=None)`
Split text into sentences.

**Parameters:**
- `text` (str): The text to split
- `language` (str): Language code ("en" or "ja", default: "en")
- `config` (ProcessorConfig, optional): Custom configuration

**Returns:** List[str] - List of sentences

#### `sakurs.load(language, config=None)`
Create a processor instance for repeated use.

**Parameters:**
- `language` (str): Language code ("en" or "ja")
- `config` (ProcessorConfig, optional): Custom configuration

**Returns:** Processor instance

#### `sakurs.supported_languages()`
Get list of supported languages.

**Returns:** List[str] - Supported language codes

### Classes

#### `sakurs.Processor`
Main processor class for sentence boundary detection.

**Methods:**
- `split(text)`: Split text into sentences
- `language`: Get the configured language (property)
- `supports_parallel`: Check if parallel processing is supported (property)

#### `sakurs.ProcessorConfig`
Configuration for text processing.

**Attributes:**
- `chunk_size`: Size of text chunks for parallel processing (default: 8192)
- `overlap_size`: Overlap size between chunks (default: 256)
- `num_threads`: Number of threads to use (None for automatic)

## Supported Languages

- English (`en`, `english`)
- Japanese (`ja`, `japanese`)

## Performance Tips

1. **Reuse Processor instances**: Create once, use many times
   ```python
   processor = sakurs.Processor("en")
   for text in documents:
       sentences = processor.split(text)
   ```

2. **Configure threads for your workload**: 
   ```python
   # For CPU-bound batch processing
   config = sakurs.ProcessorConfig(num_threads=8)
   
   # For I/O-bound or interactive use
   config = sakurs.ProcessorConfig(num_threads=2)
   ```

3. **Adjust chunk size for document length**:
   ```python
   # For very long documents
   config = sakurs.ProcessorConfig(chunk_size=32768)
   ```

## Error Handling

```python
import sakurs

try:
    processor = sakurs.Processor("unsupported_language")
except sakurs.SakursError as e:
    print(f"Error: {e}")

# The library will handle edge cases gracefully
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