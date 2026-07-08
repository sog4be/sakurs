# sakurs-core

High-performance sentence boundary detection library using the Δ-Stack Monoid algorithm.

⚠️ **API Stability Notice**: This crate is pre-1.0. The 0.2 series is the first pass at a
stable public surface — the API is intentionally small (the `api` module, re-exported at the
crate root), so internal improvements no longer require breaking changes. Pin a minor version:

```toml
sakurs-core = "0.2"
```

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Advanced Usage](#advanced-usage)
  - [Custom Configuration](#custom-configuration)
  - [Processing Files](#processing-files)
  - [Streaming Large Files](#streaming-large-files)
- [Language Support](#language-support)
- [Algorithm](#algorithm)
- [License](#license)
- [Links](#links)

## Features

- **Parallel Processing**: near-linear multicore scaling using the Δ-Stack Monoid algorithm
  (measured: 252 MB/s single-threaded, 1.44 GB/s at 8 threads on plain English text)
- **Sequential Equivalence**: any chunk size and thread count produce exactly the same
  boundaries as processing the whole text sequentially — a guaranteed, property-tested invariant
- **Language Support**: English and Japanese bundled; new languages are compiled TOML
  configurations, no code required
- **Complex Text Support**: handles nested quotes, abbreviations, and cross-chunk boundaries
  correctly, including candidates whose deciding context crosses a chunk edge

## Quick Start

```rust
use sakurs_core::{SentenceProcessor, Input};

// Create processor with default configuration
let processor = SentenceProcessor::with_language("en")?;

// Process text
let text = "Hello world. This is a test.";
let output = processor.process(Input::from_text(text))?;

// Use the boundaries
for boundary in &output.boundaries {
    println!("Sentence ends at byte offset: {}", boundary.offset);
}
```

## Advanced Usage

### Custom Configuration

```rust
use sakurs_core::{Config, Input, SentenceProcessor};

let config = Config::builder()
    .language("ja")?          // Japanese language rules
    .threads(Some(4))         // Use 4 threads
    .chunk_size(512 * 1024)   // 512KB chunks
    .build()?;

let processor = SentenceProcessor::with_config(config)?;
```

### Processing Files

```rust
use sakurs_core::{Input, SentenceProcessor};

let processor = SentenceProcessor::new();
let output = processor.process(Input::from_file("document.txt"))?;

println!("Found {} sentences", output.boundaries.len());
println!("Processing took {:?}", output.metadata.duration);
```

### Streaming Large Files

```rust
use sakurs_core::{Config, Input, SentenceProcessor};

// Use the streaming preset for memory-efficient processing (32KB chunks, limited threads)
let processor = SentenceProcessor::with_config(Config::streaming())?;
let output = processor.process(Input::from_file("large_document.txt"))?;
```

`Config::streaming()`, `Config::small_text()`, and `Config::large_text()` are fixed presets
for English; to combine a preset's chunk size/thread count with another language, use
`Config::builder()` directly with the same `chunk_size`/`threads` values.

## Language Support

Currently bundled:
- English (`en`)
- Japanese (`ja`)

A language is a TOML configuration file compiled at load time into the algorithm's decision
oracles — adding a language requires no code. See the [main repository](https://github.com/sog4be/sakurs)
for documentation on adding new languages.

## Algorithm

This library implements the Δ-Stack Monoid algorithm, which represents parsing state as an
associative monoid and defers context-dependent decisions (abbreviations, sentence starters,
enclosure suppression) whose window crosses a chunk edge until the neighboring chunk's context
is available. This gives:

1. Splitting text into chunks at arbitrary positions
2. Processing chunks independently, in parallel
3. Combining results in any order
4. Guaranteed identical results to sequential processing

For detailed algorithm documentation, see the [main repository](https://github.com/sog4be/sakurs).

## License

MIT License. See [LICENSE](https://github.com/sog4be/sakurs/blob/main/LICENSE) for details.

## Links

- [GitHub Repository](https://github.com/sog4be/sakurs)
- [Documentation](https://docs.rs/sakurs-core)
- [Issue Tracker](https://github.com/sog4be/sakurs/issues)