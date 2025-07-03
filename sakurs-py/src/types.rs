//! Python type definitions and conversions

#![allow(non_local_definitions)]

use pyo3::prelude::*;
use sakurs_core::application::{ProcessingMetrics, ProcessorConfig};
use sakurs_core::Boundary;

/// Python wrapper for sentence boundary information
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyBoundary {
    /// Character offset in the original text
    #[pyo3(get)]
    pub offset: usize,

    /// Whether this boundary marks the end of a sentence
    #[pyo3(get)]
    pub is_sentence_end: bool,

    /// Confidence score for this boundary (0.0-1.0)
    #[pyo3(get)]
    pub confidence: f32,
}

#[pymethods]
impl PyBoundary {
    #[new]
    fn new(offset: usize, is_sentence_end: bool, confidence: Option<f32>) -> Self {
        Self {
            offset,
            is_sentence_end,
            confidence: confidence.unwrap_or(1.0),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Boundary(offset={}, is_sentence_end={}, confidence={:.2})",
            self.offset, self.is_sentence_end, self.confidence
        )
    }
}

impl From<Boundary> for PyBoundary {
    fn from(boundary: Boundary) -> Self {
        Self {
            offset: boundary.offset,
            is_sentence_end: true, // All Boundary instances are sentence ends
            confidence: 1.0,
        }
    }
}

/// Python wrapper for processing configuration
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyProcessorConfig {
    /// Size of text chunks for parallel processing
    #[pyo3(get, set)]
    pub chunk_size: usize,

    /// Overlap size between chunks
    #[pyo3(get, set)]
    pub overlap_size: usize,

    /// Maximum number of threads to use (None for automatic)
    #[pyo3(get, set)]
    pub max_threads: Option<usize>,
}

#[pymethods]
impl PyProcessorConfig {
    #[new]
    #[pyo3(signature = (chunk_size=8192, overlap_size=256, max_threads=None))]
    pub fn new(chunk_size: usize, overlap_size: usize, max_threads: Option<usize>) -> Self {
        Self {
            chunk_size,
            overlap_size,
            max_threads,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ProcessorConfig(chunk_size={}, overlap_size={}, max_threads={:?})",
            self.chunk_size, self.overlap_size, self.max_threads
        )
    }
}

impl From<PyProcessorConfig> for ProcessorConfig {
    fn from(py_config: PyProcessorConfig) -> Self {
        ProcessorConfig::builder()
            .chunk_size(py_config.chunk_size)
            .overlap_size(py_config.overlap_size)
            .max_threads(py_config.max_threads)
            .build()
            .expect("ProcessorConfig should be valid with these parameters")
    }
}

impl From<ProcessorConfig> for PyProcessorConfig {
    fn from(config: ProcessorConfig) -> Self {
        Self {
            chunk_size: config.chunk_size,
            overlap_size: config.overlap_size,
            max_threads: config.max_threads,
        }
    }
}

/// Python wrapper for processing metrics
#[pyclass]
#[derive(Clone, Debug)]
pub struct PyProcessingMetrics {
    /// Number of sentence boundaries found
    #[pyo3(get)]
    pub boundaries_found: usize,

    /// Number of chunks processed
    #[pyo3(get)]
    pub chunk_count: usize,

    /// Number of threads used
    #[pyo3(get)]
    pub thread_count: usize,

    /// Total processing time in microseconds
    #[pyo3(get)]
    pub total_time_us: u64,

    /// Time spent on chunking in microseconds
    #[pyo3(get)]
    pub chunking_time_us: u64,

    /// Time spent on parallel processing in microseconds
    #[pyo3(get)]
    pub parallel_time_us: u64,

    /// Time spent on merging results in microseconds
    #[pyo3(get)]
    pub merge_time_us: u64,
}

#[pymethods]
impl PyProcessingMetrics {
    fn __repr__(&self) -> String {
        format!(
            "ProcessingMetrics(boundaries={}, chunks={}, threads={}, total_time={}μs)",
            self.boundaries_found, self.chunk_count, self.thread_count, self.total_time_us
        )
    }

    /// Get processing speed in characters per second
    #[getter]
    fn chars_per_second(&self) -> f64 {
        if self.total_time_us == 0 {
            0.0
        } else {
            // This would need the original text length, placeholder for now
            1_000_000.0 / self.total_time_us as f64
        }
    }
}

impl From<ProcessingMetrics> for PyProcessingMetrics {
    fn from(metrics: ProcessingMetrics) -> Self {
        Self {
            boundaries_found: metrics.boundaries_found,
            chunk_count: metrics.chunk_count,
            thread_count: metrics.thread_count,
            total_time_us: metrics.total_time_us,
            chunking_time_us: metrics.chunking_time_us,
            parallel_time_us: metrics.parallel_time_us,
            merge_time_us: metrics.merge_time_us,
        }
    }
}

/// Result of text processing containing boundaries and metrics
#[pyclass]
pub struct PyProcessingResult {
    /// List of detected sentence boundaries
    #[pyo3(get)]
    pub boundaries: Vec<PyBoundary>,

    /// Processing performance metrics
    #[pyo3(get)]
    pub metrics: PyProcessingMetrics,

    /// Original text (kept for sentence extraction)
    original_text: String,
}

#[pymethods]
impl PyProcessingResult {
    /// Extract sentences as a list of strings
    pub fn sentences(&self) -> Vec<String> {
        // For now, return simple split until we fix the boundary issue
        if self.boundaries.is_empty() {
            return vec![self.original_text.clone()];
        }

        // TODO: Fix proper boundary-based sentence extraction
        // For now, use a simple implementation that handles basic cases
        let mut sentences = Vec::new();
        let text = &self.original_text;

        // If we have boundaries, use them as split points
        if !self.boundaries.is_empty() {
            let mut last_end = 0;

            for boundary in &self.boundaries {
                if boundary.is_sentence_end && boundary.offset < text.len() {
                    // Extract sentence up to and including the boundary character
                    let end = (boundary.offset + 1).min(text.len());
                    if let Some(sentence) = text.get(last_end..end) {
                        let trimmed = sentence.trim();
                        if !trimmed.is_empty() {
                            sentences.push(trimmed.to_string());
                        }
                        last_end = end;
                    }
                }
            }

            // Add any remaining text
            if last_end < text.len() {
                if let Some(remaining) = text.get(last_end..) {
                    let trimmed = remaining.trim();
                    if !trimmed.is_empty() {
                        sentences.push(trimmed.to_string());
                    }
                }
            }
        }

        // Fallback if no sentences were extracted
        if sentences.is_empty() {
            vec![self.original_text.clone()]
        } else {
            sentences
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ProcessingResult(boundaries={}, text_length={})",
            self.boundaries.len(),
            self.original_text.len()
        )
    }
}

impl PyProcessingResult {
    pub fn new(
        boundaries: Vec<Boundary>,
        metrics: ProcessingMetrics,
        original_text: String,
    ) -> Self {
        Self {
            boundaries: boundaries.into_iter().map(PyBoundary::from).collect(),
            metrics: PyProcessingMetrics::from(metrics),
            original_text,
        }
    }
}
