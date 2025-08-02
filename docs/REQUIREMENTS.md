# Sakurs: Rule-Based Sentence Boundary Detection Library - Requirements Specification

**Version 2025-07-30**

## Executive Summary

Sakurs is a high-performance, rule-based sentence boundary detection (SBD) library that implements the mathematically proven Δ-Stack Monoid algorithm for parallel text processing. The library provides accurate sentence segmentation across multiple languages through a configurable rule system, while maintaining perfect accuracy through parallel execution.

**New in this revision:** An adaptive execution strategy that automatically selects between sequential and parallel processing modes based on input size and available CPU resources, ensuring optimal performance across all workload sizes.

## 1. Product Overview

### 1.1 Purpose
Provide a fast, accurate, and scalable solution for sentence boundary detection in natural language text, supporting multiple languages through configurable rule sets without requiring machine learning models.

### 1.2 Key Differentiators
- **Mathematical Foundation**: Based on the Δ-Stack Monoid algorithm enabling true parallel processing
- **Rule-Based Approach**: Deterministic, explainable results without ML model dependencies
- **Performance at Scale**: Near-linear speedup on multicore systems
- **Language Extensibility**: Simple TOML configuration for adding new languages
- **Adaptive Execution**: Automatic selection between sequential and parallel modes for optimal performance

### 1.3 Target Users
- NLP researchers and engineers requiring fast sentence segmentation
- Applications processing large text corpora
- Multilingual text processing pipelines
- Systems requiring deterministic, explainable segmentation
- Applications with varying workload sizes requiring consistent performance

## 2. Functional Requirements

### 2.1 Core Algorithm

#### 2.1.1 Δ-Stack Monoid Implementation
- **Requirement**: Implement the Δ-Stack Monoid algorithm as specified in the NAACL 2024 paper
- **Details**:
  - Monoid structure with associative combine operation
  - State representation as triple ⟨B, Δ, A⟩ (Boundaries, Delta Stack, Abbreviation State)
  - Parallel-safe operations maintaining mathematical properties

#### 2.1.2 Sentence Boundary Detection
- **Requirement**: Accurately detect sentence boundaries based on language-specific rules
- **Criteria**:
  - Terminal punctuation detection (., !, ?)
  - Context-aware boundary decisions
  - Handling of edge cases (abbreviations, ellipsis, quotations)

#### 2.1.3 Parallel Processing
- **Requirement**: Enable parallel text processing while maintaining accuracy
- **Implementation**:
  - Chunk-based parallel processing
  - Tree reduction for combining results
  - Cross-chunk boundary resolution
  - UTF-8 safe chunking

#### 2.1.4 Adaptive Execution Strategy (NEW)
- **Requirement**: Automatically select optimal execution mode based on workload characteristics
- **Implementation Details**:
  - **Threshold Formula**: `bytes_per_core = total_bytes / available_logical_cpus`
  - **Decision Logic**: 
    - Use SequentialScan if `bytes_per_core < 128 KiB` AND `total_bytes < 512 KiB`
    - Use DeltaStackParallel otherwise
  - **Rationale**: Measurements show Δ-Stack fixed overhead ≈ 0.3 ms + 2 allocations; below 128 KiB per core, sequential is faster
  - **Configuration**:
    - Default thresholds as above
    - Configurable via `EngineConfig::adaptive_threshold(bytes_per_core, total_bytes)`
    - CLI override: `--threshold <size>`
    - Python override: `processor = sakurs.load(..., threshold_kb=64)`
  - **Telemetry**: Selected mode and decision metrics returned in `Output.metadata.execution_mode`
  - **Manual Override**: Support for `execution_mode="sequential"|"parallel"|"adaptive"` parameters
  - **Consistency**: Both execution paths must return identical boundary sets

### 2.2 Language Support

#### 2.2.1 Core Language Rule Components

The system implements a comprehensive rule-based approach to sentence boundary detection through the following components:

##### Terminator Rules
- **Basic Terminators**: Single character punctuation marks (., !, ?, 。, ！, ？)
- **Pattern Terminators**: Multi-character patterns with specific semantics
  - `!?` (surprised question) - Always creates strong boundary
  - `?!` (questioning exclamation) - Always creates strong boundary
- **Context-Aware Evaluation**:
  - Strong boundaries: `!`, `?`, and pattern terminators
  - Weak boundaries: `.` (unless part of decimal number)
  - Decimal number detection: Suppresses boundary when period appears between digits

##### Abbreviation Detection System
- **Trie-Based Matching**: High-performance prefix tree for O(1) lookups
- **Categories**: Organized abbreviation lists (titles, academic, business, time, geographic, etc.)
- **Word Boundary Enforcement**: Abbreviations must start at word boundaries
- **Multi-Period Abbreviations**: Special handling for patterns like "U.S.A.", "Ph.D."
  - Detects sequences: letter(1-2) + period + whitespace + letter(1-2) + period
  - Prevents false boundaries within these patterns
- **Sentence Starter Integration**: Abbreviations followed by sentence starters create boundaries

##### Ellipsis Handling
- **Pattern Recognition**: Configurable patterns ("...", "…", "....")
- **Context Rules**:
  - `followed_by_capital`: Creates boundary when followed by capital letter
  - `followed_by_lowercase`: Suppresses boundary when followed by lowercase
- **Exception Patterns**: Regex-based exceptions (e.g., "um...", "uh...", "er...")
- **Incomplete Pattern Detection**: Prevents boundaries at first/middle dots of ellipsis

##### Enclosure Management
- **Paired Delimiters**: Tracks nested depth for each enclosure type
  - Standard: `()`, `[]`, `{}`
  - Language-specific: `「」`, `『』`, `（）` (Japanese)
  - Symmetric: Quotes that use same character for open/close
- **Depth Tracking**: Maintains separate depth counter for each enclosure type
- **Boundary Suppression**: No sentence boundaries inside any enclosure
- **Symmetric Quote Handling**:
  - Depth 0→1: Opening quote
  - Depth 1→0: Closing quote
  - Depth ≥2: Ignored (requires ML approach)

##### Suppression Patterns
- **Fast Patterns**: Single-character context matching
  - Apostrophe suppression: `'` between alphabetic characters (contractions)
  - Possessive detection: `'` after 's' before whitespace (partial support)
  - List item suppression: `)` at line start after alphanumeric
- **Regex Patterns**: Complex pattern matching
  - Numbered references: `(1)`, `(12)`, `(123)`
  - Measurements: `5'9"`, `45°30'`, `6'`, `12"`
  - Possessives: `John's`, `students'`

##### Sentence Starter Detection
- **Word Lists by Category**:
  - Pronouns: I, You, He, She, It, We, They
  - Articles: The, A, An
  - Conjunctions: However, But, And, So, Therefore, etc.
  - Interrogatives: What, When, Where, Why, How, etc.
- **Context Requirements**:
  - Minimum word length enforcement
  - Optional following space requirement (prevents "Theater" matching "The")
  - Case-sensitive exact matching

#### 2.2.2 Rule Application Flow

1. **Ellipsis Check**: First checks if position is part of ellipsis pattern
2. **Incomplete Pattern Detection**: Prevents boundaries within multi-character patterns
3. **Multi-Period Abbreviation Check**: Detects patterns like "U.S.A."
4. **Terminator Pattern Matching**: Checks for multi-character terminator patterns
5. **Abbreviation Resolution**: 
   - Detects abbreviation at position
   - If found, checks following word against sentence starters
   - Creates boundary only if followed by sentence starter or EOF
6. **Default Terminator Evaluation**: Applies standard rules for single terminators

#### 2.2.3 Built-in Language Configurations

##### English Configuration
- **Terminators**: `.`, `!`, `?`
- **Patterns**: `!?`, `?!`
- **Abbreviations**: 
  - 70+ common abbreviations across 8 categories
  - Special handling for multi-period forms (Ph.D., U.S.A.)
- **Sentence Starters**: 160+ words across 8 categories
- **Enclosures**: 5 types including symmetric quotes
- **Suppression**: Contractions, possessives, list items, measurements

##### Japanese Configuration
- **Terminators**: `。`, `！`, `？`, `!`, `?` (full and half-width)
- **Patterns**: `！？`, `？！`, `!?`, `?!`
- **Enclosures**: 9 types of Japanese brackets and quotes
- **Suppression**: Line-start parentheses
- **No abbreviations or sentence starters** (not applicable to Japanese)

#### 2.2.4 Language Extensibility
- **TOML Configuration**: Declarative rule definition
- **Compile-Time Embedding**: Zero runtime overhead
- **Validation**: Configuration validation at load time
- **Categories**: Flexible categorization for maintainability

#### 2.2.5 Supported Sentence Boundary Detection Patterns

The rule-based system can accurately detect sentence boundaries in the following scenarios:

##### Standard Sentences
- **Simple declarative**: "The cat sat on the mat."
- **Exclamatory**: "What a beautiful day!"
- **Interrogative**: "Where are you going?"
- **Multiple terminators**: "Really?!" "No way!?"

##### Abbreviation Handling
- **Title abbreviations**: "Dr. Smith arrived. The patient waited."
- **Business abbreviations**: "Apple Inc. announced new products."
- **Geographic abbreviations**: "He lives in Washington D.C. The city is beautiful."
- **Academic degrees**: "She has a Ph.D. Her research is groundbreaking."
- **Multi-period abbreviations**: "The U.S.A. is large. Canada is north of it."

##### Ellipsis Patterns
- **Standard ellipsis**: "He paused... Then continued."
- **Trailing thought**: "I wonder..." (boundary at end)
- **Contextual ellipsis**: "Wait... Why did he leave?" (boundary due to capital)
- **Non-boundary ellipsis**: "She said... well, never mind." (no boundary due to lowercase)

##### Nested Structures
- **Parenthetical expressions**: "The result (as shown in Fig. 1) was significant."
- **Quoted speech**: 'He said "Hello there." Then he left.'
- **Nested quotes**: "She replied, 'He said "No way!" to me.'"
- **Multiple enclosure types**: "The formula [x = (a + b) * c] is correct."

##### Special Patterns
- **Decimal numbers**: "The price is $3.50 per unit." (no false boundary at decimal)
- **List items**: "1) First item" (suppressed at line start)
- **Contractions**: "It's raining. That's unfortunate." (apostrophes don't interfere)
- **Possessives**: "That is James' book." (possessive apostrophe handled)
- **Measurements**: "The board is 6' long." (measurement marks suppressed)

##### Cross-Chunk Patterns (Parallel Processing)
- **Abbreviations at chunk boundaries**: Text split at "Dr." + "Smith" correctly merged
- **Ellipsis across chunks**: "..." pattern correctly identified even when split
- **Nested structures**: Parentheses depth maintained across chunk boundaries

##### Complex Real-World Examples
1. **Mixed abbreviations and quotes**: 
   - Input: 'Dr. Johnson said "The U.S.A. is vast." He meant it.'
   - Output: 2 sentences (after "vast." and "meant it.")

2. **Nested structures with abbreviations**:
   - Input: "The company (Apple Inc.) announced profits. Sales increased."
   - Output: 2 sentences (after "profits." and "increased.")

3. **Ellipsis with context**:
   - Input: "He thought... Was it true? Perhaps... Indeed it was."
   - Output: 3 sentences (after "true?", not after first "...", after "was.")

4. **Multi-language punctuation** (Japanese):
   - Input: "これは日本語です。「こんにちは。」と言った。"
   - Output: 2 sentences (after "です。" and "言った。")

##### Limitations Requiring ML/Advanced Approaches
- **Ambiguous abbreviations without clear context**
- **Domain-specific abbreviations not in configuration**
- **Complex nested symmetric quotes beyond depth 2**
- **Sentence fragments in informal text**
- **Poetry or artistic text with unconventional punctuation**

### 2.3 API Requirements

#### 2.3.1 Core API (Rust)
- **SentenceProcessor**: Main entry point
  ```rust
  pub enum ExecutionMode { 
      Sequential, 
      Parallel, 
      Adaptive  // Default
  }
  
  pub struct EngineConfig {
      pub language: String,
      pub threads: Option<usize>,
      pub chunk_kb: Option<usize>,
      pub threshold_kb: Option<usize>,      // NEW
      pub execution_mode: ExecutionMode,
  }
  
  pub struct SentenceProcessor {
      // Implementation details hidden
  }
  
  impl SentenceProcessor {
      pub fn new() -> Self;
      pub fn with_config(config: EngineConfig) -> Result<Self>;
      pub fn with_language(lang_code: &str) -> Result<Self>;
      pub fn process(input: Input) -> Result<Output>;
      pub fn process_auto(&self, input: Input) -> Result<Output>; // NEW: selects mode per F-ALG-04
  }
  ```

- **Input Types**:
  - Text strings
  - File paths
  - Byte arrays
  - Reader streams

- **Output Format**:
  - Boundary positions (byte and character offsets)
  - Processing metadata (performance stats)
  - Boundary confidence/flags
  - **NEW**: Execution mode information
    ```rust
    pub struct ProcessingMetadata {
        pub execution_mode: ExecutionMode,  // final mode used
        pub bytes_per_core: usize,
        pub total_bytes: usize,
        pub duration_ms: f64,
        pub thread_count: usize,
    }
    ```

- **Configuration**:
  - Language selection
  - Thread count control
  - Chunk size tuning
  - Execution mode (sequential/parallel/adaptive)
  - **NEW**: Adaptive threshold tuning

#### 2.3.2 CLI Interface
- **Command**: `sakurs process`
- **Features**:
  - File and stdin input
  - Glob pattern support
  - Multiple output formats (text, JSON, markdown)
  - Progress indication
  - Language selection
  - Performance tuning options
  - **NEW**: Execution mode control
    ```bash
    sakurs process file.txt --adaptive          # default
    sakurs process file.txt --sequential
    sakurs process file.txt --parallel --threads 8
    sakurs process file.txt --threshold 64KB    # override adaptive threshold
    ```

#### 2.3.3 Python Bindings
- **NLTK-Compatible API**:
  ```python
  sentences = sakurs.split(text, language="en")  # adaptive by default
  sentences = sakurs.sent_tokenize(text)
  ```

- **Advanced API**:
  ```python
  # Default adaptive mode
  processor = sakurs.load("en")
  
  # Override threshold
  processor = sakurs.load("en", threshold_kb=64)
  
  # Force execution mode
  sentences = sakurs.split(text, execution_mode="sequential")
  sentences = sakurs.split(text, execution_mode="parallel")
  
  # Process with metadata
  result = processor.process(text)
  print(f"Mode used: {result.metadata.execution_mode}")
  for sentence in result:
      print(f"{sentence.text} [{sentence.start}:{sentence.end}]")
  ```

- **Streaming Support**:
  ```python
  for sentence in sakurs.split_large_file("huge.txt", max_memory_mb=50):
      process(sentence)
  ```

### 2.4 Performance Requirements

#### 2.4.1 Speed
- Sequential: O(N) linear time complexity
- Parallel: O(N/P + log P) with P processors
- Near-linear speedup up to available cores
- **NEW**: Adaptive mode must select fastest path within 5% of oracle on corpora ranging 1 KiB – 1 GiB

#### 2.4.2 Memory
- Sequential: O(1) constant memory
- Parallel: O(P) linear in processor count
- Streaming: Configurable memory limit
- **NEW**: Sequential mode when selected has no Δ-Stack allocations

#### 2.4.3 Scalability
- Support for gigabyte-sized text files
- Efficient handling of 10K+ sentences
- Adaptive execution mode selection
- **NEW**: Validate up to 32 physical cores & 10 GiB text

#### 2.4.4 Adaptive Performance
- Adaptive decision executed in < 100 ns (pure arithmetic, no syscalls)
- Crossover point accuracy: ≤ 5% performance penalty vs. optimal static choice

## 3. Non-Functional Requirements

### 3.1 Architecture

#### 3.1.1 Hexagonal Architecture
- **Domain Layer**: Pure algorithm implementation
- **Application Layer**: Orchestration and execution strategies (including adaptive logic)
- **Adapter Layer**: External interfaces (CLI, Python, future WASM)
- **API Layer**: Stable public interface

#### 3.1.2 Design Principles
- Separation of concerns
- Dependency inversion
- Interface segregation
- Single responsibility

### 3.2 Code Quality

#### 3.2.1 Language Standards
- Rust 1.81+ with edition 2021
- Safe Rust preferred, unsafe blocks documented
- Comprehensive error handling with thiserror
- Zero-copy operations where possible

#### 3.2.2 Testing
- Unit tests for core algorithm
- Integration tests for language rules
- Property-based testing with proptest
- Benchmarks with criterion
- Minimum 90% test coverage
- **NEW**: Correctness tests comparing sequential vs parallel outputs
- **NEW**: Adaptive threshold validation across hardware profiles

#### 3.2.3 Documentation
- Rustdoc for all public APIs
- Architecture documentation
- Algorithm explanation
- Language configuration guide
- Contributing guidelines
- **NEW**: Adaptive execution tuning guide

### 3.3 Build and Distribution

#### 3.3.1 Rust Crate
- Published to crates.io as `sakurs-core`
- Cargo workspace structure
- Feature flags for optional dependencies

#### 3.3.2 CLI Tool
- Published as `sakurs-cli`
- Single binary distribution
- Cross-platform support (Linux, macOS, Windows)

#### 3.3.3 Python Package
- Published to PyPI as `sakurs`
- Wheels for major platforms
- Python 3.9+ support
- Type hints included

### 3.4 Reliability

#### 3.4.1 Error Handling
- Graceful degradation on errors
- Clear error messages
- Recovery from partial failures
- Input validation

#### 3.4.2 UTF-8 Safety
- Proper handling of Unicode text
- Safe chunking at character boundaries
- Support for all Unicode scripts

#### 3.4.3 Determinism
- Identical results for same input
- Reproducible across platforms
- No randomness in core algorithm
- **NEW**: Sequential and parallel modes produce identical results

## 4. Implementation Constraints

### 4.1 Technology Stack
- **Core**: Rust with workspace structure
- **Parallelism**: Rayon for work-stealing
- **Configuration**: TOML with serde
- **Python Bindings**: PyO3
- **CLI**: Clap for argument parsing
- **Error Handling**: thiserror and anyhow

### 4.2 Dependencies
- Minimal external dependencies
- Security-audited crates only
- No network dependencies
- No ML framework dependencies

### 4.3 Platform Support
- **Operating Systems**: Linux, macOS, Windows
- **Architectures**: x86_64, ARM64
- **Rust**: MSRV 1.81
- **Python**: 3.9+

### 4.4 Implementation Notes
- **Core** remains Δ-Stack-free when SequentialScan is selected—no extra allocations
- SequentialScan re-uses the same `LanguageRules` tables; only the orchestration differs
- Adaptive decision executed in < 100 ns (pure arithmetic, no syscalls)

## 5. Validation Requirements

### 5.1 Correctness
- Algorithm maintains monoid properties
- Parallel results match sequential
- Language rules applied consistently
- UTF-8 boundaries preserved
- **NEW**: Random texts comparison between SequentialScan vs DeltaStackParallel outputs (100% identical)

### 5.2 Performance
- Benchmark suite with criterion
- Performance regression detection
- Scaling tests up to 16 cores
- Memory usage profiling
- **NEW**: Adaptive mode accuracy (≤ 5% slowdown vs best static choice)

### 5.3 Language Accuracy
- Test corpus for each language
- Edge case coverage
- Cross-chunk boundary tests
- Abbreviation handling validation

## 6. Future Extensibility

### 6.1 Planned Features
- Additional languages (German, French, Spanish)
- WASM adapter for browser usage
- C API for other language bindings
- GPU acceleration exploration
- SIMD optimizations
- **NEW**: Adaptive logic may later incorporate measured throughput per core or GPU availability

### 6.2 Architecture Extensions
- Plugin system for custom rules
- Machine learning integration points
- Real-time streaming mode
- Distributed processing support
- **NEW**: ExecutionMode enum already anticipates additional variants (e.g., `GpuParallel`)

## 7. Success Criteria

### 7.1 Functional Success
- ✓ Accurate sentence boundary detection
- ✓ Multi-language support
- ✓ Parallel processing capability
- ✓ Easy language extensibility
- ✓ Adaptive execution for optimal performance across all workload sizes

### 7.2 Performance Success
- ✓ Near-linear parallel speedup
- ✓ Constant memory streaming
- ✓ Gigabyte file support
- ✓ Sub-second processing for typical documents
- ✓ Adaptive mode delivers equal or better latency than user-forced modes for ≥ 90% of benchmark cases

### 7.3 Adoption Success
- ✓ Simple API for common use cases
- ✓ Comprehensive documentation
- ✓ Multiple language bindings
- ✓ Active maintenance and support

## 8. Risk Mitigation

### 8.1 Technical Risks
- **Complex Unicode**: Extensive UTF-8 testing
- **Language Ambiguity**: Conservative rule design
- **Performance Regression**: Automated benchmarking
- **API Stability**: Semantic versioning
- **NEW - Poor threshold tuning on exotic hardware**: Expose CLI/SDK override, ship telemetry for feedback

### 8.2 Adoption Risks
- **Learning Curve**: Clear examples and tutorials
- **Migration Path**: NLTK-compatible API
- **Language Support**: Community contribution guide
- **Performance Concerns**: Benchmark comparisons

## 9. Compliance and Standards

### 9.1 Software Standards
- Semantic versioning (SemVer)
- Conventional commits
- MIT license
- Security policy

### 9.2 Language Standards
- Unicode compliance
- Locale-aware processing
- Script-agnostic core algorithm
- BCP 47 language codes

## 10. Conclusion

Sakurs provides a mathematically sound, high-performance solution for rule-based sentence boundary detection. By combining the innovative Δ-Stack Monoid algorithm with a flexible configuration system and intelligent adaptive execution, it achieves both accuracy and scalability while remaining accessible to users through clean APIs and comprehensive documentation.

The adaptive execution strategy ensures optimal performance across all workload sizes—from interactive single-sentence processing to massive corpus analysis—while the architecture supports future extensions and maintains the core promise of fast, accurate, and explainable sentence segmentation across multiple languages.