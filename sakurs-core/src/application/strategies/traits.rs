//! Unified traits for text processing strategies
//!
//! This module combines parsing and processing strategies into a unified interface.

use crate::application::config::ProcessingResult as Result;
use crate::domain::{language::LanguageRules, state::PartialState};
use std::sync::Arc;

/// Input for strategy operations
pub enum StrategyInput<'a> {
    /// Complete text in memory
    Text(&'a str),
    /// File path for file-based processing
    File(std::path::PathBuf),
    /// Stream for streaming processing
    Stream(Box<dyn std::io::Read + Send + 'a>),
    /// Pre-chunked text
    Chunks(Vec<&'a str>),
}

/// Output from strategy operations
pub enum StrategyOutput {
    /// Sentence boundary offsets
    Boundaries(Vec<usize>),
    /// Partial states for further processing
    States(Vec<PartialState>),
    /// Streaming iterator for large outputs
    StreamingBoundaries(Box<dyn Iterator<Item = Result<usize>> + Send>),
}

/// Unified trait for text processing strategies
pub trait ProcessingStrategy: Send + Sync {
    /// Process input with this strategy
    fn process(
        &self,
        input: StrategyInput,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<StrategyOutput>;

    /// Estimate if this strategy is suitable for given input
    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32;

    /// Get optimal configuration for this strategy
    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig;

    /// Check if this strategy supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Check if this strategy supports parallel processing
    fn supports_parallel(&self) -> bool {
        false
    }

    /// Strategy name for debugging and metrics
    fn name(&self) -> &'static str;
}

/// Configuration parameters for processing strategies
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    /// Size of chunks for processing
    pub chunk_size: usize,
    /// Number of threads for parallel processing
    pub thread_count: usize,
    /// Buffer size for streaming operations
    pub buffer_size: usize,
    /// Overlap size for cross-chunk context
    pub overlap_size: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 65536,      // 64KB default
            thread_count: 1,        // Sequential by default
            buffer_size: 4_194_304, // 4MB for streaming
            overlap_size: 256,      // For context preservation
        }
    }
}

impl ProcessingConfig {
    /// Create config for sequential processing
    pub fn sequential() -> Self {
        Self {
            thread_count: 1,
            chunk_size: 131_072,    // 128KB for sequential
            buffer_size: 4_194_304, // 4MB for streaming
            overlap_size: 256,      // For context preservation
        }
    }

    /// Create config for parallel processing
    pub fn parallel(thread_count: usize) -> Self {
        Self {
            thread_count,
            chunk_size: 65536,      // 64KB per thread
            buffer_size: 4_194_304, // 4MB for streaming
            overlap_size: 256,      // For context preservation
        }
    }

    /// Create config for streaming processing
    pub fn streaming() -> Self {
        Self {
            thread_count: 1,
            buffer_size: 8_388_608, // 8MB buffer
            chunk_size: 1_048_576,  // 1MB chunks
            overlap_size: 512,      // Larger overlap for streaming
        }
    }
}

/// Characteristics of the input
#[derive(Debug, Clone)]
pub struct InputCharacteristics {
    /// Size of input in bytes
    pub size_bytes: usize,
    /// Estimated character count
    pub estimated_char_count: usize,
    /// Whether input is from a stream
    pub is_streaming: bool,
    /// Available system memory in bytes
    pub available_memory: usize,
    /// Number of CPU cores
    pub cpu_count: usize,
    /// Language hint if available
    pub language_hint: Option<String>,
}

impl InputCharacteristics {
    /// Create characteristics for text input
    pub fn from_text(text: &str) -> Self {
        Self {
            size_bytes: text.len(),
            estimated_char_count: text.chars().count(),
            is_streaming: false,
            available_memory: Self::estimate_available_memory(),
            cpu_count: num_cpus::get(),
            language_hint: None,
        }
    }

    /// Create characteristics for file input
    pub fn from_file_metadata(metadata: &std::fs::Metadata) -> Self {
        Self {
            size_bytes: metadata.len() as usize,
            estimated_char_count: metadata.len() as usize / 3, // Rough estimate
            is_streaming: false,
            available_memory: Self::estimate_available_memory(),
            cpu_count: num_cpus::get(),
            language_hint: None,
        }
    }

    /// Create characteristics for streaming input
    pub fn streaming() -> Self {
        Self {
            size_bytes: usize::MAX,
            estimated_char_count: usize::MAX,
            is_streaming: true,
            available_memory: Self::estimate_available_memory(),
            cpu_count: num_cpus::get(),
            language_hint: None,
        }
    }

    /// Estimate available system memory
    fn estimate_available_memory() -> usize {
        // Simple heuristic: assume 1GB is available
        // In production, this could use system APIs
        1_073_741_824
    }

    /// Check if input is considered small
    pub fn is_small(&self) -> bool {
        self.size_bytes < 100_000 // < 100KB
    }

    /// Check if input is considered medium
    pub fn is_medium(&self) -> bool {
        self.size_bytes >= 100_000 && self.size_bytes <= 10_000_000 // 100KB - 10MB
    }

    /// Check if input is considered large
    pub fn is_large(&self) -> bool {
        self.size_bytes > 10_000_000 // > 10MB
    }

    /// Check if parallel processing would be beneficial
    pub fn would_benefit_from_parallel(&self) -> bool {
        self.size_bytes > 500_000 && self.cpu_count > 1 && !self.is_streaming
    }

    /// Check if streaming is required
    pub fn requires_streaming(&self) -> bool {
        self.is_streaming || self.size_bytes > self.available_memory / 4
    }
}

/// Strategy selection criteria
#[derive(Debug, Clone, Copy)]
pub enum StrategyType {
    /// Sequential processing for small inputs
    Sequential,
    /// Parallel processing for medium to large inputs
    Parallel,
    /// Streaming processing for very large inputs
    Streaming,
    /// Adaptive selection based on input
    Adaptive,
}

/// Result of strategy selection
#[derive(Debug)]
pub struct StrategySelection {
    /// Selected strategy type
    pub strategy_type: StrategyType,
    /// Suitability score (0.0 to 1.0)
    pub score: f32,
    /// Reasoning for selection
    pub reason: String,
}
