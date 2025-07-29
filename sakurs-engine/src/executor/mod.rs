//! Execution strategies for text processing

use crate::error::Result;
use sakurs_core::{Boundary, LanguageRules};

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
}

/// Trait for execution strategies
pub trait Executor: Send + Sync {
    /// Process text and return boundaries
    fn process<R: LanguageRules>(&self, text: &str, rules: &R) -> Result<Vec<Boundary>>;

    /// Get the execution mode
    fn mode(&self) -> ExecutionMode;
}

/// Automatically select execution mode based on text size and config
pub fn auto_select(text_len: usize, config: &crate::config::EngineConfig) -> ExecutionMode {
    use crate::config::ChunkPolicy;

    // Check if streaming is explicitly configured
    if matches!(config.chunk_policy, ChunkPolicy::Streaming { .. }) {
        return ExecutionMode::Streaming;
    }

    // Check for very small texts
    if text_len < 1024 {
        return ExecutionMode::Sequential;
    }

    // Check against parallel threshold
    if text_len < config.parallel_threshold {
        return ExecutionMode::Sequential;
    }

    // Check if parallel is disabled via thread count
    if config.threads == Some(1) {
        return ExecutionMode::Sequential;
    }

    // Use parallel if available
    #[cfg(feature = "parallel")]
    return ExecutionMode::Parallel;

    #[cfg(not(feature = "parallel"))]
    ExecutionMode::Sequential
}
