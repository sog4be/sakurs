//! Overlap processing for cross-chunk pattern detection

use super::types::*;
use crate::{
    application::{chunking::TextChunk, config::ProcessingResult},
    domain::enclosure_suppressor::{EnclosureContext, EnclosureSuppressor},
};
use smallvec::SmallVec;
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
            self.detect_suppressions(&overlap_text, &overlap_text, left_chunk.end_offset)?;

        // Detect partial patterns
        let partial_patterns = self.detect_partial_patterns(&left_context, &right_context);

        // Calculate boundary adjustments
        let boundary_adjustments =
            self.calculate_boundary_adjustments(&suppressions, &overlap_text);

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

        let left_end = left_chunk.content.len();
        let left_start = left_end.saturating_sub(self.overlap_size);
        let left_context = &left_chunk.content[left_start..];

        let right_end = right_chunk.content.len().min(self.overlap_size);
        let right_context = &right_chunk.content[..right_end];

        // The overlap region is the concatenation of the boundary area
        // This allows us to detect patterns that span the chunk boundary
        let overlap_text = format!("{}{}", left_context, right_context);

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
        overlap_text: &str,
        base_offset: usize,
    ) -> ProcessingResult<Vec<SuppressionMarker>> {
        let mut suppressions = Vec::new();

        // Iterate through each character in the extended context
        for (idx, ch) in extended_context.char_indices() {
            // Skip non-enclosure characters
            if !self.is_potential_enclosure(ch) {
                continue;
            }

            // Create context for this position
            let context = self.create_enclosure_context(extended_context, idx, ch);

            // Check if this enclosure should be suppressed
            if self
                .enclosure_suppressor
                .should_suppress_enclosure(ch, &context)
            {
                let reason = self.determine_suppression_reason(ch, &context);
                let confidence = self.calculate_confidence(&context, &reason);

                // Calculate the actual position in the original text
                // We need to adjust based on where this character is in the overlap
                let position = if idx < overlap_text.len() / 2 {
                    // In the left chunk part
                    base_offset
                        .saturating_sub(self.overlap_size)
                        .saturating_add(idx)
                } else {
                    // In the right chunk part
                    base_offset
                        .saturating_add(idx)
                        .saturating_sub(overlap_text.len() / 2)
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

    /// Checks if a character could be an enclosure
    fn is_potential_enclosure(&self, ch: char) -> bool {
        matches!(
            ch,
            '\'' | '"'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '\u{2018}'
                | '\u{2019}'
                | '\u{201C}'
                | '\u{201D}'
        )
    }

    /// Creates an enclosure context for a position
    fn create_enclosure_context<'a>(
        &self,
        text: &'a str,
        position: usize,
        _ch: char,
    ) -> EnclosureContext<'a> {
        // Get preceding characters (up to 3)
        let preceding_chars: SmallVec<[char; 3]> = text[..position]
            .chars()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Get following characters (up to 3)
        let following_chars: SmallVec<[char; 3]> = text[position + 1..].chars().take(3).collect();

        // Calculate line offset (simplified - count from last newline)
        let line_offset = text[..position]
            .rfind('\n')
            .map(|pos| position - pos - 1)
            .unwrap_or(position);

        EnclosureContext {
            position,
            preceding_chars,
            following_chars,
            line_offset,
            chunk_text: text,
        }
    }

    /// Determines the reason for suppression based on context
    fn determine_suppression_reason(
        &self,
        ch: char,
        context: &EnclosureContext,
    ) -> SuppressionReason {
        // Check for contractions
        if matches!(ch, '\'' | '\u{2019}') {
            let prev_alpha = context
                .preceding_chars
                .last()
                .map(|c| c.is_alphabetic())
                .unwrap_or(false);
            let next_alpha = context
                .following_chars
                .first()
                .map(|c| c.is_alphabetic())
                .unwrap_or(false);

            if prev_alpha && next_alpha {
                return SuppressionReason::Contraction;
            }

            // Check for possessives
            if prev_alpha && !next_alpha {
                return SuppressionReason::Possessive;
            }

            // Check for measurements
            if context
                .preceding_chars
                .last()
                .map(|c| c.is_numeric())
                .unwrap_or(false)
            {
                return SuppressionReason::Measurement;
            }
        }

        // Check for list items
        if ch == ')' && context.line_offset < 10 {
            return SuppressionReason::ListItem;
        }

        // Default to cross-chunk pattern
        SuppressionReason::CrossChunkPattern {
            pattern: format!("{:?}", context.preceding_chars),
        }
    }

    /// Calculates confidence score for a suppression
    fn calculate_confidence(&self, context: &EnclosureContext, reason: &SuppressionReason) -> f32 {
        match reason {
            SuppressionReason::Contraction => {
                // High confidence if we have clear alphabetic characters on both sides
                if context.preceding_chars.len() >= 2 && !context.following_chars.is_empty() {
                    0.95
                } else {
                    0.8
                }
            }
            SuppressionReason::Possessive => {
                // High confidence for possessives
                0.9
            }
            SuppressionReason::Measurement => {
                // Very high confidence for measurements
                0.95
            }
            SuppressionReason::ListItem => {
                // Moderate confidence for list items
                0.85
            }
            SuppressionReason::CrossChunkPattern { .. } => {
                // Lower confidence for generic patterns
                0.7
            }
        }
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
        _overlap_text: &str,
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
