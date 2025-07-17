use thiserror::Error;

/// Domain-specific errors
#[derive(Debug, Error)]
pub enum DomainError {
    /// Configuration loading or parsing error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Unsupported language requested
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Invalid language rules configuration
    #[error("Invalid language rules: {0}")]
    InvalidLanguageRules(String),
}
