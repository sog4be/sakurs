# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2026-07-06

### Fixed

- **Chunked/parallel results now match single-chunk results** for enclosure
  tracking (#231). Previously, Japanese text larger than one chunk lost
  roughly half of all boundaries at default settings, and quote-heavy
  English lost up to 100% at smaller chunk sizes. Four root causes fixed:
  overlapping chunks double-counted enclosure deltas (chunks are now
  strictly contiguous), the tree-based prefix sum computed incorrect
  cumulative deltas for more than four chunks (replaced with a sequential
  scan), symmetric quotes decided open/close from chunk-local depth (now
  tracked by parity), and line-start suppression rules mistook every chunk
  start for a line start
- The final sentence boundary is no longer dropped when the text ends
  exactly at a terminator in multi-chunk mode (#231)
- Abbreviation lookup no longer mixes byte offsets and character indices;
  abbreviations after non-ASCII text (e.g. "Café note: Dr. Smith") are
  recognized again (#231)
- Terminator characters defined in TOML configurations are now actually
  evaluated during scanning; previously a hardcoded set was used and the
  single-character ellipsis `…` never reached the ellipsis rules (#231)
- Fixed a panic on multi-byte characters at the edges of the ellipsis
  exception window (`byte index N is not a char boundary`), trivially
  triggered by ordinary prose with accented characters (#236)
- Unbalanced enclosures no longer suppress the rest of the document (#237):
  the line-start `)` / `）` list-item suppression rules misfired on CJK
  characters and hard-wrapped prose (Aozora Bunko's 吾輩は猫である
  segmented into zero sentences; Project Gutenberg's War and Peace lost 94%
  of its boundaries), and closing delimiters without matching openers drove
  the depth negative, silencing everything after them. The rules were
  removed and negative depth no longer suppresses boundaries

### Changed

- `overlap_size` no longer affects processing: the delta-stack pipeline
  always uses strictly contiguous chunks, because overlapping chunks
  corrupt the prefix sum by construction. The setting is deprecated and
  will be removed in v0.2.0 (#231)
- Updated dependencies: thiserror 2.0, criterion 0.8, pyo3 0.27 (with
  migration off deprecated APIs), and assorted minor bumps (#235)

### Performance

- 35–110× faster at default settings (256KB chunks, single thread) compared
  to v0.1.1: plain English 0.18 → 12.3 MB/s, quote-heavy 0.07 → 7.4 MB/s,
  abbreviation-heavy 0.09 → 3.1 MB/s. Real texts reach 17–41 MB/s
  single-threaded and up to 94 MB/s with 8 threads (#231)
- Removed per-terminator full-chunk copies and re-decodes, reused the rayon
  thread pool across phases, and replaced a quadratic character-offset
  calculation in the output layer (#231)

### Known Issues

- Boundary decisions whose lookahead is cut exactly at a chunk edge (e.g.
  an abbreviation split as `Dr.`|`Smith` across chunks) can differ from
  single-chunk results at aggressive chunk sizes (16–64KB). Not observed at
  the default 256KB in testing. A structural fix ships with the v0.2.0
  scanner redesign
- Curly quotes (`“ ” ‘ ’`) are not tracked as enclosures in the English
  configuration, so terminators inside curly-quoted speech split sentences

## [0.1.1] - 2025-07-27

### Fixed
- Fixed artifact name collision in release workflow that prevented PyPI uploads
- Release workflow now properly handles all platform builds (Linux, macOS Intel/ARM, Windows)

### Changed
- Release workflow artifacts now include OS name to prevent naming conflicts

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