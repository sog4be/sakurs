//! Core processor Python interface

#![allow(non_local_definitions)]

use crate::exceptions::InternalError;
use crate::input::PyInput;
use crate::language_config::LanguageConfig;
use crate::types::PyProcessingResult;
use pyo3::prelude::*;
use sakurs_core::{Config, SentenceProcessor};

/// Resolve public execution options to the explicit core thread setting.
///
/// The core represents adaptive execution as `None`, so forced parallel modes
/// must resolve the available worker count before building the configuration.
pub(crate) fn resolve_execution_threads(
    execution_mode: &str,
    threads: Option<usize>,
    force_parallel: bool,
) -> Result<Option<usize>, InternalError> {
    if !matches!(execution_mode, "sequential" | "parallel" | "adaptive") {
        return Err(InternalError::ConfigurationError(format!(
            "Invalid execution_mode: {execution_mode}"
        )));
    }

    if force_parallel || execution_mode == "parallel" {
        return Ok(Some(threads.unwrap_or_else(available_worker_threads)));
    }

    match execution_mode {
        "sequential" => Ok(Some(1)),
        "adaptive" => Ok(threads),
        _ => unreachable!("execution mode was validated above"),
    }
}

fn available_worker_threads() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
}

/// Main sentence splitter class for sentence boundary detection
#[pyclass(name = "SentenceSplitter")]
pub struct PyProcessor {
    processor: SentenceProcessor,
    language: String,
    chunk_size: usize,
    num_threads: Option<usize>,
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

        // Build Rust configuration and optionally a custom language config
        let (mut config_builder, language_display, custom_language) =
            if let Some(lang_config) = language_config {
                // Use custom language configuration
                let core_config = lang_config.to_core_config(py)?;
                let display_name = format!("custom({})", lang_config.metadata.code);

                (
                    Config::builder()
                        .language("en") // Default, will be overridden
                        .map_err(|e| InternalError::ConfigurationError(e.to_string()))?,
                    display_name,
                    Some(core_config),
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
                    None,
                )
            };

        // Handle execution mode. Parallel mode resolves an explicit worker
        // count so it remains parallel even for short inputs.
        let resolved_threads = resolve_execution_threads(execution_mode, threads, false)?;
        if let Some(thread_count) = resolved_threads {
            config_builder = config_builder.threads(Some(thread_count));
        }

        config_builder = config_builder.chunk_size(chunk_size_bytes);

        let rust_config = config_builder
            .build()
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        // Create processor with a custom language config if provided
        let processor = if let Some(language) = custom_language {
            SentenceProcessor::with_language_config(rust_config, &language)
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?
        } else {
            SentenceProcessor::with_config(rust_config)
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?
        };

        Ok(Self {
            processor,
            language: language_display,
            chunk_size: chunk_size_bytes,
            num_threads: resolved_threads,
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
    ) -> PyResult<Py<PyAny>> {
        use crate::output::boundaries_to_sentences_with_char_offsets;
        use pyo3::types::PyList;

        // Extract input from Python object
        let py_input = PyInput::from_py_object(py, input)?;

        // Convert to core Input type and get the text content
        let (core_input, text) = py_input.into_core_input_and_text(py, encoding)?;

        // Release GIL during processing for better performance
        let output = py
            .detach(|| self.processor.process(core_input))
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

    /// Load the input and iterate over the resulting sentences
    #[pyo3(signature = (input, *, encoding="utf-8", preserve_whitespace=false))]
    pub fn iter_split(
        &self,
        input: &Bound<'_, PyAny>,
        encoding: &str,
        preserve_whitespace: bool,
        py: Python,
    ) -> PyResult<crate::iterator::SentenceIterator> {
        use crate::stream::create_iter_split_iterator_from_processor;

        create_iter_split_iterator_from_processor(
            py,
            input,
            &self.processor,
            preserve_whitespace,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_execution_modes() {
        assert_eq!(
            resolve_execution_threads("sequential", None, false).unwrap(),
            Some(1)
        );
        assert_eq!(
            resolve_execution_threads("adaptive", None, false).unwrap(),
            None
        );
        assert_eq!(
            resolve_execution_threads("adaptive", Some(3), false).unwrap(),
            Some(3)
        );
        assert_eq!(
            resolve_execution_threads("parallel", Some(2), false).unwrap(),
            Some(2)
        );
    }

    #[test]
    fn resolves_forced_parallel_without_explicit_threads() {
        assert_eq!(
            resolve_execution_threads("parallel", None, false).unwrap(),
            Some(available_worker_threads())
        );
        assert_eq!(
            resolve_execution_threads("sequential", None, true).unwrap(),
            Some(available_worker_threads())
        );
    }

    #[test]
    fn rejects_invalid_execution_mode() {
        assert!(resolve_execution_threads("invalid", None, false).is_err());
        assert!(resolve_execution_threads("invalid", None, true).is_err());
    }
}
