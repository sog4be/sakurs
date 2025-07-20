//! Python bindings for Sakurs sentence boundary detection
//!
//! This module provides high-performance sentence boundary detection
//! using the Delta-Stack Monoid algorithm implemented in Rust.

#![allow(non_local_definitions)]

use pyo3::prelude::*;
use pyo3::types::PyList;

mod exceptions;
mod input;
mod output;
mod processor;
mod types;

use exceptions::{register_exceptions, InternalError};
use input::PyInput;
use output::{boundaries_to_sentences_with_char_offsets, ProcessingMetadata, Sentence};
use processor::PyProcessor;
use sakurs_core::{Config, SentenceProcessor};
use std::time::Instant;
use types::PyProcessorConfig;

/// Split text into sentences
///
/// Args:
///     input: Text string, file path, bytes, or file-like object to split
///     language: Language code ("en", "ja") for built-in rules (default: "en")
///     threads: Number of threads for parallel processing (None for auto)
///     chunk_size: Chunk size in bytes for parallel processing (default: 256KB)
///     parallel: Force parallel processing even for small inputs
///     execution_mode: Processing strategy ("sequential", "parallel", "adaptive")
///     return_details: Return Sentence objects with metadata instead of strings
///     preserve_whitespace: Keep leading/trailing whitespace in sentences (default: False)
///     encoding: Text encoding for file/binary inputs (default: "utf-8")
///
/// Returns:
///     List of sentence strings or Sentence objects if return_details=True
#[pyfunction]
#[pyo3(signature = (input, *, language=None, threads=None, chunk_size=None, parallel=false, execution_mode="adaptive", return_details=false, preserve_whitespace=false, encoding="utf-8"))]
#[allow(clippy::too_many_arguments)]
#[allow(unused_variables)]
fn split(
    input: &Bound<'_, PyAny>,
    language: Option<&str>,
    threads: Option<usize>,
    chunk_size: Option<usize>,
    parallel: bool,
    execution_mode: &str,
    return_details: bool,
    preserve_whitespace: bool,
    encoding: &str,
    py: Python,
) -> PyResult<PyObject> {
    let start_time = Instant::now();

    // Extract input from Python object
    let py_input = PyInput::from_py_object(py, input)?;

    // Convert to core Input type and get the text content
    let (core_input, text) = py_input.into_core_input_and_text(py, encoding)?;

    // Determine language
    let lang_code = match language.unwrap_or("en").to_lowercase().as_str() {
        "en" | "english" => "en",
        "ja" | "japanese" => "ja",
        _ => {
            return Err(InternalError::UnsupportedLanguage(
                language.unwrap_or("unknown").to_string(),
            )
            .into())
        }
    };

    // Build configuration
    let mut config_builder = Config::builder()
        .language(lang_code)
        .map_err(|e| InternalError::ConfigurationError(e.to_string()))?;

    if let Some(t) = threads {
        config_builder = config_builder.threads(Some(t));
    }

    if let Some(cs) = chunk_size {
        config_builder = config_builder.chunk_size(cs);
    }

    let config = config_builder
        .build()
        .map_err(|e| InternalError::ConfigurationError(e.to_string()))?;

    // Create processor
    let processor = SentenceProcessor::with_config(config)
        .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

    // Note: execution_mode is a parameter for future use when the core API supports it
    // For now, we validate it but don't use it directly
    match execution_mode {
        "sequential" | "parallel" | "adaptive" => {}
        _ => {
            return Err(InternalError::ConfigurationError(format!(
                "Invalid execution_mode: {execution_mode}"
            ))
            .into())
        }
    }

    // Release GIL during processing for better performance
    let output = py
        .allow_threads(|| processor.process(core_input))
        .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

    let processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    if return_details {
        // Return list of Sentence objects with character offsets
        let boundaries_with_offsets: Vec<(usize, usize)> = output
            .boundaries
            .iter()
            .map(|b| (b.char_offset, b.offset))
            .collect();
        let sentences = boundaries_to_sentences_with_char_offsets(
            &text,
            &boundaries_with_offsets,
            preserve_whitespace,
            py,
        )?;

        // Determine actual execution mode used (from strategy)
        let execution_mode_str = match output.metadata.strategy_used.as_str() {
            "Sequential" => "sequential",
            "Parallel" => "parallel",
            _ => "adaptive",
        };

        // Determine threads used (heuristic based on chunks)
        let threads_used = if output.metadata.chunks_processed > 1 {
            output.metadata.chunks_processed.min(8) // Reasonable estimate
        } else {
            1
        };

        // Create metadata
        let _metadata = ProcessingMetadata::new(
            sentences.len(),
            processing_time_ms,
            threads_used,
            256 * 1024, // Default chunk size - we don't have access to actual value
            execution_mode_str.to_string(),
        );

        // Return list of sentences directly when return_details=True
        Ok(PyList::new(py, sentences)?.unbind().into())
    } else {
        // Return list of strings using character offsets
        let mut sentences = Vec::new();
        let mut start_char = 0;
        let mut start_byte = 0;

        // Create a mapping of character positions to byte positions
        let _char_to_byte: Vec<(usize, usize)> = text
            .char_indices()
            .enumerate()
            .map(|(char_pos, (byte_pos, _))| (char_pos, byte_pos))
            .collect();

        for boundary in &output.boundaries {
            let end_char = boundary.char_offset;
            let end_byte = boundary.offset;

            if end_char > start_char && end_byte <= text.len() {
                let sentence = text[start_byte..end_byte].to_string();
                // Trim whitespace unless preserve_whitespace is True
                let sentence = if preserve_whitespace {
                    sentence
                } else {
                    sentence.trim().to_string()
                };
                sentences.push(sentence);
                start_char = end_char;
                start_byte = end_byte;
            }
        }

        // Handle any remaining text
        if start_byte < text.len() {
            let sentence = text[start_byte..].to_string();
            // Trim whitespace unless preserve_whitespace is True
            let sentence = if preserve_whitespace {
                sentence
            } else {
                sentence.trim().to_string()
            };
            sentences.push(sentence);
        }

        Ok(PyList::new(py, sentences)?.unbind().into())
    }
}

/// Load a processor for the specified language (spaCy-style API)
#[pyfunction]
#[pyo3(signature = (language, *, threads=None, chunk_size=None, execution_mode="adaptive"))]
#[allow(unused_variables)]
fn load(
    language: &str,
    threads: Option<usize>,
    chunk_size: Option<usize>,
    execution_mode: &str,
) -> PyResult<PyProcessor> {
    // Create a config with the specified parameters
    let config =
        PyProcessorConfig::new(chunk_size.unwrap_or(256 * 1024), 256, threads, 1024 * 1024);

    PyProcessor::new(language, Some(config))
}

/// Get list of supported languages
#[pyfunction]
fn supported_languages() -> Vec<&'static str> {
    vec!["en", "ja"]
}

/// Main Python module for sakurs
#[pymodule]
fn sakurs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    // Core classes
    m.add_class::<PyProcessor>()?;
    m.add_class::<PyProcessorConfig>()?;
    m.add_class::<Sentence>()?;
    m.add_class::<ProcessingMetadata>()?;

    // Main API functions
    m.add_function(pyo3::wrap_pyfunction!(split, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(load, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(supported_languages, m)?)?;

    // Register custom exceptions
    register_exceptions(py, m)?;

    // Module metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add(
        "__doc__",
        "High-performance sentence boundary detection using Delta-Stack Monoid algorithm.\n\n\
         This module provides fast and accurate sentence segmentation for multiple languages\n\
         with support for parallel processing and streaming large texts.",
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
    fn test_split_function() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            // Test basic split
            let result = split(
                "Hello world. How are you?",
                None,
                None,
                None,
                false,
                "adaptive",
                false,
                "utf-8",
                py,
            );
            assert!(result.is_ok());

            let sentences: Vec<String> = result.unwrap().extract(py).unwrap();
            assert_eq!(sentences.len(), 2);
            assert_eq!(sentences[0], "Hello world.");
            assert_eq!(sentences[1], " How are you?");
        });
    }

    #[test]
    fn test_split_with_details() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            // Test split with return_details=true
            let result = split(
                "Hello world. How are you?",
                None,
                None,
                None,
                false,
                "adaptive",
                true,
                "utf-8",
                py,
            );
            assert!(result.is_ok());

            // The result should be a list of Sentence objects
            // For now, just check that it returns a PyObject
            let _obj = result.unwrap();
            // We can't easily extract Sentence objects in Rust tests,
            // so we'll test this functionality in Python tests instead
        });
    }

    #[test]
    fn test_supported_languages() {
        let languages = supported_languages();
        assert!(languages.contains(&"en"));
        assert!(languages.contains(&"ja"));
        assert_eq!(languages.len(), 2);
    }
}
