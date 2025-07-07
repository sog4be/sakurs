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
            CliError::FileNotFound(path) => write!(f, "File not found: {path}"),
            CliError::InvalidPattern(pattern) => write!(f, "Invalid file pattern: {pattern}"),
            CliError::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            CliError::ProcessingError(msg) => write!(f, "Processing error: {msg}"),
        }
    }
}

impl std::error::Error for CliError {}

/// Result type alias for CLI operations
pub type CliResult<T> = Result<T, anyhow::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found_error_display() {
        let error = CliError::FileNotFound("test.txt".to_string());
        assert_eq!(error.to_string(), "File not found: test.txt");
    }

    #[test]
    fn test_invalid_pattern_error_display() {
        let error = CliError::InvalidPattern("[invalid".to_string());
        assert_eq!(error.to_string(), "Invalid file pattern: [invalid");
    }

    #[test]
    fn test_config_error_display() {
        let error = CliError::ConfigError("invalid format".to_string());
        assert_eq!(error.to_string(), "Configuration error: invalid format");
    }

    #[test]
    fn test_processing_error_display() {
        let error = CliError::ProcessingError("parse failed".to_string());
        assert_eq!(error.to_string(), "Processing error: parse failed");
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = CliError::FileNotFound("test.txt".to_string());
        // Test that it implements std::error::Error
        let _: &dyn std::error::Error = &error;

        // Test Debug formatting
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("FileNotFound"));
        assert!(debug_str.contains("test.txt"));
    }

    #[test]
    fn test_cli_result_type_alias() {
        // Test successful result
        let success: CliResult<String> = Ok("test".to_string());
        assert!(success.is_ok());
        assert_eq!(success.as_ref().unwrap(), "test");

        // Test error result
        let failure: CliResult<String> = Err(anyhow::anyhow!("test error"));
        assert!(failure.is_err());
        assert!(failure
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("test error"));
    }

    #[test]
    fn test_all_error_variants_creation() {
        // Test that all enum variants can be created
        let file_error = CliError::FileNotFound("/path/to/file.txt".to_string());
        let pattern_error = CliError::InvalidPattern("*.{".to_string());
        let config_error = CliError::ConfigError("missing field 'language'".to_string());
        let processing_error = CliError::ProcessingError("segmentation fault".to_string());

        // Verify they all implement Display properly
        assert!(file_error.to_string().starts_with("File not found:"));
        assert!(pattern_error
            .to_string()
            .starts_with("Invalid file pattern:"));
        assert!(config_error.to_string().starts_with("Configuration error:"));
        assert!(processing_error
            .to_string()
            .starts_with("Processing error:"));
    }

    #[test]
    fn test_error_with_special_characters() {
        // Test with special characters and Unicode
        let error = CliError::FileNotFound("ファイル/test 文件.txt".to_string());
        assert_eq!(error.to_string(), "File not found: ファイル/test 文件.txt");

        let pattern_error = CliError::InvalidPattern("**[!@#$%^&*()".to_string());
        assert_eq!(
            pattern_error.to_string(),
            "Invalid file pattern: **[!@#$%^&*()"
        );
    }
}
