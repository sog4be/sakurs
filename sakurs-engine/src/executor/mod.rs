//! Execution strategies for text processing

use crate::error::Result;
use sakurs_core::{Boundary, LanguageRules};
use std::time::Duration;

#[cfg(feature = "parallel")]
pub mod parallel;
pub mod sequential;
pub mod streaming;

// Re-export executors
#[cfg(feature = "parallel")]
pub use parallel::ParallelExecutor;
pub use sequential::SequentialExecutor;
pub use streaming::StreamingExecutor;

/// Execution mode selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Single-threaded sequential processing
    Sequential,
    /// Multi-threaded parallel processing
    Parallel,
    /// Memory-efficient streaming
    Streaming,
    /// Automatically select between Sequential and Parallel based on input size
    Adaptive,
}

/// Performance metrics for execution
#[derive(Debug, Clone)]
pub struct ExecutionMetrics {
    /// The execution mode that was actually used
    pub mode_used: ExecutionMode,
    /// Number of chunks processed (for parallel/streaming modes)
    pub chunks_processed: usize,
    /// Processing throughput in bytes per second
    pub bytes_per_second: f64,
    /// Thread efficiency (0.0 to 1.0, only meaningful for parallel mode)
    pub thread_efficiency: f64,
    /// Total processing time
    pub processing_time: Duration,
    /// Total bytes processed
    pub bytes_processed: usize,
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            mode_used: ExecutionMode::Sequential,
            chunks_processed: 1,
            bytes_per_second: 0.0,
            thread_efficiency: 1.0,
            processing_time: Duration::from_secs(0),
            bytes_processed: 0,
        }
    }
}

/// Extended result with execution metadata
#[derive(Debug, Clone)]
pub struct ProcessingOutput {
    /// Detected sentence boundaries
    pub boundaries: Vec<Boundary>,
    /// Processing metadata
    pub metadata: ExecutionMetrics,
}

/// Trait for execution strategies
pub trait Executor: Send + Sync {
    /// Process text and return boundaries with metadata
    fn process_with_metadata<R: LanguageRules>(
        &self,
        text: &str,
        rules: &R,
    ) -> Result<ProcessingOutput>;

    /// Process text and return boundaries (legacy API)
    fn process<R: LanguageRules>(&self, text: &str, rules: &R) -> Result<Vec<Boundary>> {
        self.process_with_metadata(text, rules)
            .map(|output| output.boundaries)
    }

    /// Get the execution mode
    fn mode(&self) -> ExecutionMode;
}

/// Automatically select execution mode based on text size and config
/// Implements the adaptive logic specified in DESIGN.md Section 6.1
pub fn auto_select(text_len: usize, config: &crate::config::EngineConfig) -> ExecutionMode {
    use crate::config::ChunkPolicy;

    // If execution mode is not adaptive, return as configured
    if config.execution_mode != ExecutionMode::Adaptive {
        return config.execution_mode;
    }

    // Check if streaming is explicitly configured
    if matches!(config.chunk_policy, ChunkPolicy::Streaming { .. }) {
        return ExecutionMode::Streaming;
    }

    // Adaptive logic per DESIGN.md Section 6.1:
    // - Sequential if bytes_per_core < threshold AND input_size < threshold * 4
    // - Parallel otherwise
    let cores = rayon::current_num_threads().max(1);
    let bytes_per_core = text_len / cores;

    // Use configured threshold or default 128KB
    let adaptive_threshold_bytes = config.adaptive_threshold.unwrap_or(128 * 1024);
    let max_sequential_size = adaptive_threshold_bytes * 4; // 4x threshold

    if bytes_per_core < adaptive_threshold_bytes && text_len < max_sequential_size {
        ExecutionMode::Sequential
    } else {
        ExecutionMode::Parallel
    }
}
