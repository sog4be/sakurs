//! Exception handling for Python bindings

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyIOError, PyTypeError};
use pyo3::prelude::*;
use pyo3::{create_exception, PyErr};
use std::io;
use thiserror::Error;

// Create custom Python exception types
create_exception!(
    sakurs,
    SakursError,
    PyException,
    "Base exception for all sakurs errors."
);
create_exception!(
    sakurs,
    InvalidLanguageError,
    SakursError,
    "Raised when language code is not recognized."
);
create_exception!(
    sakurs,
    ProcessingError,
    SakursError,
    "Raised when text processing fails."
);
create_exception!(
    sakurs,
    ConfigurationError,
    SakursError,
    "Raised when configuration is invalid."
);

/// Internal error type for Rust-side error handling
#[derive(Error, Debug)]
pub enum InternalError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),
}

impl From<InternalError> for PyErr {
    fn from(err: InternalError) -> PyErr {
        match err {
            InternalError::UnsupportedLanguage(msg) => InvalidLanguageError::new_err(msg),
            InternalError::ProcessingError(msg) => ProcessingError::new_err(msg),
            InternalError::ConfigurationError(msg) => ConfigurationError::new_err(msg),
            InternalError::InvalidInput(msg) => PyTypeError::new_err(msg),
            InternalError::FileNotFound(msg) => PyFileNotFoundError::new_err(msg),
            InternalError::IoError(msg) => PyIOError::new_err(msg),
            InternalError::EncodingError(msg) => PyIOError::new_err(msg),
        }
    }
}

// Note: We're removing the automatic conversion from ProcessingError
// since it's an internal type now. Errors will be converted to strings
// at the API boundary instead.

impl From<io::Error> for InternalError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => InternalError::FileNotFound(err.to_string()),
            _ => InternalError::IoError(err.to_string()),
        }
    }
}

/// Register all custom exceptions with the Python module
pub fn register_exceptions(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("SakursError", py.get_type::<SakursError>())?;
    m.add(
        "InvalidLanguageError",
        py.get_type::<InvalidLanguageError>(),
    )?;
    m.add("ProcessingError", py.get_type::<ProcessingError>())?;
    m.add("ConfigurationError", py.get_type::<ConfigurationError>())?;
    Ok(())
}
