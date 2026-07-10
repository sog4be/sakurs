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

- **Parallel Processing**: automatically utilizes multiple CPU cores; throughput no longer
  depends on chunk size, so tuning is optional
- **Multiple Output Formats**: plain text, JSON, or Markdown
- **Language Support**: built-in configurations for English and Japanese, plus external TOML
  language configurations via `--language-config`
- **Configuration Tooling**: `validate` compiles a language configuration and reports
  rule-level errors; `generate-config` scaffolds a new one

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

# Markdown format
sakurs process -i file.txt -f markdown

# Suppress the progress bar (sentence output is unchanged)
sakurs process -i file.txt -q
```

### Performance Tuning

The scanner does constant, allocation-free work per character and chunking is zero-copy, so
throughput no longer depends on chunk size — the 256KB default is fine for files of any size.
`--threads`/`--chunk-kb` remain available for explicit control:

```bash
# Force a specific thread count
sakurs process -i large_file.txt --threads 8

# Force single-threaded processing (useful for debugging)
sakurs process -i file.txt --threads 1

# Force parallel processing even for small files (default: chosen automatically)
sakurs process -i file.txt --parallel
```

## Command Reference

```
sakurs process [OPTIONS] --input <FILE/PATTERN>

OPTIONS:
    -i, --input <FILE/PATTERN>            Input files or patterns (supports glob, use '-' for stdin)
    -o, --output <FILE>                   Output file (default: stdout)
    -f, --format <FORMAT>                 Output format [default: text]
                                           [possible values: text (txt), json, markdown (md)]
    -l, --language <LANGUAGE>             Language for sentence detection (default: english)
                                           [possible values: english (en, eng), japanese (ja, jpn)]
                                           Mutually exclusive with --language-config
    -c, --language-config <FILE>          Path to an external language configuration file (TOML)
                                           Mutually exclusive with --language
    --language-code <LANGUAGE_CODE>       Language code for the external configuration (optional,
                                           only used with --language-config)
    -p, --parallel                        Force parallel processing even for small files
    -t, --threads <COUNT>                 Number of threads for parallel processing (default: auto)
    --chunk-kb <SIZE_KB>                  Chunk size in KB for parallel processing [default: 256]
    -q, --quiet                           Suppress progress output
    -v, --verbose...                      Increase verbosity
    --stream                              Enable streaming mode for large files
    --stream-chunk-mb <STREAM_CHUNK_MB>   Streaming chunk size in MB [default: 10]
    -h, --help                            Print help
    -V, --version                         Print version
```

`sakurs process` is the main subcommand; three more are available:

```bash
# Validate (and compile) a language configuration, catching rule-level problems
# like invalid regexes or rules whose context exceeds the algorithm's judgment window
sakurs validate -c my_language.toml

# Scaffold a new language configuration template
sakurs generate-config -l fr -o french.toml

# List built-in languages or output formats
sakurs list languages
sakurs list formats
```

## Examples

### Processing Japanese Text

```bash
sakurs process -i japanese_novel.txt -l ja -f json > sentences.json
```

### Analyzing Code Documentation

```bash
# Count sentences across all README files (one sentence per output line)
sakurs process -i "**/README.md" -q | wc -l
```

### Pipeline Integration

```bash
# Count sentences in git commit messages
git log --format=%B | sakurs process -i - -q | wc -l

# Extract sentences from specific files
find . -name "*.txt" -exec sakurs process -i {} \;
```

## License

MIT License. See [LICENSE](https://github.com/sog4be/sakurs/blob/main/LICENSE) for details.

## Links

- [GitHub Repository](https://github.com/sog4be/sakurs)
- [Issue Tracker](https://github.com/sog4be/sakurs/issues)
- [Core Library Documentation](https://docs.rs/sakurs-core)