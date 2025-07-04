//! Error types for benchmarking operations

use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type for benchmark operations
pub type BenchmarkResult<T> = Result<T, BenchmarkError>;

/// Error types that can occur during benchmarking
#[derive(Debug)]
pub enum BenchmarkError {
    /// IO error (file not found, permission denied, etc.)
    Io { path: PathBuf, source: io::Error },

    /// JSON parsing error
    JsonParse {
        path: PathBuf,
        source: serde_json::Error,
    },

    /// Data validation error
    Validation { message: String },

    /// Corpus not found or not downloaded
    CorpusNotFound {
        corpus_name: String,
        expected_path: PathBuf,
    },

    /// Invalid data format
    InvalidFormat { expected: String, found: String },

    /// Configuration error
    Config { message: String },
}

impl fmt::Display for BenchmarkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "IO error for path '{}': {}", path.display(), source)
            }
            Self::JsonParse { path, source } => {
                write!(
                    f,
                    "Failed to parse JSON from '{}': {}",
                    path.display(),
                    source
                )
            }
            Self::Validation { message } => {
                write!(f, "Validation error: {}", message)
            }
            Self::CorpusNotFound {
                corpus_name,
                expected_path,
            } => {
                write!(
                    f,
                    "Corpus '{}' not found at '{}'. Please run download script first.",
                    corpus_name,
                    expected_path.display()
                )
            }
            Self::InvalidFormat { expected, found } => {
                write!(f, "Invalid format: expected {}, found {}", expected, found)
            }
            Self::Config { message } => {
                write!(f, "Configuration error: {}", message)
            }
        }
    }
}

impl StdError for BenchmarkError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::JsonParse { source, .. } => Some(source),
            _ => None,
        }
    }
}

// Conversion implementations
impl From<io::Error> for BenchmarkError {
    fn from(err: io::Error) -> Self {
        Self::Io {
            path: PathBuf::from("<unknown>"),
            source: err,
        }
    }
}

impl From<serde_json::Error> for BenchmarkError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParse {
            path: PathBuf::from("<unknown>"),
            source: err,
        }
    }
}
