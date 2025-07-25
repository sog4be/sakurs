//! Output types for Python bindings

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Sentence with metadata
#[pyclass]
pub struct Sentence {
    /// The text content of the sentence
    #[pyo3(get)]
    pub text: String,

    /// Character offset where the sentence starts in the original text
    #[pyo3(get)]
    pub start: usize,

    /// Character offset where the sentence ends in the original text
    #[pyo3(get)]
    pub end: usize,

    /// Confidence score for the sentence boundary (future extension)
    #[pyo3(get)]
    pub confidence: f32,

    /// Additional metadata as a dictionary
    #[pyo3(get)]
    pub metadata: Py<PyAny>,
}

#[pymethods]
impl Sentence {
    /// Create a new Sentence instance
    #[new]
    #[pyo3(signature = (text, start, end, confidence=1.0, metadata=None))]
    pub fn new(
        text: String,
        start: usize,
        end: usize,
        confidence: Option<f32>,
        metadata: Option<Bound<'_, PyDict>>,
        py: Python,
    ) -> PyResult<Self> {
        let metadata = if let Some(dict) = metadata {
            dict.unbind()
        } else {
            PyDict::new(py).into()
        };

        Ok(Self {
            text,
            start,
            end,
            confidence: confidence.unwrap_or(1.0),
            metadata: metadata.into(),
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "Sentence(text='{}', start={}, end={}, confidence={})",
            self.text, self.start, self.end, self.confidence
        )
    }

    fn __str__(&self) -> String {
        self.text.clone()
    }
}

/// Processing statistics and metadata
#[pyclass]
#[derive(Clone)]
pub struct ProcessingMetadata {
    /// Total number of sentences found
    #[pyo3(get)]
    pub total_sentences: usize,

    /// Processing time in milliseconds
    #[pyo3(get)]
    pub processing_time_ms: f64,

    /// Number of threads used for processing
    #[pyo3(get)]
    pub threads_used: usize,

    /// Chunk size used for processing (in KB)
    #[pyo3(get)]
    pub chunk_kb_used: usize,

    /// Execution mode used ("sequential", "parallel", or "adaptive")
    #[pyo3(get)]
    pub execution_mode_used: String,
}

#[pymethods]
impl ProcessingMetadata {
    /// Create a new ProcessingMetadata instance
    #[new]
    pub fn new(
        total_sentences: usize,
        processing_time_ms: f64,
        threads_used: usize,
        chunk_kb_used: usize,
        execution_mode_used: String,
    ) -> Self {
        Self {
            total_sentences,
            processing_time_ms,
            threads_used,
            chunk_kb_used,
            execution_mode_used,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ProcessingMetadata(total_sentences={}, processing_time_ms={:.2}, threads_used={}, chunk_kb_used={}, execution_mode_used='{}')",
            self.total_sentences,
            self.processing_time_ms,
            self.threads_used,
            self.chunk_kb_used,
            self.execution_mode_used
        )
    }
}

/// Helper function to convert boundaries and text into Sentence objects using character offsets
pub fn boundaries_to_sentences_with_char_offsets(
    text: &str,
    boundaries: &[(usize, usize)], // (char_offset, byte_offset)
    preserve_whitespace: bool,
    py: Python,
) -> PyResult<Vec<Sentence>> {
    let mut sentences = Vec::new();
    let mut start_char = 0;
    let mut start_byte = 0;

    for &(end_char, end_byte) in boundaries {
        if end_char > start_char && end_byte <= text.len() {
            let sentence_text = text[start_byte..end_byte].to_string();

            // Calculate offsets based on whether we trim whitespace
            let (final_text, final_start, final_end) = if preserve_whitespace {
                (sentence_text, start_char, end_char)
            } else {
                // Trim the text but keep original offsets
                let trimmed = sentence_text.trim();
                if trimmed.is_empty() {
                    // Skip empty sentences
                    start_char = end_char;
                    start_byte = end_byte;
                    continue;
                }
                (trimmed.to_string(), start_char, end_char)
            };

            let sentence = Sentence::new(final_text, final_start, final_end, Some(1.0), None, py)?;
            sentences.push(sentence);
            start_char = end_char;
            start_byte = end_byte;
        }
    }

    // Handle any remaining text after the last boundary
    if start_byte < text.len() {
        let sentence_text = text[start_byte..].to_string();
        let char_count = text.chars().count();

        let (final_text, final_start, final_end) = if preserve_whitespace {
            (sentence_text, start_char, char_count)
        } else {
            // Trim the text but keep original offsets
            let trimmed = sentence_text.trim();
            if !trimmed.is_empty() {
                (trimmed.to_string(), start_char, char_count)
            } else {
                // Skip empty sentences
                return Ok(sentences);
            }
        };

        let sentence = Sentence::new(final_text, final_start, final_end, Some(1.0), None, py)?;
        sentences.push(sentence);
    }

    Ok(sentences)
}
