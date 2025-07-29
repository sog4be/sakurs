//! Execution strategies for text processing

use crate::error::Result;
use sakurs_delta_core::{Boundary, LanguageRules};

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

/// Automatically select execution mode based on text size
pub fn auto_select(text_len: usize, threshold: usize) -> ExecutionMode {
    if text_len < 1024 {
        // Very small texts: always sequential
        ExecutionMode::Sequential
    } else if text_len < threshold {
        // Medium texts: sequential is often faster
        ExecutionMode::Sequential
    } else {
        // Large texts: use parallel
        #[cfg(feature = "parallel")]
        return ExecutionMode::Parallel;

        #[cfg(not(feature = "parallel"))]
        ExecutionMode::Sequential
    }
}
