<p>
    <img src=".github/assets/logo.png" style="margin-left:100px; margin-right:100px;" >
</p>

<p align="center">
    <em>Fast, safe, and structurally correct — sentence boundary detection that scales.</em>
</p>

<p align="center">
    <img src="https://img.shields.io/badge/coverage-90.95%25-brightgreen" alt="Coverage" id="coverage-badge">
</p>

## Features

- 🚀 Scalable performance through true parallel processing
- 🌏 Multi-language support (English, Japanese)
- 🐍 Python bindings
- 📊 Streaming processing support
- ⚡ SIMD optimization

## Installation

```bash
# CLI tool
cargo install sakurs-cli

# Python library
pip install sakurs
```

## Usage Examples

### CLI

```bash
sakurs -i input.txt -o output.txt --language en
```

### Python

```python
from sakurs import DeltaSBD

sbd = DeltaSBD(language="en")
sentences = sbd.split_sentences("This is an example sentence. Another sentence follows.")
```

## Architecture

The library consists of three main components:

- **`sakurs-core`** - Core Rust library implementing the Δ-Stack Monoid algorithm
- **`sakurs-cli`** - Command-line interface for batch processing
- **`sakurs-py`** - Python bindings for easy integration

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.