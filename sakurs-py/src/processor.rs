//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::exceptions::InternalError;
use crate::input::PyInput;
use crate::language_config::LanguageConfig;
use crate::types::{PyProcessingResult, PyProcessorConfig};
use pyo3::prelude::*;
use sakurs_core::{Config, SentenceProcessor};

/// Main processor class for sentence boundary detection
#[pyclass]
pub struct PyProcessor {
    processor: SentenceProcessor,
    language: String,
    config: PyProcessorConfig,
    #[allow(dead_code)]
    custom_config: bool, // Track if using custom language config
}

#[pymethods]
impl PyProcessor {
    /// Create a new processor for the specified language
    #[new]
    #[pyo3(signature = (*, language=None, language_config=None, threads=None, chunk_size=None, execution_mode="adaptive", streaming=false, stream_chunk_size=10*1024*1024))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        language: Option<&str>,
        language_config: Option<LanguageConfig>,
        threads: Option<usize>,
        chunk_size: Option<usize>,
        execution_mode: &str,
        streaming: bool,
        stream_chunk_size: usize,
        py: Python,
    ) -> PyResult<Self> {
        // Create Python config for internal use
        let py_config = PyProcessorConfig::new(
            chunk_size.unwrap_or(if streaming {
                stream_chunk_size
            } else {
                256 * 1024
            }),
            256, // overlap_size
            threads,
            1024 * 1024, // parallel_threshold
        );

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

        if let Some(cs) = chunk_size {
            config_builder = config_builder.chunk_size(cs);
        } else if streaming {
            config_builder = config_builder.chunk_size(stream_chunk_size);
        }

        config_builder = config_builder
            .parallel_threshold(py_config.parallel_threshold)
            .overlap_size(py_config.overlap_size);

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
            config: py_config,
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
    #[pyo3(signature = (input, *, encoding="utf-8", preserve_whitespace=false))]
    pub fn iter_split(
        &self,
        input: &Bound<'_, PyAny>,
        encoding: &str,
        preserve_whitespace: bool,
        py: Python,
    ) -> PyResult<crate::iterator::SentenceIterator> {
        use crate::stream::create_stream_iterator;

        // Extract language from processor
        let language = if self.custom_config {
            None // Custom config already embedded in processor
        } else {
            Some(self.language.as_str())
        };

        // Use the processor's configuration for streaming
        let chunk_size_mb = self.config.chunk_size / (1024 * 1024);
        let chunk_size_mb = if chunk_size_mb > 0 { chunk_size_mb } else { 10 };

        create_stream_iterator(
            py,
            input,
            language,
            None, // language_config already in processor
            chunk_size_mb,
            self.config.overlap_size,
            encoding,
            preserve_whitespace,
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
        format!(
            "Processor(language='{}', threads={:?}, chunk_size={})",
            self.language, self.config.num_threads, self.config.chunk_size
        )
    }
}
