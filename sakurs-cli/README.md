# sakurs-cli

Fast, parallel sentence boundary detection for the command line.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Features](#features)
- [Usage Examples](#usage-examples)
  - [Basic File Processing](#basic-file-processing)
  - [Batch Processing](#batch-processing)
  - [Output Formats](#output-formats)
  - [Performance Tuning](#performance-tuning)
- [Command Reference](#command-reference)
- [Examples](#examples)
  - [Processing Japanese Text](#processing-japanese-text)
  - [Analyzing Code Documentation](#analyzing-code-documentation)
  - [Pipeline Integration](#pipeline-integration)
- [License](#license)
- [Links](#links)

## Installation

```bash
cargo install sakurs-cli
```

After installation, the `sakurs` command will be available in your PATH.

## Quick Start

```bash
# Process text files
sakurs process -i document.txt

# Process multiple files with glob pattern
sakurs process -i "*.txt"

# Process from stdin
echo "Hello world. How are you?" | sakurs process -i -

# Output as JSON
sakurs process -i document.txt -f json
```

## Features

- **Parallel Processing**: Automatically utilizes multiple CPU cores for optimal performance
- **Multiple Output Formats**: Plain text, JSON, or quiet mode for different use cases
- **Language Support**: Built-in configurations for English and Japanese

## Usage Examples

### Basic File Processing

```bash
# Process a single file
sakurs process -i report.txt

# Process with specific language
sakurs process -i japanese_text.txt -l japanese
```

### Batch Processing

```bash
# Process all text files in a directory
sakurs process -i "documents/*.txt"

# Recursive processing with complex patterns
sakurs process -i "**/*.{txt,md}"
```

### Output Formats

```bash
# Default format (human-readable)
sakurs process -i file.txt

# JSON format for programmatic use
sakurs process -i file.txt -f json

# Quiet mode (only sentence count)
sakurs process -i file.txt -f quiet
```

### Performance Tuning

For large files, you can tune performance:

```bash
# Use 8 threads with 1MB chunks
sakurs process -i large_file.txt --threads 8 --chunk-kb 1024

# Sequential processing (useful for debugging)
sakurs process -i file.txt --sequential
```

## Command Reference

```
sakurs process [OPTIONS]

OPTIONS:
    -i, --input <INPUT>           Input file(s) or '-' for stdin
    -o, --output <OUTPUT>         Output file (default: stdout)
    -f, --format <FORMAT>         Output format [default: text]
                                  [possible values: text, json, quiet]
    -l, --language <LANGUAGE>     Language for sentence detection [default: en]
                                  [possible values: en, ja, english, japanese]
    --sequential                  Force sequential processing
    --parallel                    Force parallel processing (default: auto)
    --threads <N>                 Number of threads (default: CPU count)
    --chunk-kb <SIZE>             Chunk size in KB [default: 256]
    -h, --help                    Print help
    -V, --version                 Print version
```

## Examples

### Processing Japanese Text

```bash
sakurs process -i japanese_novel.txt -l ja -f json > sentences.json
```

### Analyzing Code Documentation

```bash
# Extract sentences from all README files
sakurs process -i "**/README.md" -f quiet
```

### Pipeline Integration

```bash
# Count sentences in git commit messages
git log --format=%B | sakurs process -i - -f quiet

# Extract sentences from specific files
find . -name "*.txt" -exec sakurs process -i {} \;
```

## License

MIT License. See [LICENSE](https://github.com/sog4be/sakurs/blob/main/LICENSE) for details.

## Links

- [GitHub Repository](https://github.com/sog4be/sakurs)
- [Issue Tracker](https://github.com/sog4be/sakurs/issues)
- [Core Library Documentation](https://docs.rs/sakurs-core)