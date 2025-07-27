# sakurs-core

High-performance sentence boundary detection library using the Delta-Stack Monoid algorithm.

⚠️ **API Stability Warning**: This crate is in early release (v0.1.1). 
APIs may change significantly before v1.0.0. We recommend pinning to exact versions:

```toml
sakurs-core = "=0.1.1"
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

- **Parallel Processing**: Efficient speedup with multiple cores using the Delta-Stack Monoid algorithm
- **Language Support**: Configurable rules for English and Japanese via TOML-based configuration
- **Mathematically Sound**: Based on monoid algebra, ensuring correct results in parallel execution
- **Complex Text Support**: Handles nested quotes, abbreviations, and cross-chunk boundaries correctly

## Quick Start

```rust
use sakurs_core::api::{SentenceProcessor, Input};

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
use sakurs_core::api::{Config, Input, SentenceProcessor};

let config = Config::builder()
    .language("ja")?           // Japanese language rules
    .threads(Some(4))          // Use 4 threads
    .chunk_size_kb(Some(512))  // 512KB chunks
    .build()?;

let processor = SentenceProcessor::with_config(config)?;
```

### Processing Files

```rust
use sakurs_core::api::{Input, SentenceProcessor};

let processor = SentenceProcessor::new();
let output = processor.process(Input::from_file("document.txt"))?;

println!("Found {} sentences", output.boundaries.len());
println!("Processing took {:?}", output.metadata.processing_time);
```

### Streaming Large Files

```rust
use sakurs_core::api::{Config, Input, SentenceProcessor};

// Use streaming configuration for memory-efficient processing
let config = Config::streaming()
    .language("en")?
    .build()?;

let processor = SentenceProcessor::with_config(config)?;
let output = processor.process(Input::from_file("large_document.txt"))?;
```

## Language Support

Currently supported:
- English (`en`)
- Japanese (`ja`)

Language rules are configured via TOML files. See the [main repository](https://github.com/sog4be/sakurs) 
for documentation on adding new languages.

## Algorithm

This library implements the Delta-Stack Monoid algorithm, which represents parsing 
state as an associative monoid. This mathematical property enables:

1. Splitting text into chunks
2. Processing chunks in parallel
3. Combining results in any order
4. Getting identical results to sequential processing

For detailed algorithm documentation, see the [main repository](https://github.com/sog4be/sakurs).

## License

MIT License. See [LICENSE](https://github.com/sog4be/sakurs/blob/main/LICENSE) for details.

## Links

- [GitHub Repository](https://github.com/sog4be/sakurs)
- [Documentation](https://docs.rs/sakurs-core)
- [Issue Tracker](https://github.com/sog4be/sakurs/issues)