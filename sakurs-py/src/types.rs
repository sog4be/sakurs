//! Python type definitions and conversions

#![allow(non_local_definitions)]

use sakurs_core::ProcessingStats;

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
            // boundary_offset points to the position AFTER the sentence-ending punctuation
            // So we use it directly as the end position
            let end = boundary_offset.min(text.len());

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
