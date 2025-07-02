//! Error handling for the CLI application

use std::fmt;

/// Custom error type for CLI-specific errors
#[derive(Debug)]
pub enum CliError {
    /// File not found or inaccessible
    FileNotFound(String),
    /// Invalid file pattern
    InvalidPattern(String),
    /// Configuration error
    ConfigError(String),
    /// Processing error from core
    ProcessingError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::FileNotFound(path) => write!(f, "File not found: {}", path),
            CliError::InvalidPattern(pattern) => write!(f, "Invalid file pattern: {}", pattern),
            CliError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            CliError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

/// Result type alias for CLI operations
pub type CliResult<T> = Result<T, anyhow::Error>;
