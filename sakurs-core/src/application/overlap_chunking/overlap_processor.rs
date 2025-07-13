//! Overlap processing for cross-chunk pattern detection

use super::{pattern_detector::PatternDetector, types::*};
use crate::{
    application::{chunking::TextChunk, config::ProcessingResult},
    domain::enclosure_suppressor::EnclosureSuppressor,
};
use std::sync::Arc;

/// Processes overlap regions between chunks to detect cross-chunk patterns
pub struct OverlapProcessor {
    /// Size of overlap region in bytes (default: 32)
    overlap_size: usize,

    /// Enclosure suppressor for pattern detection
    pub(super) enclosure_suppressor: Arc<dyn EnclosureSuppressor>,
}

impl OverlapProcessor {
    /// Creates a new overlap processor
    pub fn new(overlap_size: usize, enclosure_suppressor: Arc<dyn EnclosureSuppressor>) -> Self {
        Self {
            overlap_size,
            enclosure_suppressor,
        }
    }

    /// Processes the overlap between two adjacent chunks
    pub fn process_overlap(
        &self,
        left_chunk: &TextChunk,
        right_chunk: &TextChunk,
    ) -> ProcessingResult<OverlapResult> {
        // Extract overlap region
        let (overlap_text, left_context, right_context) =
            self.extract_overlap_region(left_chunk, right_chunk)?;

        // Detect suppressions in the overlap region
        let suppressions =
            self.detect_suppressions(&overlap_text, left_chunk.end_offset, left_context.len())?;

        // Detect partial patterns
        let partial_patterns = self.detect_partial_patterns(&left_context, &right_context);

        // Calculate boundary adjustments
        let boundary_adjustments = self.calculate_boundary_adjustments(&suppressions);

        Ok(OverlapResult {
            suppressions,
            boundary_adjustments,
            extended_context: overlap_text,
            partial_patterns,
        })
    }

    /// Extracts the overlap region and surrounding context from two chunks
    fn extract_overlap_region(
        &self,
        left_chunk: &TextChunk,
        right_chunk: &TextChunk,
    ) -> ProcessingResult<(String, String, String)> {
        // For overlap processing, we need to look at the boundary area between chunks
        // Take the end of the left chunk and the beginning of the right chunk

        let left_content = &left_chunk.content;
        let right_content = &right_chunk.content;

        // Find safe UTF-8 boundaries for left context
        let left_end = left_content.len();
        let mut left_start = left_end.saturating_sub(self.overlap_size);

        // Ensure left_start is at a char boundary
        while left_start < left_end && !left_content.is_char_boundary(left_start) {
            left_start += 1;
        }

        let left_context = &left_content[left_start..];

        // Find safe UTF-8 boundaries for right context
        let mut right_end = right_content.len().min(self.overlap_size);

        // Ensure right_end is at a char boundary
        while right_end > 0 && !right_content.is_char_boundary(right_end) {
            right_end -= 1;
        }

        let right_context = &right_content[..right_end];

        // The overlap region is the concatenation of the boundary area
        // This allows us to detect patterns that span the chunk boundary
        let overlap_text = format!("{left_context}{right_context}");

        Ok((
            overlap_text,
            left_context.to_string(),
            right_context.to_string(),
        ))
    }

    /// Detects suppression patterns in the overlap region
    fn detect_suppressions(
        &self,
        extended_context: &str,
        base_offset: usize,
        left_context_len: usize,
    ) -> ProcessingResult<Vec<SuppressionMarker>> {
        let mut suppressions = Vec::new();

        // Iterate through each character in the extended context
        for (idx, ch) in extended_context.char_indices() {
            // Skip non-enclosure characters
            if !PatternDetector::is_potential_enclosure(ch) {
                continue;
            }

            // Create context for this position
            let context = PatternDetector::create_enclosure_context(extended_context, idx, ch);

            // Check if this enclosure should be suppressed
            if self
                .enclosure_suppressor
                .should_suppress_enclosure(ch, &context)
            {
                let reason = PatternDetector::determine_suppression_reason(ch, &context);
                let confidence = PatternDetector::calculate_confidence(&context, &reason);

                // Calculate the actual position in the original text
                // The overlap text consists of:
                // - left_context (from left chunk end)
                // - right_context (from right chunk start)

                let position = if idx < left_context_len {
                    // Character is in the left chunk's overlap region
                    // base_offset is the end of left chunk, so we subtract the distance from end
                    base_offset.saturating_sub(left_context_len - idx)
                } else {
                    // Character is in the right chunk's overlap region
                    // Position relative to the start of right chunk
                    base_offset + (idx - left_context_len)
                };

                suppressions.push(SuppressionMarker {
                    position,
                    character: ch,
                    reason,
                    confidence,
                    from_overlap: true,
                });
            }
        }

        Ok(suppressions)
    }

    /// Detects partial patterns at chunk boundaries
    fn detect_partial_patterns(
        &self,
        left_context: &str,
        right_context: &str,
    ) -> Vec<PartialPattern> {
        let mut patterns = Vec::new();

        // Check for potential contraction starts at left end
        if left_context.ends_with("isn")
            || left_context.ends_with("don")
            || left_context.ends_with("won")
            || left_context.ends_with("can")
        {
            patterns.push(PartialPattern {
                text: left_context
                    .chars()
                    .rev()
                    .take(3)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect(),
                expected_continuations: vec!["'t".to_string()],
                pattern_type: PatternType::Contraction,
            });
        }

        // Check for possessive patterns
        if left_context
            .chars()
            .last()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false)
            && right_context.starts_with('\'')
        {
            patterns.push(PartialPattern {
                text: left_context.chars().last().unwrap().to_string(),
                expected_continuations: vec!["'".to_string(), "'s".to_string()],
                pattern_type: PatternType::Possessive,
            });
        }

        patterns
    }

    /// Calculates boundary adjustments based on suppressions
    fn calculate_boundary_adjustments(
        &self,
        suppressions: &[SuppressionMarker],
    ) -> Vec<BoundaryAdjustment> {
        let mut adjustments = Vec::new();

        // For each suppression, check if there's a boundary nearby that should be adjusted
        for suppression in suppressions {
            // If this is a sentence-ending punctuation that was suppressed,
            // we might need to adjust a nearby boundary
            if matches!(suppression.character, '.' | '!' | '?') {
                adjustments.push(BoundaryAdjustment {
                    original_position: suppression.position,
                    adjusted_position: None, // Remove the boundary
                    reason: format!("Suppressed due to {:?}", suppression.reason),
                });
            }
        }

        adjustments
    }
}
