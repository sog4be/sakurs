//! Error handling for Python bindings

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use thiserror::Error;

/// Errors that can occur during Python processing
#[derive(Error, Debug)]
pub enum SakursError {
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<SakursError> for PyErr {
    fn from(err: SakursError) -> PyErr {
        PyRuntimeError::new_err(err.to_string())
    }
}

impl From<sakurs_core::application::ProcessingError> for SakursError {
    fn from(err: sakurs_core::application::ProcessingError) -> Self {
        SakursError::ProcessingError(err.to_string())
    }
}
