//! API error types

use std::string::FromUtf8Error;
use thiserror::Error;

/// API-level errors
#[derive(Error, Debug)]
pub enum ApiError {
    /// Engine error
    #[error("engine error: {0}")]
    Engine(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// UTF-8 conversion error
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),

    /// Configuration error
    #[error("configuration error: {0}")]
    Config(String),

    /// Serialization error
    #[cfg(feature = "serde")]
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Result type for API operations
pub type Result<T> = std::result::Result<T, ApiError>;
