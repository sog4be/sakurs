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
        <img src="https://img.shields.io/badge/coverage-91.47%25-brightgreen" alt="Coverage" id="coverage-badge">
    </a>
    <a href="https://github.com/sog4be/sakurs/blob/main/LICENSE">
        <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
    </a>
    <a href="https://github.com/sog4be/sakurs">
        <img src="https://img.shields.io/badge/rust-1.81+-orange.svg" alt="Rust Version">
    </a>
</p>

> [!WARNING]
> **This project is in pre-release development (v0.1.0-dev)**. 
> APIs and features may change significantly before the first stable release.
> We welcome early adopters and contributors to help shape the project!

## Installation

> **Note**: Sakurs has not yet reached its first stable release (v0.1.0). 
> Installation from crates.io will be available after the initial release.
> For now, please build from source.

### From Source (Recommended for pre-release)

```bash
# Clone the repository
git clone https://github.com/sog4be/sakurs.git
cd sakurs

# Build and install the CLI
cargo install --path sakurs-cli

# Or build without installing
cargo build --release
./target/release/sakurs --help
```

### From crates.io (Coming Soon)

```bash
cargo install sakurs-cli
```

## Quick Start

### Basic Usage

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

### Output Formats

```bash
# JSON format with sentence boundaries and offsets
sakurs process -i text.txt -f json -o output.json

# Markdown format with numbered sentences
sakurs process -i text.txt -f markdown

# Default text format (one sentence per line)
sakurs process -i text.txt
```

### Using External Language Configuration

```bash
# Generate a language configuration template
sakurs generate-config --language-code custom --output custom-lang.toml

# Edit the configuration file to define your language rules
# Then validate it
sakurs validate --language-config custom-lang.toml

# Use the custom configuration for processing
sakurs process -i text.txt --language-config custom-lang.toml

# Override the language code if needed
sakurs process -i text.txt --language-config base.toml --language-code mycode
```

### Advanced Options

```bash
# Force parallel processing for small files
sakurs process -i small.txt --parallel

# Specify exact thread count
sakurs process -i large.txt --threads 4

# Customize chunk size for parallel processing (in KB)
sakurs process -i large.txt --chunk-kb 512

# Combine thread count and chunk size for optimal performance
sakurs process -i very_large.txt --threads 8 --chunk-kb 1024

# Suppress progress output for scripting
sakurs process -i *.txt --quiet > sentences.txt

# Increase verbosity for debugging
sakurs process -i debug.txt -vv
```

## Command Reference

### `sakurs process`

Process text files to detect sentence boundaries.

**Options:**
- `-i, --input <FILE/PATTERN>` - Input files (supports glob patterns)
- `-o, --output <FILE>` - Output file (default: stdout)
- `-f, --format <FORMAT>` - Output format: text, json, markdown (default: text)
- `-l, --language <LANG>` - Language: english, japanese (default: english)
- `--language-config <FILE>` - Path to external language configuration file (TOML)
- `--language-code <CODE>` - Override language code when using external config
- `-p, --parallel` - Force parallel processing
- `-t, --threads <COUNT>` - Number of threads (default: auto)
- `-q, --quiet` - Suppress progress output
- `-v, --verbose` - Increase verbosity (can be repeated)

**Note:** `--language` and `--language-config` are mutually exclusive.

### `sakurs validate`

Validate a language configuration file.

```bash
# Validate a custom language configuration
sakurs validate --language-config my-language.toml
```

**Options:**
- `-c, --language-config <FILE>` - Path to language configuration file to validate

### `sakurs generate-config`

Generate a language configuration template.

```bash
# Generate a template for a new language
sakurs generate-config --language-code fr --output french.toml
```

**Options:**
- `-l, --language-code <CODE>` - Language code for the new configuration
- `-o, --output <FILE>` - Output file path

### `sakurs list`

List available components.

```bash
# List supported languages
sakurs list languages

# List output formats
sakurs list formats
```

## Performance and Configuration

Sakurs automatically optimizes performance based on text size and available CPU cores. For advanced usage:

- See [PERFORMANCE.md](docs/PERFORMANCE.md) for performance tuning guide
- See [SHELL_ALIASES.md](docs/SHELL_ALIASES.md) for useful shell aliases and functions

```bash
# Manual thread control
sakurs process -i large.txt --threads 8

# Common aliases (add to ~/.bashrc or ~/.zshrc)
alias sakurs-ja='sakurs process -l japanese'
alias sakurs-json='sakurs process -f json'
```

## Examples

### Processing Research Papers

```bash
# Process all PDF-extracted text files
sakurs process -i "papers/*.txt" -f json -o analysis.json

# Extract first sentences for abstract generation
sakurs process -i paper.txt | head -n 5
```

### Multilingual Document Processing

```bash
# Process English documents
sakurs process -i "en/*.txt" -l english -o english_sentences.txt

# Process Japanese documents  
sakurs process -i "ja/*.txt" -l japanese -o japanese_sentences.txt
```

### Integration with Other Tools

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

### Key Features

- **Configurable Language Rules**: Languages are defined via TOML configuration files, making it easy to add new languages or customize existing ones (see [Adding Languages](docs/ADDING_LANGUAGES.md))
- **High Performance**: Parallel processing with near-linear speedup on multicore systems
- **Memory Efficient**: Streaming support for processing large files with constant memory usage
- **Extensible**: Clean architecture allows easy addition of new languages and adapters

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.