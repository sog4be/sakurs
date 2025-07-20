//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::exceptions::InternalError;
use crate::types::{PyProcessingResult, PyProcessorConfig};
use pyo3::prelude::*;
use sakurs_core::{Config, Input, SentenceProcessor};

/// Main processor class for sentence boundary detection
#[pyclass]
pub struct PyProcessor {
    processor: SentenceProcessor,
    language: String,
}

#[pymethods]
impl PyProcessor {
    /// Create a new processor for the specified language
    #[new]
    #[pyo3(signature = (language="en", config=None))]
    pub fn new(language: &str, config: Option<PyProcessorConfig>) -> PyResult<Self> {
        // Validate language
        let lang_code = match language.to_lowercase().as_str() {
            "en" | "english" => "en",
            "ja" | "japanese" => "ja",
            _ => return Err(InternalError::UnsupportedLanguage(language.to_string()).into()),
        };

        let processor = if let Some(cfg) = config {
            let rust_config = Config::builder()
                .language(lang_code)
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?
                .chunk_size(cfg.chunk_size) // Now in bytes, no conversion needed
                .parallel_threshold(cfg.parallel_threshold)
                .overlap_size(cfg.overlap_size)
                .threads(cfg.num_threads)
                .build()
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?;
            SentenceProcessor::with_config(rust_config)
        } else {
            SentenceProcessor::with_language(lang_code)
        }
        .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        Ok(Self {
            processor,
            language: language.to_string(),
        })
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
