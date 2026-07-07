//! Configuration and error handling for the application layer
//!
//! This module provides configuration options for performance tuning
//! and comprehensive error types for robust error handling.

use thiserror::Error;

/// Configuration options for text processing
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Target size for each chunk in bytes
    pub chunk_size: usize,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            chunk_size: 256 * 1024, // 256KB chunks
        }
    }
}

/// Errors that can occur during text processing
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// Text exceeds maximum size limit
    #[error("Text too large for processing: {size} bytes (max: {max} bytes)")]
    TextTooLarge { size: usize, max: usize },

    /// Invalid configuration parameters
    #[error("Invalid configuration: {reason}")]
    InvalidConfig { reason: String },

    /// Error during parallel processing
    #[error("Parallel processing failed")]
    ParallelError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// UTF-8 encoding error
    #[error("Invalid UTF-8 in text at position {position}")]
    Utf8Error { position: usize },

    /// Chunk boundary calculation error
    #[error("Failed to calculate chunk boundaries: {reason}")]
    ChunkingError { reason: String },

    /// UTF-8 boundary detection failed
    #[error("Failed to find UTF-8 boundary at position {position}")]
    Utf8BoundaryError { position: usize },

    /// Word boundary detection failed
    #[error("Failed to find word boundary near position {position}")]
    WordBoundaryError { position: usize },

    /// Invalid chunk configuration
    #[error("Invalid chunk boundaries: start={start}, end={end}, next={next}")]
    InvalidChunkBoundaries {
        start: usize,
        end: usize,
        next: usize,
    },

    /// Memory allocation failure
    #[error("Memory allocation failed: {reason}")]
    AllocationError { reason: String },

    /// I/O error (for future file operations)
    #[error("I/O operation failed")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    /// Language rules error
    #[error("Language rules processing failed: {reason}")]
    LanguageRulesError { reason: String },

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for processing operations
pub type ProcessingResult<T> = Result<T, ProcessingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProcessorConfig::default();
        assert_eq!(config.chunk_size, 256 * 1024);
    }

    #[test]
    fn test_error_display() {
        let error = ProcessingError::InvalidConfig {
            reason: "test reason".to_string(),
        };
        assert!(error.to_string().contains("test reason"));
    }
}
