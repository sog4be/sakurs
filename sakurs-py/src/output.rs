use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

/// Represents a single sentence with detailed metadata.
#[pyclass]
pub struct Sentence {
    /// The actual text content of the sentence
    #[pyo3(get)]
    pub text: String,

    /// Character offset where the sentence starts in the original text
    #[pyo3(get)]
    pub start: usize,

    /// Character offset where the sentence ends in the original text
    #[pyo3(get)]
    pub end: usize,

    /// Confidence score for sentence boundary detection (currently always 1.0)
    #[pyo3(get)]
    pub confidence: f32,

    /// Additional metadata about the sentence
    #[pyo3(get)]
    pub metadata: Py<PyDict>,
}

#[pymethods]
impl Sentence {
    #[new]
    #[pyo3(signature = (text, start, end, confidence=1.0, metadata=None))]
    fn new(
        py: Python,
        text: String,
        start: usize,
        end: usize,
        confidence: Option<f32>,
        metadata: Option<HashMap<String, PyObject>>,
    ) -> PyResult<Self> {
        let metadata_dict = PyDict::new(py);

        if let Some(meta) = metadata {
            for (key, value) in meta {
                metadata_dict.set_item(key, value)?;
            }
        }

        Ok(Self {
            text,
            start,
            end,
            confidence: confidence.unwrap_or(1.0),
            metadata: metadata_dict.into(),
        })
    }

    fn __repr__(&self, _py: Python) -> PyResult<String> {
        let text_preview = if self.text.len() > 50 {
            format!("{}...", &self.text[..50])
        } else {
            self.text.clone()
        };

        Ok(format!(
            "Sentence(text='{}', start={}, end={}, confidence={})",
            text_preview, self.start, self.end, self.confidence
        ))
    }

    fn __str__(&self) -> &str {
        &self.text
    }

    fn __len__(&self) -> usize {
        self.text.len()
    }
}

/// Processing statistics and metadata for sentence detection.
#[pyclass]
#[derive(Clone)]
pub struct ProcessingMetadata {
    /// Total number of sentences detected
    #[pyo3(get)]
    pub total_sentences: usize,

    /// Processing time in milliseconds
    #[pyo3(get)]
    pub processing_time_ms: f64,

    /// Number of threads used for processing
    #[pyo3(get)]
    pub threads_used: usize,

    /// Chunk size used for processing (in bytes)
    #[pyo3(get)]
    pub chunk_size_used: usize,

    /// Execution mode used ("sequential", "parallel", or "adaptive")
    #[pyo3(get)]
    pub execution_mode_used: String,
}

#[pymethods]
impl ProcessingMetadata {
    #[new]
    fn new(
        total_sentences: usize,
        processing_time_ms: f64,
        threads_used: usize,
        chunk_size_used: usize,
        execution_mode_used: String,
    ) -> Self {
        Self {
            total_sentences,
            processing_time_ms,
            threads_used,
            chunk_size_used,
            execution_mode_used,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ProcessingMetadata(total_sentences={}, processing_time_ms={:.2}, threads_used={}, chunk_size_used={}, execution_mode_used='{}')",
            self.total_sentences,
            self.processing_time_ms,
            self.threads_used,
            self.chunk_size_used,
            self.execution_mode_used
        )
    }
}

/// Helper function to create a list of Sentence objects from processing results
pub fn create_sentence_list(
    py: Python,
    text: &str,
    boundaries: &[sakurs_core::api::Boundary],
) -> PyResult<Vec<Sentence>> {
    let mut sentences = Vec::new();
    let mut last_end = 0;

    for boundary in boundaries {
        // Get the sentence text from last_end to current boundary
        let sentence_text = text
            .chars()
            .skip(last_end)
            .take(boundary.char_offset - last_end)
            .collect::<String>();

        if !sentence_text.trim().is_empty() {
            sentences.push(Sentence::new(
                py,
                sentence_text,
                last_end,
                boundary.char_offset,
                None,
                None,
            )?);
        }

        last_end = boundary.char_offset;
    }

    // Handle any remaining text after the last boundary
    if last_end < text.chars().count() {
        let remaining_text = text.chars().skip(last_end).collect::<String>();

        if !remaining_text.trim().is_empty() {
            sentences.push(Sentence::new(
                py,
                remaining_text,
                last_end,
                text.chars().count(),
                None,
                None,
            )?);
        }
    }

    Ok(sentences)
}
