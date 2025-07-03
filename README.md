<p>
    <img src=".github/assets/logo.png" style="margin-left:100px; margin-right:100px;" >
</p>

<p align="center">
    <em>Fast, safe, and structurally correct — sentence boundary detection that scales.</em>
</p>

<p align="center">
    <a href="https://github.com/sog4be/sakurs/actions/workflows/ci.yml">
        <img src="https://github.com/sog4be/sakurs/actions/workflows/ci.yml/badge.svg" alt="CI Status">
    </a>
    <a href="https://github.com/sog4be/sakurs/actions/workflows/coverage.yml">
        <img src="https://img.shields.io/badge/coverage-89.89%25-green" alt="Coverage" id="coverage-badge">
    </a>
    <a href="https://github.com/sog4be/sakurs/blob/main/LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
    <a href="https://github.com/sog4be/sakurs">
        <img src="https://img.shields.io/badge/rust-1.81+-orange.svg" alt="Rust Version">
    </a>
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