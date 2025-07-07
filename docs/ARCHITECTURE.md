# Architecture Design

## Overview

The Delta-Stack Monoid Sentence Boundary Detection (SBD) library is a high-performance, parallel text processing system designed to identify sentence boundaries in multiple languages. This document describes the architecture, design decisions, and extension points to help contributors understand and work with the codebase effectively.

### Why This Architecture?

We chose a **Hexagonal Architecture (Ports & Adapters)** pattern to achieve:

- **Clear separation of concerns** - Algorithm logic is isolated from I/O and language bindings
- **Easy testing** - Core logic can be tested without external dependencies
- **Multiple interfaces** - Same core supports CLI, Python, WASM, and streaming APIs
- **Future extensibility** - New languages and adapters can be added without modifying the core

## System Architecture

```mermaid
graph TB
    subgraph "External World"
        USER[Users]
        PY[Python Apps]
        CLI[Command Line]
        WEB[Web Browser]
        STREAM[Streaming Apps]
    end
    
    subgraph "Adapters (Outer Layer)"
        ADP_CLI[CLI Adapter]
        ADP_PY[Python Adapter]
        ADP_WASM[WASM Adapter]
        ADP_STRM[Streaming Adapter]
    end
    
    subgraph "API Layer (Public Interface)"
        API[SentenceProcessor]
        CFG[Config Builder]
        IN[Input Abstraction]
        OUT[Output & Metadata]
    end
    
    subgraph "Application (Middle Layer)"
        APP[UnifiedProcessor]
        STRAT[Processing Strategies]
        CHUNK[Chunk Manager]
        POOL[Thread Pool]
    end
    
    subgraph "Domain Core (Inner Layer)"
        ALGO[Delta-Stack Algorithm]
        RULES[Language Rules]
        STATE[State Machine]
    end
    
    USER --> CLI --> ADP_CLI
    PY --> ADP_PY
    WEB --> ADP_WASM
    STREAM --> ADP_STRM
    
    ADP_CLI --> API
    ADP_PY --> API
    ADP_WASM --> API
    ADP_STRM --> API
    
    API --> APP
    CFG --> APP
    IN --> APP
    OUT --> APP
    
    APP --> STRAT
    STRAT --> CHUNK
    STRAT --> POOL
    
    CHUNK --> ALGO
    POOL --> ALGO
    ALGO --> RULES
    ALGO --> STATE
```

## Core Algorithm

The system is built around the **Delta-Stack Monoid** algorithm for parallel sentence boundary detection. For detailed mathematical foundation and implementation details, see [DELTA_STACK_ALGORITHM.md](DELTA_STACK_ALGORITHM.md).

## Key Design Decisions

### 1. Hexagonal Architecture

**Decision**: Separate domain logic from infrastructure concerns using Ports & Adapters pattern.

**Rationale**: 
- Allows pure functional core that's easy to test
- Enables multiple delivery mechanisms (CLI, Python, WASM) without duplicating logic
- Prepares for `no_std` support for embedded systems

**Trade-off**: More layers can be initially confusing for newcomers.

### 2. Rust for Core Implementation

**Decision**: Implement core algorithm in Rust with safe abstractions.

**Rationale**:
- Memory safety without garbage collection
- Zero-cost abstractions for performance
- Excellent FFI for Python/WASM bindings
- Strong ecosystem for parallel processing (rayon)

**Trade-off**: Steeper learning curve than Python/Go.

### 3. Rayon for Parallelism

**Decision**: Use rayon's work-stealing thread pool for parallel processing.

**Rationale**:
- Battle-tested in production
- Automatic load balancing
- Integrates well with Rust iterators

**Trade-off**: Not available in WASM (we fall back to sequential).

### 4. Language Rules as Traits

**Decision**: Define `LanguageRules` trait for language-specific logic.

**Rationale**:
- Easy to add new languages without modifying core
- Community can contribute language implementations
- Compile-time type safety
- Future support for runtime plugin loading

**Trade-off**: Requires careful trait design to remain stable.

### 5. Unified Public API

**Decision**: Create a separate API layer (`src/api/`) as the public interface.

**Rationale**:
- Stable public interface independent of internal changes
- Simplified usage for external consumers
- Better encapsulation of implementation details
- Easier to maintain backward compatibility

**Trade-off**: Additional abstraction layer to maintain.

## Component Structure

### API Layer (`src/api/`)

The public interface that provides a clean, stable API for external consumers:

```rust
// Main entry point
pub struct SentenceProcessor {
    // Internal implementation details hidden
}

// Unified input handling
pub enum Input {
    Text(String),
    File(PathBuf),
    Bytes(Vec<u8>),
    Reader(Box<dyn Read>),
}

// Configuration with builder pattern
pub struct Config { /* fields */ }
pub struct ConfigBuilder { /* builder */ }

// Rich output information
pub struct Output {
    pub boundaries: Vec<Boundary>,
    pub metadata: ProcessingMetadata,
}
```

Key features:
- Hides internal implementation complexity
- Provides intuitive, type-safe API
- Supports configuration presets (fast, balanced, accurate)
- Unified error handling

### Domain Layer (`src/domain/`)

The pure business logic, no external dependencies:

```rust
// Core algorithm trait
pub trait Monoid {
    fn identity() -> Self;
    fn combine(&self, other: &Self) -> Self;
}

// Language-specific rules
pub trait LanguageRules: Send + Sync {
    fn is_sentence_boundary(&self, state: &PartialState, offset: usize) -> BoundaryDecision;
    fn process_character(&self, ch: char, context: &ProcessingContext) -> CharacterEffect;
    // ... other methods
}
```

### Application Layer (`src/application/`)

Orchestrates the domain logic with various processing strategies:

```rust
// Unified processor that delegates to strategies
pub struct UnifiedProcessor {
    rules: Arc<dyn LanguageRules>,
    config: ProcessingConfig,
}

// Processing strategies
pub trait ProcessingStrategy: Send + Sync {
    fn process(&self, input: StrategyInput) -> Result<StrategyOutput>;
    fn is_suitable(&self, context: &AnalysisContext) -> SuitabilityScore;
}
```

Key responsibilities:
- Strategy selection (sequential, parallel, streaming, adaptive)
- Chunk management at valid UTF-8 boundaries
- Cross-chunk boundary resolution
- Performance optimization

### Adapter Layer

Each adapter provides a different interface to the API layer:

- **CLI** (`sakurs-cli/`): Command-line tool with file globbing and stdin support
- **Python** (`sakurs-py/`): PyO3 bindings with NLTK-compatible API
- **WASM** (future): Browser-compatible with streaming support
- **C API** (future): For integration with other languages

## Performance Characteristics

### Memory Usage

- **Sequential mode**: O(1) - Only current position state
- **Parallel mode**: O(P) - One state per thread
- **Streaming mode**: O(W) - Window size only

### Time Complexity

- **Sequential**: O(N) - Linear scan
- **Parallel**: O(N/P + log P) - Near-linear speedup
- **SIMD optimization**: ~4-8x faster for terminal detection

### Optimization Strategies

1. **SIMD for character scanning** - Uses AVX2/NEON when available
2. **Zero-copy string handling** - Minimizes allocations
3. **Cache-aware chunking** - Chunks fit in L2 cache
4. **Lock-free combining** - Tree reduction without mutexes


## Usage Examples

### Basic Usage (via API Layer)

```rust
use sakurs::{SentenceProcessor, Input};

// Simple usage
let processor = SentenceProcessor::for_language("en")?;
let output = processor.process(Input::from_text("Hello world. How are you?"))?;

for boundary in &output.boundaries {
    println!("Sentence ends at: {}", boundary.offset);
}
```

### CLI Usage

```bash
# Process files
sakurs process -i "*.txt" -f json

# Process from stdin
echo "Hello world." | sakurs process -i -

# Japanese text with custom settings
sakurs process -i doc.txt -l japanese --parallel
```

### Python Usage

```python
import sakurs

# NLTK-compatible API
sentences = sakurs.sent_tokenize(text, "en")

# Advanced usage
processor = sakurs.load("ja")
result = processor.process(text)
```

## FAQ

### Q: Why not use regex for sentence detection?

Regex cannot handle nested delimiters (parentheses within quotes within parentheses) correctly. Our state machine approach handles arbitrary nesting.

### Q: How does cross-chunk abbreviation detection work?

We track "dangling dots" at chunk boundaries and look ahead in the next chunk for alphabetic characters. If found, we merge the boundary.

### Q: Why a separate API layer?

The API layer provides a stable public interface that shields users from internal implementation changes. This allows us to refactor and optimize internals without breaking existing code.

### Q: Can I use this in production?

Yes! The library is designed for production use with:
- Comprehensive error handling
- Graceful degradation
- Extensive testing
- Performance monitoring hooks

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development setup and guidelines.

Key areas for contribution:
- Language rule implementations
- Performance optimizations
- Documentation improvements
- Test coverage expansion
