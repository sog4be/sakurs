//! Boundary detection for streaming processing

use crate::domain::language::{BoundaryContext, BoundaryDecision, LanguageRules};
use std::sync::Arc;

/// Detector for finding sentence boundaries in streaming chunks
pub struct BoundaryDetector {
    language_rules: Arc<dyn LanguageRules>,
    /// Context from previous chunk for accurate detection
    previous_context: String,
    /// Maximum context size to maintain
    max_context_size: usize,
}

impl BoundaryDetector {
    /// Create a new boundary detector
    pub fn new(language_rules: Arc<dyn LanguageRules>) -> Self {
        Self {
            language_rules,
            previous_context: String::new(),
            max_context_size: 100, // Keep last 100 chars for context
        }
    }

    /// Detect boundaries in a chunk with context from previous chunk
    pub fn detect_boundaries(&mut self, chunk: &str) -> Vec<usize> {
        // Combine previous context with current chunk
        let combined_text = format!("{}{}", self.previous_context, chunk);
        let context_offset = self.previous_context.len();

        let mut boundaries = Vec::new();

        // Process using char indices for accurate byte positions
        for (byte_pos, ch) in combined_text.char_indices() {
            // Check for potential terminators
            if is_potential_terminator(ch) {
                // Build context for language rules
                let context = self.build_boundary_context(&combined_text, byte_pos, ch);

                // Ask language rules for decision
                let decision = self.language_rules.detect_sentence_boundary(&context);

                if let BoundaryDecision::Boundary(_flags) = decision {
                    // Calculate position relative to current chunk
                    if byte_pos >= context_offset {
                        let chunk_pos = byte_pos - context_offset;
                        // Add position after the terminator
                        let terminator_end = byte_pos + ch.len_utf8();
                        if terminator_end > context_offset {
                            boundaries.push(chunk_pos + ch.len_utf8());
                        }
                    }
                }
            }
        }

        // Update context for next chunk
        self.update_context(chunk);

        boundaries
    }

    /// Build boundary context for language rule evaluation
    fn build_boundary_context(
        &self,
        text: &str,
        position: usize,
        boundary_char: char,
    ) -> BoundaryContext {
        // Extract preceding context (up to 10 chars)
        let mut start = position.saturating_sub(10);
        while start > 0 && !text.is_char_boundary(start) {
            start -= 1;
        }
        let preceding_context = text[start..position].to_string();

        // Extract following context (up to 10 chars)
        let end_pos = position + boundary_char.len_utf8();
        let mut end = (end_pos + 10).min(text.len());
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        let following_context = if end_pos < text.len() {
            text[end_pos..end].to_string()
        } else {
            String::new()
        };

        BoundaryContext {
            text: text.to_string(),
            position,
            boundary_char,
            preceding_context,
            following_context,
        }
    }

    /// Update context with end of current chunk
    fn update_context(&mut self, chunk: &str) {
        // Keep last N characters as context
        if chunk.len() > self.max_context_size {
            let start = chunk.len() - self.max_context_size;
            // Find valid UTF-8 boundary
            let mut valid_start = start;
            while valid_start < chunk.len() && !chunk.is_char_boundary(valid_start) {
                valid_start += 1;
            }
            self.previous_context = chunk[valid_start..].to_string();
        } else {
            // If previous context + chunk is still small, keep it all
            let combined = format!("{}{}", self.previous_context, chunk);
            if combined.len() <= self.max_context_size {
                self.previous_context = combined;
            } else {
                // Trim to max size
                let start = combined.len() - self.max_context_size;
                let mut valid_start = start;
                while valid_start < combined.len() && !combined.is_char_boundary(valid_start) {
                    valid_start += 1;
                }
                self.previous_context = combined[valid_start..].to_string();
            }
        }
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        self.previous_context.clear();
    }
}

/// Check if a character is a potential sentence terminator
fn is_potential_terminator(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | '。' | '！' | '？')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::EnglishLanguageRules;

    #[test]
    fn test_boundary_detection() {
        let rules = Arc::new(EnglishLanguageRules::new());
        let mut detector = BoundaryDetector::new(rules);

        let chunk = "Hello world. This is a test.";
        let boundaries = detector.detect_boundaries(chunk);
        assert_eq!(boundaries.len(), 2);
    }

    #[test]
    fn test_context_preservation() {
        let rules = Arc::new(EnglishLanguageRules::new());
        let mut detector = BoundaryDetector::new(rules);

        // First chunk ends mid-sentence
        let chunk1 = "This is a long sentence that continues";
        let boundaries1 = detector.detect_boundaries(chunk1);
        assert_eq!(boundaries1.len(), 0);

        // Second chunk completes the sentence
        let chunk2 = " into the next chunk. And then another.";
        let boundaries2 = detector.detect_boundaries(chunk2);
        assert_eq!(boundaries2.len(), 2);
    }
}
