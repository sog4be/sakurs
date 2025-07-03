//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::error::SakursError;
use crate::types::{PyProcessingResult, PyProcessorConfig};
use pyo3::prelude::*;
use sakurs_core::application::UnifiedProcessor;
use sakurs_core::domain::language::{EnglishLanguageRules, JapaneseLanguageRules, LanguageRules};
use std::sync::Arc;

/// Main processor class for sentence boundary detection
#[pyclass]
pub struct PyProcessor {
    processor: UnifiedProcessor,
    language: String,
}

#[pymethods]
impl PyProcessor {
    /// Create a new processor for the specified language
    #[new]
    #[pyo3(signature = (language="en", config=None))]
    pub fn new(language: &str, config: Option<PyProcessorConfig>) -> PyResult<Self> {
        let language_rules: Arc<dyn LanguageRules> = match language.to_lowercase().as_str() {
            "en" | "english" => Arc::new(EnglishLanguageRules::new()),
            "ja" | "japanese" => Arc::new(JapaneseLanguageRules::new()),
            _ => return Err(SakursError::UnsupportedLanguage(language.to_string()).into()),
        };

        let processor = if let Some(cfg) = config {
            UnifiedProcessor::with_config(language_rules, cfg.into())
        } else {
            UnifiedProcessor::new(language_rules)
        };

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
                if let Some(thread_count) = threads {
                    self.processor.process_with_threads(text, thread_count)
                } else {
                    self.processor.process(text)
                }
            })
            .map_err(SakursError::from)?;

        Ok(PyProcessingResult::new(
            output.boundaries,
            output.metrics,
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
