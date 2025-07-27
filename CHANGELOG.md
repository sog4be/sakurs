# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-07-27

### Added

- **Core Features**
  - High-performance sentence boundary detection using the Delta-Stack Monoid algorithm
  - Parallel text processing with configurable execution modes (sequential, parallel, adaptive)
  - Built-in support for English and Japanese languages
  - Cross-chunk boundary resolution for accurate sentence detection in parallel processing

- **Language System**
  - TOML-based configurable language rule system - add new languages without code changes
  - External language configuration support via CLI (`--language-config`) and API
  - Language validation and configuration generation tools (`validate`, `generate-config` commands)
  - Comprehensive language rules including:
    - Abbreviation detection with Trie-based lookup
    - Context-aware ellipsis handling
    - Nested enclosure tracking (quotes, parentheses, brackets)
    - Suppression patterns for contractions and special cases
    - Multi-character terminator patterns (e.g., "!?", "?!")

- **Python API (`sakurs`)**
  - Primary `split()` function for text segmentation
  - `SentenceSplitter` class with configurable parameters
  - Streaming support with `iter_split()` and `split_large_file()` for memory-efficient processing
  - Multiple input types: text, bytes, files, and file-like objects
  - Rich output with character offsets and processing metadata
  - Performance parameters: `chunk_kb`, `stream_chunk_mb`, threads, execution modes
  - NLTK-compatible whitespace handling with `preserve_whitespace` option

- **Command-Line Interface (`sakurs-cli`)**
  - Process multiple files with glob patterns
  - Stdin/stdout support for pipeline integration
  - Multiple output formats: text, JSON, Markdown
  - Performance tuning with `--chunk-kb` option (default 256KB)
  - Thread count configuration with `--threads`
  - Language aliases for convenience (e.g., 'en'/'eng', 'ja'/'jpn')
  - Progress bars for large file processing

- **Performance Features**
  - Adaptive execution mode that automatically selects optimal processing strategy
  - Configurable chunk sizes for different workload optimizations
  - Memory-efficient streaming for processing files larger than RAM
  - Zero runtime overhead with compile-time embedded language configurations
  - UTF-8 safe chunking ensuring valid boundaries

### Fixed

- Year abbreviation handling with apostrophes (e.g., '90s, '60s)
- Thread pool configuration initialization issues
- Character vs byte offset discrepancies in Python API
- Cross-chunk abbreviation detection (e.g., "U.S." split across chunks)
- Compound punctuation pattern handling ("!?", "?!")
- Symmetric quote processing using depth-based context determination

### Security

- No external runtime dependencies for language configurations
- Safe handling of untrusted text input with bounded memory usage
- UTF-8 validation at chunk boundaries

[0.1.0]: https://github.com/sog4be/sakurs/releases/tag/v0.1.0