//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::exceptions::InternalError;
use crate::types::{PyProcessingResult, PyProcessorConfig};
use pyo3::prelude::*;
use sakurs_core::{Config, Input, SentenceProcessor};

/// Python interface for the sentence processor
#[pyclass(name = "Processor")]
pub struct PyProcessor {
    processor: SentenceProcessor,
    language: String,
}

impl PyProcessor {
    /// Create a new processor for the specified language
    pub fn new(language: &str, config: Option<PyProcessorConfig>) -> PyResult<Self> {
        let lang_code = match language.to_lowercase().as_str() {
            "en" | "english" => "en",
            "ja" | "japanese" => "ja",
            _ => return Err(InternalError::UnsupportedLanguage(language.to_string()).into()),
        };

        let processor = if let Some(cfg) = config {
            // Build with custom configuration
            let chunk_size = cfg.chunk_size.min(cfg.max_chunk_size);

            let rust_config = Config::builder()
                .language(lang_code)
                .map_err(InternalError::from)?
                .chunk_size(chunk_size) // Now in bytes, no conversion needed
                .overlap_size(cfg.overlap_size)
                .threads(cfg.num_threads)
                .build()
                .map_err(InternalError::from)?;
            SentenceProcessor::with_config(rust_config)
        } else {
            // Use default configuration
            SentenceProcessor::with_language(lang_code)
        }
        .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        Ok(PyProcessor {
            processor,
            language: language.to_string(),
        })
    }

    /// Process text and return detailed boundary information (internal method, not exposed to Python)
    pub(crate) fn process_with_details(
        &self,
        text: &str,
        threads: Option<usize>,
        py: Python,
    ) -> PyResult<sakurs_core::api::Output> {
        // Apply thread override if provided
        let output = if let Some(num_threads) = threads {
            // Warn about threads parameter deprecation
            let warnings = py.import("warnings")?;
            warnings.call_method1(
                "warn",
                (
                    "The 'threads' parameter is deprecated. Configure threads when creating the Processor.",
                    py.get_type::<pyo3::exceptions::PyDeprecationWarning>(),
                )
            )?;

            // Create a new processor with overridden thread count
            let config = Config::builder()
                .language(&self.language)
                .map_err(InternalError::from)?
                .threads(Some(num_threads))
                .build()
                .map_err(InternalError::from)?;
            let processor = SentenceProcessor::with_config(config).map_err(InternalError::from)?;
            py.allow_threads(|| processor.process(Input::from_text(text)))
                .map_err(InternalError::from)?
        } else {
            py.allow_threads(|| self.processor.process(Input::from_text(text)))
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?
        };

        Ok(output)
    }
}

#[pymethods]
impl PyProcessor {
    #[new]
    #[pyo3(signature = (language="en", config=None))]
    fn __new__(language: &str, config: Option<PyProcessorConfig>) -> PyResult<Self> {
        Self::new(language, config)
    }

    /// Split text into sentences
    #[pyo3(signature = (text, threads=None))]
    pub fn split(&self, text: &str, threads: Option<usize>, py: Python) -> PyResult<Vec<String>> {
        // Warn about deprecated threads parameter using Python warnings module
        if threads.is_some() {
            let warnings = py.import("warnings")?;
            warnings.call_method1(
                "warn",
                (
                    "The 'threads' parameter is deprecated. Configure threads when creating the Processor.",
                    py.get_type::<pyo3::exceptions::PyDeprecationWarning>(),
                )
            )?;
        }

        // Release GIL during processing for better performance
        let output = py
            .allow_threads(|| self.processor.process(Input::from_text(text)))
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        // Convert boundaries to sentence list
        let boundaries: Vec<usize> = output.boundaries.iter().map(|b| b.offset).collect();
        let result = PyProcessingResult::new(boundaries, output.metadata.stats, text.to_string());

        Ok(result.sentences())
    }

    /// Extract sentences as a list of strings (legacy method, use split() instead)
    #[pyo3(signature = (text, threads=None))]
    pub fn sentences(
        &self,
        text: &str,
        threads: Option<usize>,
        py: Python,
    ) -> PyResult<Vec<String>> {
        self.split(text, threads, py)
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

    fn __repr__(&self) -> String {
        format!("Processor(language='{}')", self.language)
    }
}
