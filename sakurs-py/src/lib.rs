//! Python bindings for Sakurs sentence boundary detection
//!
//! This module provides high-performance sentence boundary detection
//! using the Delta-Stack Monoid algorithm implemented in Rust.

#![allow(non_local_definitions)]

use pyo3::prelude::*;

mod exceptions;
mod output;
mod processor;
mod types;

use crate::exceptions::register_exceptions;
use crate::output::{create_sentence_list, ProcessingMetadata, Sentence};
use processor::PyProcessor;
use types::PyProcessorConfig;

/// Split text into sentences
#[pyfunction]
#[pyo3(signature = (text, *, language="en", config=None, threads=None, return_details=false))]
fn split(
    text: &str,
    language: &str,
    config: Option<PyProcessorConfig>,
    threads: Option<usize>,
    return_details: bool,
    py: Python,
) -> PyResult<PyObject> {
    let processor = PyProcessor::new(language, config)?;

    if return_details {
        // Return List[Sentence] with detailed information
        let result = processor.process_with_details(text, threads, py)?;
        let sentences = create_sentence_list(py, text, &result.boundaries)?;

        // Convert Vec<Sentence> to Python list
        let py_list = pyo3::types::PyList::new(py, sentences)?;
        Ok(py_list.into())
    } else {
        // Return List[str] for backward compatibility
        let sentences = processor.split(text, threads, py)?;
        let py_list = pyo3::types::PyList::new(py, sentences)?;
        Ok(py_list.into())
    }
}

/// Load a processor for the specified language (spaCy-style API)
#[pyfunction]
#[pyo3(signature = (language, config=None))]
fn load(language: &str, config: Option<PyProcessorConfig>) -> PyResult<PyProcessor> {
    PyProcessor::new(language, config)
}

/// Get list of supported languages
#[pyfunction]
fn supported_languages() -> Vec<&'static str> {
    vec!["en", "english", "ja", "japanese"]
}

/// Main Python module for sakurs
#[pymodule]
fn sakurs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    // Core classes
    m.add_class::<PyProcessor>()?;
    m.add_class::<PyProcessorConfig>()?;

    // Output classes
    m.add_class::<Sentence>()?;
    m.add_class::<ProcessingMetadata>()?;

    // Main API functions
    m.add_function(wrap_pyfunction!(split, m)?)?;
    m.add_function(wrap_pyfunction!(load, m)?)?;
    m.add_function(wrap_pyfunction!(supported_languages, m)?)?;

    // Register exception classes
    register_exceptions(py, m)?;

    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add(
        "__doc__",
        "High-performance sentence boundary detection using Delta-Stack Monoid algorithm",
    )?;

    // Aliases for compatibility
    m.add("Processor", py.get_type::<PyProcessor>())?;
    m.add("ProcessorConfig", py.get_type::<PyProcessorConfig>())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_builds() {
        // Test that the module can be created
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let module = PyModule::new(py, "test_sakurs").unwrap();
            let result = sakurs(&module);
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_processor_creation() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|_py| {
            // Test English processor
            let en_processor = PyProcessor::new("en", None);
            assert!(en_processor.is_ok());

            // Test Japanese processor
            let ja_processor = PyProcessor::new("ja", None);
            assert!(ja_processor.is_ok());

            // Test unsupported language
            let unsupported = PyProcessor::new("unsupported", None);
            assert!(unsupported.is_err());
        });
    }

    #[test]
    fn test_config_creation() {
        let config = PyProcessorConfig::new(4096, 128, Some(2), 1048576);
        assert_eq!(config.chunk_size, 4096);
        assert_eq!(config.overlap_size, 128);
        assert_eq!(config.num_threads, Some(2));
        assert_eq!(config.max_chunk_size, 1048576);
    }

    #[test]
    fn test_supported_languages() {
        let languages = supported_languages();
        assert!(languages.contains(&"en"));
        assert!(languages.contains(&"ja"));
        assert_eq!(languages.len(), 4); // en, english, ja, japanese (short and long forms)
    }
}
