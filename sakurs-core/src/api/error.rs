//! Error types for the API

use thiserror::Error;

/// Error type for API operations
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid language specification
    #[error("Invalid language: {0}")]
    InvalidLanguage(String),

    /// Processing error from the application layer
    #[error("Processing error: {0}")]
    Processing(#[from] crate::application::config::ProcessingError),

    /// Infrastructure error (I/O, etc.)
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Unsupported feature
    #[error("Feature not supported: {0}")]
    Unsupported(String),
}

/// Result type for API operations
pub type Result<T> = std::result::Result<T, Error>;
