//! Exception hierarchy for Python bindings

use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyIOError, PyTypeError};
use pyo3::prelude::*;
use std::io;
use thiserror::Error;

// Create Python exception classes
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
create_exception!(
    sakurs,
    FileNotFoundError,
    SakursError,
    "Raised when input file is not found."
);

/// Internal error enum for Rust-side error handling
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
    IoError(#[from] io::Error),
}

impl From<InternalError> for PyErr {
    fn from(err: InternalError) -> PyErr {
        match err {
            InternalError::UnsupportedLanguage(msg) => InvalidLanguageError::new_err(msg),
            InternalError::ProcessingError(msg) => ProcessingError::new_err(msg),
            InternalError::ConfigurationError(msg) => ConfigurationError::new_err(msg),
            InternalError::InvalidInput(msg) => PyTypeError::new_err(msg),
            InternalError::FileNotFound(msg) => FileNotFoundError::new_err(msg),
            InternalError::IoError(err) => PyIOError::new_err(err.to_string()),
        }
    }
}

impl From<sakurs_core::application::ProcessingError> for InternalError {
    fn from(err: sakurs_core::application::ProcessingError) -> Self {
        InternalError::ProcessingError(err.to_string())
    }
}

impl From<sakurs_core::api::Error> for InternalError {
    fn from(err: sakurs_core::api::Error) -> Self {
        use sakurs_core::api::Error;
        match err {
            Error::InvalidLanguage(language) => InternalError::UnsupportedLanguage(format!(
                "Language '{language}' is not supported"
            )),
            Error::Configuration(message) => InternalError::ConfigurationError(message),
            Error::Processing(source) => InternalError::ProcessingError(source.to_string()),
            Error::Infrastructure(message) => {
                InternalError::ProcessingError(format!("Infrastructure error: {message}"))
            }
            Error::InvalidInput(message) => InternalError::InvalidInput(message),
            Error::Unsupported(message) => {
                InternalError::ConfigurationError(format!("Unsupported: {message}"))
            }
        }
    }
}

/// Register all exception types with the Python module
pub fn register_exceptions(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("SakursError", py.get_type::<SakursError>())?;
    m.add(
        "InvalidLanguageError",
        py.get_type::<InvalidLanguageError>(),
    )?;
    m.add("ProcessingError", py.get_type::<ProcessingError>())?;
    m.add("ConfigurationError", py.get_type::<ConfigurationError>())?;
    m.add("FileNotFoundError", py.get_type::<FileNotFoundError>())?;
    Ok(())
}
