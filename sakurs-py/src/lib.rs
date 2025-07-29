//! Python bindings for Sakurs sentence boundary detection
//!
//! This module provides high-performance sentence boundary detection
//! using the Delta-Stack Monoid algorithm implemented in Rust.

#![allow(non_local_definitions)]

use pyo3::prelude::*;
use pyo3::types::PyList;

mod exceptions;
mod input;
mod iterator;
mod language_config;
mod output;
mod processor;
mod stream;
mod types;

use exceptions::{register_exceptions, InternalError};
use input::PyInput;
use language_config::LanguageConfig;
use output::{boundaries_to_sentences_with_char_offsets, ProcessingMetadata, Sentence};
use processor::PyProcessor;
use sakurs_api::Config;
use std::time::Instant;

/// Split text into sentences
///
/// Args:
///     input: Text string, file path, bytes, or file-like object to split
///     language: Language code ("en", "ja") for built-in rules (default: "en")
///     language_config: Custom language configuration
///     threads: Number of threads for parallel processing (None for auto)
///     chunk_kb: Chunk size in KB for parallel processing (default: 256)
///     parallel: Force parallel processing even for small inputs
///     execution_mode: Processing strategy ("sequential", "parallel", "adaptive")
///     return_details: Return Sentence objects with metadata instead of strings
///     preserve_whitespace: Keep leading/trailing whitespace in sentences (default: False)
///     encoding: Text encoding for file/binary inputs (default: "utf-8")
///
/// Returns:
///     List of sentence strings or Sentence objects if return_details=True
#[pyfunction]
#[pyo3(signature = (input, *, language=None, language_config=None, threads=None, chunk_kb=None, parallel=false, execution_mode="adaptive", return_details=false, preserve_whitespace=false, encoding="utf-8"))]
#[allow(clippy::too_many_arguments)]
#[allow(unused_variables)]
fn split(
    input: &Bound<'_, PyAny>,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    threads: Option<usize>,
    chunk_kb: Option<usize>,
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

    // Build configuration
    let mut config_builder = if let Some(_lang_config) = language_config {
        // TODO: Custom language configuration not yet supported in new architecture
        return Err(InternalError::ConfigurationError(
            "Custom language configuration is not yet supported in this version".to_string(),
        )
        .into());
    } else {
        // Use built-in language
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
        Config::builder()
            .language(lang_code)
            .map_err(|e| InternalError::ConfigurationError(e.to_string()))?
    };

    // Handle execution mode and performance parameters
    match execution_mode {
        "sequential" => {
            // Force sequential mode by setting threads to 1
            config_builder = config_builder.threads(Some(1));
        }
        "parallel" => {
            // Use provided threads or let it default to all available
            config_builder = config_builder.threads(threads);
            // If parallel flag is set, ensure we use lower threshold
            if parallel {
                // Use defaults for parallel mode
            }
        }
        "adaptive" => {
            // Let the system decide based on text size
            if let Some(t) = threads {
                config_builder = config_builder.threads(Some(t));
            }
        }
        _ => {
            return Err(InternalError::ConfigurationError(format!(
                "Invalid execution_mode: {execution_mode}"
            ))
            .into())
        }
    }

    if let Some(kb) = chunk_kb {
        config_builder = config_builder.chunk_size(kb * 1024);
    }

    // Create processor
    let processor = config_builder
        .build_processor()
        .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

    // execution_mode is now properly handled in the configuration above

    // Release GIL during processing for better performance
    let output = py
        .allow_threads(|| processor.process(&text))
        .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

    let processing_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    if return_details {
        // Return list of Sentence objects with character offsets
        let boundaries_with_offsets: Vec<(usize, usize)> = output
            .iter()
            .map(|b| (b.char_offset, b.byte_offset))
            .collect();
        let sentences = boundaries_to_sentences_with_char_offsets(
            &text,
            &boundaries_with_offsets,
            preserve_whitespace,
            py,
        )?;

        // Determine actual execution mode used (simplified for now)
        let execution_mode_str = "adaptive";

        // Thread count is not available in the simplified API
        let threads_used = 1;

        // Create metadata
        let _metadata = ProcessingMetadata::new(
            sentences.len(),
            processing_time_ms,
            threads_used,
            256, // Default chunk size in KB - we don't have access to actual value
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

        for boundary in &output {
            let end_char = boundary.char_offset;
            let end_byte = boundary.byte_offset;

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

/// Load a sentence splitter for the specified language (spaCy-style API)
#[pyfunction]
#[pyo3(signature = (language, *, threads=None, chunk_kb=None, execution_mode="adaptive"))]
fn load(
    language: &str,
    threads: Option<usize>,
    chunk_kb: Option<usize>,
    execution_mode: &str,
    py: Python,
) -> PyResult<PyProcessor> {
    // Create processor with the specified parameters
    PyProcessor::new(
        Some(language),
        None, // language_config
        threads,
        chunk_kb,
        execution_mode,
        false, // streaming
        10,    // stream_chunk_mb (not used when streaming=false)
        py,
    )
}

/// Process input and return sentences as an iterator
///
/// This function loads the entire input into memory but returns results
/// incrementally for responsive processing. For true memory-efficient
/// streaming of large files, use split_large_file().
///
/// Args:
///     input: Text string, file path, bytes, or file-like object
///     language: Language code ("en", "ja") for built-in rules (default: "en")
///     language_config: Custom language configuration
///     threads: Number of threads for parallel processing (None for auto)
///     chunk_kb: Chunk size in KB for parallel processing (default: 256)
///     encoding: Text encoding for file/binary inputs (default: "utf-8")
///
/// Returns:
///     Iterator that yields sentences one at a time
#[pyfunction]
#[pyo3(signature = (input, *, language=None, language_config=None, threads=None, chunk_kb=None, encoding="utf-8"))]
#[allow(clippy::too_many_arguments)]
fn iter_split(
    input: &Bound<'_, PyAny>,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    threads: Option<usize>,
    chunk_kb: Option<usize>,
    encoding: &str,
    py: Python,
) -> PyResult<iterator::SentenceIterator> {
    stream::create_iter_split_iterator(
        py,
        input,
        language,
        language_config,
        threads,
        chunk_kb.map(|kb| kb * 1024),
        encoding,
    )
}

/// Process large files with limited memory usage
///
/// This function reads and processes the file in chunks, ensuring memory
/// usage stays within the specified limit. Sentences that span chunk
/// boundaries are handled correctly but may be delayed until the next
/// chunk is processed.
///
/// Args:
///     file_path: Path to the file to process
///     language: Language code ("en", "ja") for built-in rules (default: "en")
///     language_config: Custom language configuration
///     max_memory_mb: Maximum memory to use in MB (default: 100)
///     overlap_size: Bytes to overlap between chunks for boundary handling (default: 1024)
///     encoding: File encoding (default: "utf-8")
///
/// Returns:
///     Iterator yielding sentences as they are found
///
/// Note:
///     Due to the nature of chunk processing, sentences near chunk
///     boundaries may be yielded slightly out of order compared to
///     their position in the file.
#[pyfunction]
#[pyo3(signature = (file_path, *, language=None, language_config=None, max_memory_mb=100, overlap_size=1024, encoding="utf-8"))]
#[allow(clippy::too_many_arguments)]
fn split_large_file(
    file_path: &str,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    max_memory_mb: usize,
    overlap_size: usize,
    encoding: &str,
    py: Python,
) -> PyResult<stream::LargeFileIterator> {
    stream::create_large_file_iterator(
        py,
        file_path,
        language,
        language_config,
        max_memory_mb,
        overlap_size,
        encoding,
    )
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
    m.add_class::<Sentence>()?;
    m.add_class::<ProcessingMetadata>()?;
    m.add_class::<iterator::SentenceIterator>()?;
    m.add_class::<stream::LargeFileIterator>()?;

    // Language configuration classes
    m.add_class::<LanguageConfig>()?;
    m.add_class::<language_config::MetadataConfig>()?;
    m.add_class::<language_config::TerminatorConfig>()?;
    m.add_class::<language_config::TerminatorPattern>()?;
    m.add_class::<language_config::EllipsisConfig>()?;
    m.add_class::<language_config::ContextRule>()?;
    m.add_class::<language_config::ExceptionPattern>()?;
    m.add_class::<language_config::EnclosureConfig>()?;
    m.add_class::<language_config::EnclosurePair>()?;
    m.add_class::<language_config::SuppressionConfig>()?;
    m.add_class::<language_config::FastPattern>()?;
    m.add_class::<language_config::RegexPattern>()?;
    m.add_class::<language_config::AbbreviationConfig>()?;
    m.add_class::<language_config::SentenceStarterConfig>()?;

    // Main API functions
    m.add_function(pyo3::wrap_pyfunction!(split, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(load, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(iter_split, m)?)?;
    m.add_function(pyo3::wrap_pyfunction!(split_large_file, m)?)?;
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
            // Create a Python string as input
            let input_str = pyo3::types::PyString::new(py, "Hello world. How are you?");

            // Test basic split
            let result = split(
                input_str.as_ref(),
                None,  // language
                None,  // language_config
                None,  // threads
                None,  // chunk_size
                false, // parallel
                "adaptive",
                false, // return_details
                false, // preserve_whitespace
                "utf-8",
                py,
            );
            assert!(result.is_ok());

            let sentences: Vec<String> = result.unwrap().extract(py).unwrap();
            assert_eq!(sentences.len(), 2);
            assert_eq!(sentences[0], "Hello world.");
            assert_eq!(sentences[1], "How are you?");
        });
    }

    #[test]
    fn test_split_with_details() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            // Create a Python string as input
            let input_str = pyo3::types::PyString::new(py, "Hello world. How are you?");

            // Test split with return_details=true
            let result = split(
                input_str.as_ref(),
                None,  // language
                None,  // language_config
                None,  // threads
                None,  // chunk_size
                false, // parallel
                "adaptive",
                true,  // return_details
                false, // preserve_whitespace
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
