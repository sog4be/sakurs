//! Engine error types

use sakurs_core::CoreError;
use thiserror::Error;

/// Engine-level errors
#[derive(Error, Debug)]
pub enum EngineError {
    /// Core algorithm error
    #[error("core algorithm error: {0}")]
    Core(#[from] CoreError),

    /// Invalid chunk boundaries
    #[error("invalid chunk boundary at position {position}")]
    InvalidChunkBoundary {
        /// The byte position where the invalid boundary was detected
        position: usize,
    },

    /// Parallel execution error
    #[cfg(feature = "parallel")]
    #[error("parallel execution failed: {0}")]
    ParallelError(String),

    /// Configuration error
    #[error("invalid configuration: {0}")]
    ConfigError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for engine operations
pub type Result<T> = std::result::Result<T, EngineError>;
