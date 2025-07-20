//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::exceptions::InternalError;
use crate::input::PyInput;
use crate::types::{PyProcessingResult, PyProcessorConfig};
use pyo3::prelude::*;
use sakurs_core::{Config, SentenceProcessor};

/// Main processor class for sentence boundary detection
#[pyclass]
pub struct PyProcessor {
    processor: SentenceProcessor,
    language: String,
    config: PyProcessorConfig,
}

#[pymethods]
impl PyProcessor {
    /// Create a new processor for the specified language
    #[new]
    #[pyo3(signature = (*, language=None, threads=None, chunk_size=None, execution_mode="adaptive", streaming=false, stream_chunk_size=10*1024*1024))]
    pub fn new(
        language: Option<&str>,
        threads: Option<usize>,
        chunk_size: Option<usize>,
        execution_mode: &str,
        streaming: bool,
        stream_chunk_size: usize,
    ) -> PyResult<Self> {
        // Validate language
        let lang = language.unwrap_or("en");
        let lang_code = match lang.to_lowercase().as_str() {
            "en" | "english" => "en",
            "ja" | "japanese" => "ja",
            _ => return Err(InternalError::UnsupportedLanguage(lang.to_string()).into()),
        };

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

        // Build Rust configuration
        let mut config_builder = Config::builder()
            .language(lang_code)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

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

        let processor = SentenceProcessor::with_config(rust_config)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        Ok(Self {
            processor,
            language: lang.to_string(),
            config: py_config,
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
