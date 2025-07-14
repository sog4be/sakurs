//! Python type definitions and conversions

#![allow(non_local_definitions)]

use pyo3::prelude::*;
use sakurs_core::ProcessingStats;

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

    /// Number of threads to use for processing
    #[pyo3(get, set)]
    pub num_threads: Option<usize>,

    /// Minimum text size to trigger parallel processing
    #[pyo3(get, set)]
    pub parallel_threshold: usize,
}

#[pymethods]
impl PyProcessorConfig {
    #[new]
    #[pyo3(signature = (chunk_size=65536, overlap_size=256, num_threads=None, parallel_threshold=1048576))]
    pub fn new(
        chunk_size: usize,
        overlap_size: usize,
        num_threads: Option<usize>,
        parallel_threshold: usize,
    ) -> Self {
        Self {
            chunk_size,
            overlap_size,
            num_threads,
            parallel_threshold,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ProcessorConfig(chunk_size={}, overlap_size={}, num_threads={:?}, parallel_threshold={})",
            self.chunk_size, self.overlap_size, self.num_threads, self.parallel_threshold
        )
    }
}

/// Internal result for processing - not exposed to Python
pub struct PyProcessingResult {
    /// Boundary offsets
    pub boundaries: Vec<usize>,
    /// Processing statistics (unused but required by core API)
    pub _stats: ProcessingStats,
    /// Original text for sentence extraction
    pub original_text: String,
}

impl PyProcessingResult {
    pub fn new(boundaries: Vec<usize>, stats: ProcessingStats, original_text: String) -> Self {
        Self {
            boundaries,
            _stats: stats,
            original_text,
        }
    }

    /// Extract sentences as a list of strings
    pub fn sentences(&self) -> Vec<String> {
        if self.boundaries.is_empty() {
            // No boundaries found, return the whole text as one sentence
            return vec![self.original_text.trim().to_string()];
        }

        let mut sentences = Vec::new();
        let text = &self.original_text;
        let mut last_end = 0;

        for &boundary_offset in &self.boundaries {
            // boundary_offset points to the sentence-ending punctuation
            // We want to include it in the sentence
            let end = (boundary_offset + 1).min(text.len());

            if end > last_end {
                if let Some(sentence) = text.get(last_end..end) {
                    let trimmed = sentence.trim();
                    if !trimmed.is_empty() {
                        sentences.push(trimmed.to_string());
                    }
                }
                last_end = end;
            }
        }

        // Add any remaining text after the last boundary
        if last_end < text.len() {
            if let Some(remaining) = text.get(last_end..) {
                let trimmed = remaining.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_string());
                }
            }
        }

        // Return the extracted sentences, or the original text if nothing was extracted
        if sentences.is_empty() {
            vec![self.original_text.trim().to_string()]
        } else {
            sentences
        }
    }
}
