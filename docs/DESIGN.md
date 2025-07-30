# Sakurs Sentence Boundary Detection - Complete Design Document

**Version**: 2025-07-30  
**Status**: Final Design

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture](#2-architecture)
3. [Layer Design](#3-layer-design)
   - [3.1 sakurs-core Layer](#31-sakurs-core-layer)
   - [3.2 sakurs-engine Layer](#32-sakurs-engine-layer)
   - [3.3 sakurs-api Layer](#33-sakurs-api-layer)
   - [3.4 sakurs-cli Layer (Adapter)](#34-sakurs-cli-layer-adapter)
   - [3.5 sakurs-py Layer (Adapter)](#35-sakurs-py-layer-adapter)
4. [Core Components](#4-core-components)
5. [Data Flow](#5-data-flow)
6. [Execution Modes](#6-execution-modes)
7. [Language Rules System](#7-language-rules-system)
8. [Error Handling](#8-error-handling)
9. [Performance Considerations](#9-performance-considerations)
10. [Testing Strategy](#10-testing-strategy)
11. [Deployment & Distribution](#11-deployment--distribution)
12. [Future Enhancements](#12-future-enhancements)

---

## 1. Overview

Sakurs is a high-performance, rule-based sentence boundary detection library implementing the mathematically proven Δ-Stack Monoid algorithm. The system is designed with a clean hexagonal architecture, separating core business logic from external adapters.

### Key Features
- **Parallel Processing**: Near-linear speedup using Δ-Stack Monoid algorithm
- **Adaptive Execution**: Automatic selection between sequential and parallel modes
- **Multi-language Support**: Extensible through TOML configuration
- **Zero-copy Operations**: Minimal memory allocation in hot paths
- **Cross-platform**: Rust core with Python bindings and CLI

### Design Principles
- **Separation of Concerns**: Clear layer boundaries with unidirectional dependencies
- **Domain-Driven Design**: Core algorithm isolated from infrastructure
- **Performance First**: Zero-allocation hot loops, cache-friendly access
- **Extensibility**: Plugin-ready architecture for new languages and adapters

---

## 2. Architecture

```
┌────────────────────────────────────────────────────────────┐
│                      External Clients                      │
│          (Python Apps, CLI Users, Future WASM/C)           │
└─────────────────┬────────────────────┬─────────────────────┘
                  │                    │
┌─────────────────▼────────┐  ┌───────▼──────────┐
│      sakurs-cli          │  │    sakurs-py     │  } Adapter
│   (CLI Adapter Layer)    │  │ (Python Adapter) │  } Layer
└─────────────┬────────────┘  └────────┬─────────┘
              │                         │
              └────────────┬────────────┘
                           │
                  ┌────────▼────────┐
                  │   sakurs-api    │  } API Layer
                  │ (Public Facade) │  } (Stable Interface)
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │ sakurs-engine   │  } Application Layer
                  │  (Executors &   │  } (Orchestration)
                  │   Dispatchers)  │
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │  sakurs-core    │  } Domain Layer
                  │  (Algorithms &  │  } (Business Logic)
                  │     Rules)      │
                  └─────────────────┘
```

### Dependency Rules
- Dependencies flow **downward only**
- Core has **no external dependencies** (pure Rust std only)
- Each layer depends only on its immediate lower layer's public API
- Adapters depend on the API layer, never on internal layers

---

## 3. Layer Design

### 3.1 sakurs-core Layer

**Purpose**: Pure domain logic implementing the Δ-Stack Monoid algorithm and rule evaluation.

**Location**: `sakurs-core/src/core/`

**Key Components**:
```rust
// Core algorithm traits
pub trait LanguageRules: Send + Sync {
    fn is_terminator_char(&self, ch: char) -> bool;
    fn boundary_decision(&self, text: &str, pos: usize) -> BoundaryDecision;
    fn enclosure_info(&self, ch: char) -> Option<EnclosureInfo>;
}

// Delta-Stack state representation
pub struct DeltaState {
    delta: Vec<(i32, i32)>,  // (net_change, min_depth) per enclosure type
    boundaries: Vec<CandidateBoundary>,
    abbr_state: AbbreviationState,
}

// Sequential scanner (zero-allocation)
pub struct SequentialScanner;

// Parallel chunk scanner
pub struct DeltaStackScanner;

// Language rule tables (immutable after init)
pub struct LanguageTables {
    term_table: TerminatorTable,
    enc_table: EnclosureTable,
    abbr_trie: AbbreviationTrie,
    suppresser: SuppressionPatterns,
}
```

**Dependencies**: None (pure Rust std only)

**Design Decisions**:
- No heap allocations in scanning loops
- Immutable rule tables shared across threads
- Monoid operations proven mathematically associative
- UTF-8 safety guaranteed at character boundaries

---

### 3.2 sakurs-engine Layer

**Purpose**: Orchestration layer managing execution strategies and parallel coordination.

**Location**: `sakurs-core/src/engine/`

**Key Components**:
```rust
// Execution mode dispatcher
pub struct AdaptiveDispatcher {
    sequential_executor: SequentialExecutor,
    parallel_executor: ParallelExecutor,
    config: EngineConfig,
}

// Configuration for execution
pub struct EngineConfig {
    pub execution_mode: ExecutionMode,
    pub thread_count: Option<usize>,
    pub chunk_size_kb: usize,
    pub adaptive_threshold_kb: usize,
}

// Parallel execution coordinator
pub struct ParallelExecutor {
    chunk_manager: ChunkManager,
    thread_pool: ThreadPool,
}

// UTF-8 safe text chunking
pub struct ChunkManager {
    overlap_size: usize,
    min_chunk_size: usize,
}

// Performance metrics
pub struct ExecutionMetrics {
    pub mode_used: ExecutionMode,
    pub chunks_processed: usize,
    pub bytes_per_second: f64,
    pub thread_efficiency: f64,
}
```

**Dependencies**: 
- `sakurs-core` (for algorithms)
- `rayon` (for parallel execution)
- `crossbeam` (for lock-free queues)

**Design Decisions**:
- Adaptive threshold: 128KB per core default
- Chunk overlap for cross-boundary patterns
- Zero-copy chunk views into original text
- Metrics collection with minimal overhead

---

### 3.3 sakurs-api Layer

**Purpose**: Stable public API with backwards compatibility guarantees.

**Location**: `sakurs-core/src/api/`

**Key Components**:
```rust
// Main entry point
pub struct SentenceProcessor {
    engine: Arc<AdaptiveDispatcher>,
    config: ProcessorConfig,
}

// Public configuration
pub struct ProcessorConfig {
    pub language: String,
    pub custom_rules: Option<CustomRules>,
    pub performance_hints: PerformanceHints,
}

// Input abstraction
pub enum Input {
    Text(String),
    TextRef(&'static str),
    File(PathBuf),
    Reader(Box<dyn Read>),
}

// Output with metadata
pub struct Output {
    pub sentences: Vec<Sentence>,
    pub metadata: ProcessingMetadata,
}

// Error types
#[derive(Error, Debug)]
pub enum SakursError {
    #[error("Language '{0}' not supported")]
    LanguageNotSupported(String),
    
    #[error("Invalid UTF-8 at position {position}")]
    InvalidUtf8 { position: usize },
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

// Builder pattern for configuration
pub struct SentenceProcessorBuilder {
    config: ProcessorConfig,
}
```

**Dependencies**:
- `sakurs-engine` (for execution)
- `serde` (for serialization)
- `thiserror` (for error types)

**Design Decisions**:
- Semantic versioning with 1.0 stability guarantee
- Builder pattern for complex configurations
- Rich error types with recovery hints
- Input abstraction supporting multiple sources

---

### 3.4 sakurs-cli Layer (Adapter)

**Purpose**: Command-line interface adapter providing Unix-style text processing.

**Location**: `sakurs-cli/`

**Key Components**:
```rust
// CLI argument structure
#[derive(Parser)]
#[command(name = "sakurs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Process {
        #[arg(short, long)]
        input: Option<PathBuf>,
        
        #[arg(short, long, default_value = "en")]
        language: String,
        
        #[arg(long)]
        format: OutputFormat,
        
        #[arg(long)]
        parallel: bool,
        
        #[arg(long)]
        benchmark: bool,
    },
    
    ListLanguages,
    
    Validate {
        config: PathBuf,
    },
}

// Output formatters
trait OutputFormatter {
    fn format(&self, output: &Output) -> Result<String>;
}

struct JsonFormatter;
struct TextFormatter;
struct MarkdownFormatter;

// Progress reporting
struct ProgressReporter {
    bar: ProgressBar,
    update_interval: Duration,
}
```

**Dependencies**:
- `sakurs-api` (for processing)
- `clap` (for CLI parsing)
- `indicatif` (for progress bars)
- `env_logger` (for debugging)

**Design Decisions**:
- Unix philosophy: composable with pipes
- Multiple output formats for different workflows
- Progress indication for large files
- Structured logging for debugging

---

### 3.5 sakurs-py Layer (Adapter)

**Purpose**: Python bindings providing NLTK-compatible and Pythonic APIs.

**Location**: `sakurs-py/`

**Key Components**:
```python
# Python module structure
sakurs/
├── __init__.py
├── core.pyi          # Type stubs
├── processor.py      # High-level API
└── _sakurs.so       # Native extension

# Rust PyO3 bindings
#[pymodule]
fn sakurs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySentenceProcessor>()?;
    m.add_function(wrap_pyfunction!(sent_tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(load_language, m)?)?;
    Ok(())
}

#[pyclass]
struct PySentenceProcessor {
    inner: SentenceProcessor,
}

#[pymethods]
impl PySentenceProcessor {
    #[new]
    fn new(language: &str) -> PyResult<Self> {
        // ...
    }
    
    fn process(&self, text: &str) -> PyResult<Vec<String>> {
        // ...
    }
    
    fn process_with_metadata(&self, text: &str) -> PyResult<PyOutput> {
        // ...
    }
}

# Python high-level API
class SentenceTokenizer:
    def __init__(self, language='en', **kwargs):
        self._processor = _sakurs.PySentenceProcessor(language)
    
    def tokenize(self, text: str) -> List[str]:
        """NLTK-compatible sentence tokenization."""
        return self._processor.process(text)
    
    def tokenize_with_spans(self, text: str) -> List[Tuple[str, int, int]]:
        """Returns sentences with their character spans."""
        result = self._processor.process_with_metadata(text)
        return [(s.text, s.start, s.end) for s in result.sentences]
```

**Dependencies**:
- `sakurs-api` (for processing)
- `pyo3` (for Python bindings)
- `maturin` (for building wheels)

**Design Decisions**:
- NLTK-compatible function names
- Pure Python wrapper for better IDE support
- Type hints for all public APIs
- Efficient zero-copy string handling where possible

---

## 4. Core Components

### 4.1 Δ-Stack Monoid Algorithm

The mathematical foundation enabling parallel processing:

```rust
// Monoid trait implementation
impl Monoid for DeltaState {
    fn identity() -> Self {
        DeltaState {
            delta: vec![(0, 0); MAX_ENCLOSURE_TYPES],
            boundaries: vec![],
            abbr_state: AbbreviationState::default(),
        }
    }
    
    fn combine(&self, other: &Self) -> Self {
        // Associative operation
        let combined_delta = self.delta.iter()
            .zip(&other.delta)
            .map(|((net1, min1), (net2, min2))| {
                (net1 + net2, (*min1).min(net1 + min2))
            })
            .collect();
        
        DeltaState {
            delta: combined_delta,
            boundaries: merge_boundaries(&self.boundaries, &other.boundaries),
            abbr_state: self.abbr_state.merge(&other.abbr_state),
        }
    }
}
```

### 4.2 Language Rule Tables

Optimized data structures for O(1) character classification:

```rust
// Terminator table with SIMD-friendly layout
pub struct TerminatorTable {
    ascii_mask: [u8; 128],      // Bit-packed for vectorization
    unicode_terms: Vec<char>,    // Sorted for binary search
}

// Enclosure table with type information
pub struct EnclosureTable {
    // char -> (type_id | (delta << 8) | (symmetric << 15))
    mapping: HashMap<char, u16, FxBuildHasher>,
}

// Abbreviation trie for fast prefix matching
pub struct AbbreviationTrie {
    nodes: Vec<TrieNode>,        // Double-array representation
    values: Vec<AbbrevInfo>,
}
```

---

## 5. Data Flow

### 5.1 Sequential Processing Flow

```
Input Text
    │
    ▼
[sakurs-api: Input validation & encoding detection]
    │
    ▼
[sakurs-engine: AdaptiveDispatcher]
    │
    ├─> (small input detected)
    ▼
[sakurs-engine: SequentialExecutor]
    │
    ▼
[sakurs-core: SequentialScanner]
    │ for each character:
    │   - Update enclosure depths
    │   - Check terminator status
    │   - Evaluate boundary rules
    │   - Emit boundary if at depth 0
    ▼
[sakurs-api: Output formatting]
    │
    ▼
Output (sentences + metadata)
```

### 5.2 Parallel Processing Flow

```
Input Text
    │
    ▼
[sakurs-api: Input validation]
    │
    ▼
[sakurs-engine: AdaptiveDispatcher]
    │
    ├─> (large input detected)
    ▼
[sakurs-engine: ParallelExecutor]
    │
    ├─> [ChunkManager: UTF-8 safe splitting]
    │
    ▼
[Rayon parallel iterator]
    │
    ├─> [sakurs-core: DeltaStackScanner] (Chunk 1)
    ├─> [sakurs-core: DeltaStackScanner] (Chunk 2)
    ├─> [sakurs-core: DeltaStackScanner] (Chunk N)
    │
    ▼
[Prefix sum computation (O(log P))]
    │
    ▼
[Parallel reduction phase]
    │
    ▼
[Boundary merging & offset adjustment]
    │
    ▼
[sakurs-api: Output formatting]
    │
    ▼
Output (sentences + metadata)
```

---

## 6. Execution Modes

### 6.1 Mode Selection Logic

```rust
impl AdaptiveDispatcher {
    fn select_mode(&self, input_size: usize) -> ExecutionMode {
        let cores = rayon::current_num_threads();
        let bytes_per_core = input_size / cores;
        
        match self.config.execution_mode {
            ExecutionMode::Adaptive => {
                if bytes_per_core < self.config.adaptive_threshold_kb * 1024
                   && input_size < self.config.adaptive_threshold_kb * 4096 {
                    ExecutionMode::Sequential
                } else {
                    ExecutionMode::Parallel
                }
            }
            other => other, // Honor explicit mode
        }
    }
}
```

### 6.2 Performance Characteristics

| Mode | Time Complexity | Space Complexity | Overhead | Best For |
|------|----------------|------------------|----------|-----------|
| Sequential | O(N) | O(1) | ~0 ms | < 512KB texts |
| Parallel | O(N/P + log P) | O(P) | ~0.3 ms | > 512KB texts |
| Adaptive | Same as selected | Same as selected | < 100ns | All sizes |

---

## 7. Language Rules System

### 7.1 Rule Categories

```toml
# Example: English language configuration
[language]
code = "en"
name = "English"
version = "2.0"

[terminators]
chars = [".", "!", "?"]
patterns = ["!?", "?!"]

[abbreviations]
titles = ["Mr", "Mrs", "Dr", "Prof"]
academic = ["Ph.D", "M.D", "B.A"]
time = ["a.m", "p.m", "A.M", "P.M"]

[enclosures]
pairs = [
    { open = "(", close = ")", type = "parenthesis" },
    { open = "[", close = "]", type = "bracket" },
    { open = "\"", close = "\"", type = "quote", symmetric = true }
]

[sentence_starters]
pronouns = ["I", "You", "He", "She", "It", "We", "They"]
articles = ["The", "A", "An"]

[suppression]
patterns = [
    { regex = "\\b\\w+'\\w+\\b", name = "contractions" },
    { regex = "\\d+\\.\\d+", name = "decimals" }
]
```

### 7.2 Rule Evaluation Pipeline

```rust
impl LanguageRules for CompiledRules {
    fn boundary_decision(&self, text: &str, pos: usize) -> BoundaryDecision {
        // 1. Check if within ellipsis
        if self.in_ellipsis_pattern(text, pos) {
            return BoundaryDecision::Suppress;
        }
        
        // 2. Check abbreviation context
        if let Some(abbr) = self.detect_abbreviation(text, pos) {
            if self.followed_by_sentence_starter(text, pos) {
                return BoundaryDecision::Accept(Strength::FromAbbreviation);
            }
            return BoundaryDecision::Suppress;
        }
        
        // 3. Apply standard terminator rules
        self.evaluate_terminator(text, pos)
    }
}
```

---

## 8. Error Handling

### 8.1 Error Categories

```rust
// Domain errors (sakurs-core)
pub enum CoreError {
    InvalidCharacterBoundary { position: usize },
    InconsistentState { details: String },
}

// Engine errors (sakurs-engine)
pub enum EngineError {
    ChunkingFailed { reason: String },
    ThreadPoolExhausted,
    Core(CoreError),
}

// API errors (sakurs-api)
pub enum ApiError {
    UnsupportedLanguage { code: String },
    InvalidInput { reason: String },
    ConfigurationError { path: String, error: String },
    Engine(EngineError),
}
```

### 8.2 Recovery Strategies

```rust
impl SentenceProcessor {
    pub fn process_with_recovery(&self, input: Input) -> Result<Output> {
        match self.try_process(input) {
            Ok(output) => Ok(output),
            Err(ApiError::Engine(EngineError::ThreadPoolExhausted)) => {
                // Fallback to sequential processing
                self.process_sequential(input)
            }
            Err(ApiError::InvalidInput { reason }) if reason.contains("UTF-8") => {
                // Process valid portion only
                self.process_valid_utf8_prefix(input)
            }
            Err(e) => Err(e),
        }
    }
}
```

---

## 9. Performance Considerations

### 9.1 Memory Layout

```rust
// Cache-line aligned structures
#[repr(C, align(64))]
struct ChunkState {
    boundaries: SmallVec<[u32; 16]>,  // Stack allocation for small chunks
    delta: [i32; MAX_ENCLOSURE_TYPES],
    padding: [u8; PADDING_SIZE],
}

// Zero-copy text views
struct TextChunk<'a> {
    text: &'a str,
    global_offset: usize,
}
```

### 9.2 Optimization Techniques

- **Branchless terminator checking**: ASCII lookup table
- **SIMD opportunities**: Batch character classification
- **Memory pooling**: Reuse allocations across chunks
- **Lock-free queues**: Work distribution without contention

---

## 10. Testing Strategy

### 10.1 Test Categories

| Category | Layer | Tools | Coverage Target |
|----------|-------|-------|----------------|
| Unit Tests | core | `cargo test` | 95% |
| Property Tests | core | `proptest` | Invariants |
| Integration | engine | Custom harness | 90% |
| Performance | engine | `criterion` | Regression detection |
| End-to-End | api | Test corpora | Language accuracy |
| Fuzzing | all | `cargo-fuzz` | Edge cases |

### 10.2 Key Test Scenarios

```rust
#[test]
fn test_sequential_parallel_equivalence() {
    proptest!(|(text: String)| {
        let seq_result = process_sequential(&text);
        let par_result = process_parallel(&text);
        assert_eq!(seq_result, par_result);
    });
}

#[test]
fn test_utf8_boundary_safety() {
    // Test with multi-byte characters at chunk boundaries
    let text = "Hello 世界. This is 测试.";
    let chunks = ChunkManager::new(8).split(text);
    for chunk in chunks {
        assert!(chunk.text.is_char_boundary(0));
        assert!(chunk.text.is_char_boundary(chunk.text.len()));
    }
}
```

---

## 11. Deployment & Distribution

### 11.1 Package Structure

```
sakurs/
├── Cargo.toml              # Workspace root
├── sakurs-core/
│   ├── Cargo.toml          # no external deps
│   └── src/
├── sakurs-engine/
│   ├── Cargo.toml          # rayon, crossbeam
│   └── src/
├── sakurs-api/
│   ├── Cargo.toml          # serde, thiserror
│   └── src/
├── sakurs-cli/
│   ├── Cargo.toml          # clap, indicatif
│   └── src/
└── sakurs-py/
    ├── Cargo.toml          # pyo3
    ├── pyproject.toml      # maturin config
    └── src/
```

### 11.2 Distribution Channels

- **Rust**: crates.io (`sakurs`, `sakurs-cli`)
- **Python**: PyPI (`sakurs`)
- **Binary**: GitHub releases (cross-platform)
- **Docker**: `sakurs/sakurs:latest`

---

## 12. Future Enhancements

### 12.1 Planned Features

1. **Additional Language Support**
   - German, French, Spanish configurations
   - Right-to-left languages (Arabic, Hebrew)
   - CJK-specific optimizations

2. **New Adapters**
   - WASM for browser deployment
   - C API for FFI integration
   - gRPC service adapter

3. **Performance Enhancements**
   - GPU acceleration exploration
   - SIMD optimizations for x86/ARM
   - Persistent caching layer

4. **Advanced Features**
   - Confidence scores for boundaries
   - Streaming mode for real-time processing
   - Custom rule plugins

### 12.2 Extension Points

```rust
// Plugin interface (future)
pub trait RulePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(&self, context: &Context) -> PluginResult;
}

// GPU acceleration (future)
pub trait GpuExecutor {
    fn process_batch(&self, texts: &[String]) -> Vec<Output>;
}
```

---

## Appendix A: Glossary

- **Δ-Stack**: Delta representation of enclosure depth changes
- **Monoid**: Algebraic structure with associative operation
- **SBD**: Sentence Boundary Detection
- **Chunk**: UTF-8 safe text segment for parallel processing
- **Enclosure**: Paired delimiters (parentheses, quotes, etc.)

## Appendix B: References

1. "Δ-Stack Monoid Algorithm" - NAACL 2024
2. Hexagonal Architecture - Alistair Cockburn
3. Rust API Guidelines - rust-lang.github.io
4. PyO3 User Guide - pyo3.rs