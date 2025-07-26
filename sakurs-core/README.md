# sakurs-core

High-performance sentence boundary detection library using the Delta-Stack Monoid algorithm.

⚠️ **API Stability Warning**: This crate is in pre-release (v0.1.0). 
APIs may change significantly before v1.0.0. We recommend pinning to exact versions:

```toml
sakurs-core = "=0.1.0"
```

## Features

- 🚀 **Parallel Processing**: Near-linear speedup with multiple cores
- 🌍 **Language Support**: Configurable rules for English and Japanese
- 📏 **Zero-Copy**: Efficient string handling without unnecessary allocations
- 🔄 **Monoid-Based**: Mathematically sound parallel algorithm
- 🎯 **High Accuracy**: Handles complex cases like nested quotes and abbreviations

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

## Performance

The Delta-Stack Monoid algorithm enables true parallel processing:

- **Sequential**: O(N) time, O(1) space
- **Parallel**: O(N/P + log P) time, O(P) space
- Benchmarks show 3.5x speedup on 4 cores, 6.8x on 8 cores

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