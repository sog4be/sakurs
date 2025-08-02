//! Layered error types
//!
//! Implements the layered error architecture specified in DESIGN.md Section 8.1.

use sakurs_core::CoreError;
use thiserror::Error;

/// Engine-level errors (Application Layer)
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

    /// Chunking failed
    #[error("chunking failed: {reason}")]
    ChunkingFailed {
        /// The reason why chunking failed
        reason: String,
    },

    /// Thread pool exhausted
    #[error("thread pool exhausted")]
    ThreadPoolExhausted,

    /// Parallel execution error
    #[cfg(feature = "parallel")]
    #[error("parallel execution failed: {0}")]
    ParallelError(String),

    /// Configuration error
    #[error("invalid configuration: {0}")]
    ConfigError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Encoding error (UTF-8, etc.)
    #[error("encoding error: {0}")]
    EncodingError(String),
}

/// API-level errors (Public Interface Layer)
#[derive(Error, Debug)]
pub enum ApiError {
    /// Unsupported language
    #[error("language '{code}' not supported")]
    UnsupportedLanguage {
        /// The language code that is not supported
        code: String,
    },

    /// Invalid input
    #[error("invalid input: {reason}")]
    InvalidInput {
        /// The reason why the input is invalid
        reason: String,
    },

    /// Configuration error with path information
    #[error("configuration error in {path}: {error}")]
    ConfigurationError {
        /// The configuration file path
        path: String,
        /// The specific error that occurred
        error: String,
    },

    /// Invalid UTF-8 at specific position
    #[error("invalid UTF-8 at position {position}")]
    InvalidUtf8 {
        /// The byte position where the invalid UTF-8 was found
        position: usize,
    },

    /// Engine layer error
    #[error("engine error: {0}")]
    Engine(#[from] EngineError),
}

impl From<std::io::Error> for EngineError {
    fn from(err: std::io::Error) -> Self {
        EngineError::IoError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for EngineError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        EngineError::EncodingError(err.to_string())
    }
}

/// Result type for engine operations
pub type Result<T> = std::result::Result<T, EngineError>;

/// Result type for API operations
pub type ApiResult<T> = std::result::Result<T, ApiError>;
