//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::error::SakursError;
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
            _ => return Err(SakursError::UnsupportedLanguage(language.to_string()).into()),
        };

        let processor = if let Some(cfg) = config {
            let rust_config = Config::builder()
                .language(lang_code)
                .chunk_size(cfg.chunk_size / 1024) // Convert bytes to KB
                .threads(cfg.num_threads.unwrap_or(0))
                .build()
                .map_err(|e| SakursError::ProcessingError(e.to_string()))?;
            SentenceProcessor::with_config(rust_config)
        } else {
            SentenceProcessor::for_language(lang_code)
        }
        .map_err(|e| SakursError::ProcessingError(e.to_string()))?;

        Ok(Self {
            processor,
            language: language.to_string(),
        })
    }

    /// Process text and return detailed results with boundaries and metrics
    #[pyo3(signature = (text, threads=None))]
    fn process(
        &self,
        text: &str,
        threads: Option<usize>,
        py: Python,
    ) -> PyResult<PyProcessingResult> {
        // Release GIL during processing for better performance
        let output = py
            .allow_threads(|| {
                // Note: The new API doesn't have explicit thread control per call
                // Thread configuration is set during processor creation
                if threads.is_some() {
                    eprintln!("Warning: per-call thread count is not supported in the new API. Use config.num_threads instead.");
                }
                self.processor.process(Input::from_text(text))
            })
            .map_err(|e| SakursError::ProcessingError(e.to_string()))?;

        // Convert new API output to Python types
        let boundaries: Vec<usize> = output.boundaries.iter().map(|b| b.offset).collect();

        Ok(PyProcessingResult::new(
            boundaries,
            output.metadata.stats.clone(),
            text.to_string(),
        ))
    }

    /// Extract sentences as a list of strings (convenience method)
    #[pyo3(signature = (text, threads=None))]
    pub fn sentences(
        &self,
        text: &str,
        threads: Option<usize>,
        py: Python,
    ) -> PyResult<Vec<String>> {
        let result = self.process(text, threads, py)?;
        Ok(result.sentences())
    }

    /// Get processor configuration (placeholder - needs to be implemented in UnifiedProcessor)
    #[getter]
    fn config(&self) -> PyProcessorConfig {
        // For now, return default values until config() method is added to UnifiedProcessor
        PyProcessorConfig::new(8192, 256, None)
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
