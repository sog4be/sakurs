<p>
    <img src=".github/assets/logo.png" style="margin-left:100px; margin-right:100px;" >
</p>

<p align="center">
    <em>Fast rule-based sentence boundary detection that scales.</em>
</p>

<p align="center">
    <a href="https://github.com/sog4be/sakurs/actions/workflows/ci.yml">
        <img src="https://github.com/sog4be/sakurs/actions/workflows/ci.yml/badge.svg" alt="CI Status">
    </a>
    <a href="https://github.com/sog4be/sakurs/actions/workflows/coverage.yml">
        <img src="https://img.shields.io/badge/coverage-91.49%25-brightgreen" alt="Coverage" id="coverage-badge">
    </a>
    <a href="https://github.com/sog4be/sakurs/blob/main/LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
    <a href="https://github.com/sog4be/sakurs">
        <img src="https://img.shields.io/badge/rust-1.81+-orange.svg" alt="Rust Version">
    </a>
    <a href="https://github.com/sog4be/sakurs/tree/main/sakurs-py">
        <img src="https://img.shields.io/badge/python-3.9+-blue.svg" alt="Python Version">
    </a>
</p>

> [!NOTE]
> **This project is in early release (v0.1.1)**. 
> APIs may change in future releases, especially the internal `sakurs-core` crate.
> We welcome feedback and contributions!

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Python](#python)
  - [Command Line Tool](#command-line-tool)
- [Quick Start](#quick-start)
  - [Python API](#python-api)
  - [Command Line Interface](#command-line-interface)
- [Python API Documentation](#python-api-documentation)
- [CLI Documentation](#cli-documentation)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## Features

- **High Performance**: Implemented in Rust with the Δ-Stack Monoid algorithm, enabling true parallel processing that scales efficiently even with large datasets
- **Multiple Languages**: Built-in support for English and Japanese, easily extensible via TOML configs
- **Memory Efficient**: Streaming support for processing gigabyte-sized files with constant memory

## Installation

### Python

```bash
# Install from PyPI
pip install sakurs

# Or build from source
git clone https://github.com/sog4be/sakurs.git
cd sakurs/sakurs-py
uv pip install -e .
```

### Command Line Tool

```bash
# Install from crates.io
cargo install sakurs-cli

# Or build from source
git clone https://github.com/sog4be/sakurs.git
cd sakurs
cargo install --path sakurs-cli
```

## Quick Start

### Python API

```python
import sakurs

# Simple sentence splitting
sentences = sakurs.split("Hello world. This is a test.")
print(sentences)  # ['Hello world.', 'This is a test.']

# Process files directly
sentences = sakurs.split("document.txt")

# Japanese text
sentences = sakurs.split("これは日本語です。テストです。", language="ja")

# Memory-efficient processing for large files
for sentence in sakurs.split_large_file("huge_corpus.txt", max_memory_mb=50):
    process(sentence)

# Get detailed output with offsets
results = sakurs.split(text, return_details=True)
for sentence in results:
    print(f"{sentence.text} [{sentence.start}:{sentence.end}]")
```

### Command Line Interface

```bash
# Process a single file
sakurs process -i document.txt

# Process with Japanese language rules
sakurs process -i japanese.txt -l japanese

# Output to a file
sakurs process -i input.txt -o sentences.txt

# Process multiple files with glob pattern
sakurs process -i "docs/*.txt" -o all_sentences.txt
```

## Python API Documentation

For comprehensive Python API documentation, including detailed examples and advanced usage, see the [Python bindings documentation](sakurs-py/README.md).

## CLI Documentation

For comprehensive CLI documentation, including detailed command reference, usage examples, and performance tuning options, see the [CLI documentation](sakurs-cli/README.md).

### Performance Tuning

Sakurs automatically optimizes performance based on text size and available CPU cores. For advanced usage:

```bash
# Manual thread control
sakurs process -i large.txt --threads 8

# Stream large files with custom chunk size
sakurs process -i huge.txt --stream --stream-chunk-mb 50

# Common aliases (add to ~/.bashrc or ~/.zshrc)
alias sakurs-ja='sakurs process -l japanese'
alias sakurs-json='sakurs process -f json'
```

See [PERFORMANCE.md](docs/PERFORMANCE.md) for detailed performance tuning guide.

### CLI Examples

#### Processing Research Papers

```bash
# Process all PDF-extracted text files
sakurs process -i "papers/*.txt" -f json -o analysis.json

# Extract first sentences for abstract generation
sakurs process -i paper.txt | head -n 5
```

#### Multilingual Document Processing

```bash
# Process English documents
sakurs process -i "en/*.txt" -l english -o english_sentences.txt

# Process Japanese documents  
sakurs process -i "ja/*.txt" -l japanese -o japanese_sentences.txt
```

#### Integration with Other Tools

```bash
# Count sentences
sakurs process -i document.txt | wc -l

# Extract sentences containing keywords
sakurs process -i text.txt | grep -i "important"

# Convert to one sentence per file
sakurs process -i input.txt | split -l 1 - sentence_
```

## Architecture

The library consists of three main components:

- **`sakurs-core`** - Core Rust library implementing the Δ-Stack Monoid algorithm with configurable language rules
- **`sakurs-cli`** - Command-line interface for batch processing
- **`sakurs-py`** - Python bindings for easy integration

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for more details.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
