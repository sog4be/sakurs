//! Python iterator implementation for streaming sentence splitting

use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;
use sakurs_core::Input;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Python iterator for streaming sentences
#[pyclass]
pub struct SentenceIterator {
    /// Shared state for streaming processing
    state: Arc<Mutex<IteratorState>>,
}

/// Internal state for the iterator
pub(crate) struct IteratorState {
    /// Buffer of pending sentences to yield
    sentence_buffer: VecDeque<String>,
    /// Current position in the text (byte offset)
    #[allow(dead_code)]
    position: usize,
    /// Whether we've reached end of input
    exhausted: bool,
    /// Text buffer for incomplete sentences
    text_buffer: String,
    /// Whether to preserve whitespace
    preserve_whitespace: bool,
}

impl IteratorState {
    fn new(preserve_whitespace: bool) -> Self {
        Self {
            sentence_buffer: VecDeque::new(),
            position: 0,
            exhausted: false,
            text_buffer: String::new(),
            preserve_whitespace,
        }
    }
}

#[pymethods]
impl SentenceIterator {
    /// Create a new sentence iterator
    #[new]
    fn new(preserve_whitespace: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(IteratorState::new(preserve_whitespace))),
        }
    }

    /// Python iterator protocol: return self
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    /// Python iterator protocol: get next sentence
    fn __next__(&self) -> PyResult<Option<String>> {
        let mut state = self.state.lock().unwrap();

        // If we have buffered sentences, return one
        if let Some(sentence) = state.sentence_buffer.pop_front() {
            return Ok(Some(sentence));
        }

        // If exhausted and no more sentences, stop iteration
        if state.exhausted {
            return Err(PyStopIteration::new_err(()));
        }

        // In a real implementation, we would:
        // 1. Read more data from the input source
        // 2. Process it to find sentences
        // 3. Buffer complete sentences
        // 4. Return the first one
        // For now, we'll implement the interface

        Err(PyStopIteration::new_err(()))
    }

    /// Add sentences to the buffer (internal method for feeding data)
    pub fn add_sentences(&self, sentences: Vec<String>) -> PyResult<()> {
        let mut state = self.state.lock().unwrap();
        state.sentence_buffer.extend(sentences);
        Ok(())
    }

    /// Mark the iterator as exhausted (no more input)
    pub fn mark_exhausted(&self) -> PyResult<()> {
        let mut state = self.state.lock().unwrap();
        state.exhausted = true;
        Ok(())
    }
}

impl SentenceIterator {
    /// Get the internal state for coordination with streaming processor (internal use only)
    pub(crate) fn get_state(&self) -> Arc<Mutex<IteratorState>> {
        Arc::clone(&self.state)
    }

    /// Create a new iterator for internal use
    pub(crate) fn new_internal(preserve_whitespace: bool) -> Self {
        Self {
            state: Arc::new(Mutex::new(IteratorState::new(preserve_whitespace))),
        }
    }
}

/// Helper function to process text incrementally and yield sentences
pub(crate) fn process_text_incrementally(
    text: &str,
    state: &Arc<Mutex<IteratorState>>,
    processor: &sakurs_core::SentenceProcessor,
) -> PyResult<()> {
    use sakurs_core::Input;

    let mut state_guard = state.lock().unwrap();

    // Append new text to buffer
    state_guard.text_buffer.push_str(text);

    // Process the buffered text
    let input = Input::from_text(&state_guard.text_buffer);
    let output = processor
        .process(input)
        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

    // Extract complete sentences (all but potentially the last one)
    let mut sentences = Vec::new();
    let mut last_boundary = 0;

    for (i, boundary) in output.boundaries.iter().enumerate() {
        let is_last = i == output.boundaries.len() - 1;

        // For streaming, we keep the last sentence in the buffer
        // unless we're sure it's complete (e.g., followed by significant whitespace)
        if !is_last || text.ends_with('\n') || text.ends_with("\n\n") {
            let sentence = state_guard.text_buffer[last_boundary..boundary.offset].to_string();
            let sentence = if state_guard.preserve_whitespace {
                sentence
            } else {
                sentence.trim().to_string()
            };
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            last_boundary = boundary.offset;
        }
    }

    // Update the text buffer to contain only unprocessed text
    if last_boundary > 0 {
        state_guard.text_buffer = state_guard.text_buffer[last_boundary..].to_string();
    }

    // Add sentences to the iterator's buffer
    state_guard.sentence_buffer.extend(sentences);

    Ok(())
}

/// Flush any remaining text in the buffer as a final sentence
pub(crate) fn flush_buffer(
    state: &Arc<Mutex<IteratorState>>,
    processor: &sakurs_core::SentenceProcessor,
) -> PyResult<()> {
    let mut state_guard = state.lock().unwrap();

    if !state_guard.text_buffer.is_empty() {
        // Process any remaining text
        let input = Input::from_text(&state_guard.text_buffer);
        let output = processor
            .process(input)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

        // Extract all sentences from the final buffer
        let mut last_boundary = 0;
        for boundary in output.boundaries {
            let sentence = state_guard.text_buffer[last_boundary..boundary.offset].to_string();
            let sentence = if state_guard.preserve_whitespace {
                sentence
            } else {
                sentence.trim().to_string()
            };
            if !sentence.is_empty() {
                state_guard.sentence_buffer.push_back(sentence);
            }
            last_boundary = boundary.offset;
        }

        // Add any remaining text as the last sentence
        if last_boundary < state_guard.text_buffer.len() {
            let sentence = state_guard.text_buffer[last_boundary..].to_string();
            let sentence = if state_guard.preserve_whitespace {
                sentence
            } else {
                sentence.trim().to_string()
            };
            if !sentence.is_empty() {
                state_guard.sentence_buffer.push_back(sentence);
            }
        }

        state_guard.text_buffer.clear();
    }

    state_guard.exhausted = true;
    Ok(())
}
