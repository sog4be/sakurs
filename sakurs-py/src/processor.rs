//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::exceptions::InternalError;
use crate::input::PyInput;
use crate::language_config::LanguageConfig;
use crate::types::PyProcessingResult;
use pyo3::prelude::*;
use sakurs_core::{Config, SentenceProcessor};

/// Main sentence splitter class for sentence boundary detection
#[pyclass(name = "SentenceSplitter")]
pub struct PyProcessor {
    processor: SentenceProcessor,
    language: String,
    chunk_size: usize,
    num_threads: Option<usize>,
    #[allow(dead_code)]
    custom_config: bool, // Track if using custom language config
}

#[pymethods]
impl PyProcessor {
    /// Create a new processor for the specified language
    #[new]
    #[pyo3(signature = (*, language=None, language_config=None, threads=None, chunk_kb=None, execution_mode="adaptive", streaming=false, stream_chunk_mb=10))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        language: Option<&str>,
        language_config: Option<LanguageConfig>,
        threads: Option<usize>,
        chunk_kb: Option<usize>,
        execution_mode: &str,
        streaming: bool,
        stream_chunk_mb: usize,
        py: Python,
    ) -> PyResult<Self> {
        // Convert KB/MB to bytes
        let chunk_size_bytes = if let Some(kb) = chunk_kb {
            kb * 1024
        } else if streaming {
            stream_chunk_mb * 1024 * 1024
        } else {
            256 * 1024 // Default 256KB (256 * 1024 bytes)
        };

        // Build Rust configuration and optionally custom language rules
        let (mut config_builder, language_display, is_custom, custom_rules) =
            if let Some(lang_config) = language_config {
                // Use custom language configuration
                let core_config = lang_config.to_core_config(py)?;
                let display_name = format!("custom({})", lang_config.metadata.code);

                // Create custom language rules from the config
                use sakurs_core::domain::language::ConfigurableLanguageRules;
                use std::sync::Arc;

                let language_rules = ConfigurableLanguageRules::from_config(&core_config)
                    .map_err(|e| InternalError::ConfigurationError(e.to_string()))?;
                let language_rules_arc: Arc<dyn sakurs_core::domain::language::LanguageRules> =
                    Arc::new(language_rules);

                (
                    Config::builder()
                        .language("en") // Default, will be overridden
                        .map_err(|e| InternalError::ConfigurationError(e.to_string()))?,
                    display_name,
                    true,
                    Some(language_rules_arc),
                )
            } else {
                // Use built-in language
                let lang = language.unwrap_or("en");
                let lang_code = match lang.to_lowercase().as_str() {
                    "en" | "english" => "en",
                    "ja" | "japanese" => "ja",
                    _ => return Err(InternalError::UnsupportedLanguage(lang.to_string()).into()),
                };
                (
                    Config::builder()
                        .language(lang_code)
                        .map_err(|e| InternalError::ProcessingError(e.to_string()))?,
                    lang.to_string(),
                    false,
                    None,
                )
            };

        // Handle execution mode
        match execution_mode {
            "sequential" => {
                config_builder = config_builder.threads(Some(1));
            }
            "parallel" => {
                config_builder = config_builder.threads(threads);
            }
            "adaptive" => {
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

        config_builder = config_builder.chunk_size(chunk_size_bytes);

        config_builder = config_builder
            .parallel_threshold(1024 * 1024) // 1MB
            .overlap_size(256);

        let rust_config = config_builder
            .build()
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        // Create processor with custom rules if provided
        let processor = if let Some(rules) = custom_rules {
            SentenceProcessor::with_custom_rules(rust_config, rules)
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?
        } else {
            SentenceProcessor::with_config(rust_config)
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?
        };

        Ok(Self {
            processor,
            language: language_display,
            chunk_size: chunk_size_bytes,
            num_threads: threads,
            custom_config: is_custom,
        })
    }

    /// Split text into sentences
    #[pyo3(signature = (input, *, return_details=false, encoding="utf-8"))]
    pub fn split(
        &self,
        input: &Bound<'_, PyAny>,
        return_details: bool,
        encoding: &str,
        py: Python,
    ) -> PyResult<PyObject> {
        use crate::output::boundaries_to_sentences_with_char_offsets;
        use pyo3::types::PyList;

        // Extract input from Python object
        let py_input = PyInput::from_py_object(py, input)?;

        // Convert to core Input type and get the text content
        let (core_input, text) = py_input.into_core_input_and_text(py, encoding)?;

        // Release GIL during processing for better performance
        let output = py
            .allow_threads(|| self.processor.process(core_input))
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        if return_details {
            // Return list of Sentence objects
            let boundaries_with_offsets: Vec<(usize, usize)> = output
                .boundaries
                .iter()
                .map(|b| (b.char_offset, b.offset))
                .collect();
            let sentences = boundaries_to_sentences_with_char_offsets(
                &text,
                &boundaries_with_offsets,
                false, // preserve_whitespace default to false
                py,
            )?;
            Ok(PyList::new(py, sentences)?.unbind().into())
        } else {
            // Convert boundaries to sentence list
            let boundaries: Vec<usize> = output.boundaries.iter().map(|b| b.offset).collect();
            let result = PyProcessingResult::new(boundaries, output.metadata.stats, text);
            Ok(PyList::new(py, result.sentences())?.unbind().into())
        }
    }

    /// Get supported language
    #[getter]
    fn language(&self) -> &str {
        &self.language
    }

    /// Check if the processor supports parallel processing
    #[getter]
    fn supports_parallel(&self) -> bool {
        true // Always true for Rust implementation
    }

    /// Iterate over sentences (memory-efficient)
    #[pyo3(signature = (input, *, encoding="utf-8", _preserve_whitespace=false))]
    pub fn iter_split(
        &self,
        input: &Bound<'_, PyAny>,
        encoding: &str,
        _preserve_whitespace: bool,
        py: Python,
    ) -> PyResult<crate::iterator::SentenceIterator> {
        use crate::stream::create_iter_split_iterator;

        // Extract language from processor
        let language = if self.custom_config {
            None // Custom config already embedded in processor
        } else {
            Some(self.language.as_str())
        };

        // Use the processor's configuration for streaming
        create_iter_split_iterator(
            py,
            input,
            language,
            None, // language_config already in processor
            self.num_threads,
            Some(self.chunk_size),
            encoding,
        )
    }

    /// Context manager entry
    fn __enter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Context manager exit
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        // Don't suppress any exceptions
        Ok(false)
    }

    fn __repr__(&self) -> String {
        let chunk_kb = self.chunk_size / 1024;
        format!(
            "SentenceSplitter(language='{}', threads={:?}, chunk_kb={})",
            self.language, self.num_threads, chunk_kb
        )
    }
}
