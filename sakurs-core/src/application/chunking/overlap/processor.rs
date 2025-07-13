//! Overlap processing and pattern detection for cross-chunk boundary handling

use super::types::{
    BoundaryAdjustment, OverlapResult, PartialPattern, PatternType, SuppressionMarker,
    SuppressionReason,
};
use crate::{
    application::{chunking::base::TextChunk, config::ProcessingResult},
    domain::enclosure_suppressor::{EnclosureContext, EnclosureSuppressor},
};
use smallvec::SmallVec;
use std::sync::Arc;

// Constants for pattern detection
/// Number of characters to look ahead/behind for context
pub const CONTEXT_CHAR_COUNT: usize = 3;

/// Maximum line offset for list item detection
pub const MAX_LIST_ITEM_LINE_OFFSET: usize = 10;

/// Confidence scores for different pattern types
pub mod confidence {
    /// High confidence for contractions with full context
    pub const CONTRACTION_HIGH: f32 = 0.95;

    /// Lower confidence for contractions with partial context
    pub const CONTRACTION_LOW: f32 = 0.8;

    /// Confidence for possessive patterns
    pub const POSSESSIVE: f32 = 0.9;

    /// Confidence for measurement patterns
    pub const MEASUREMENT: f32 = 0.95;

    /// Confidence for list item patterns
    pub const LIST_ITEM: f32 = 0.85;

    /// Default confidence for generic patterns
    pub const GENERIC_PATTERN: f32 = 0.7;
}

/// Processes overlap regions between chunks to detect cross-chunk patterns
pub struct OverlapProcessor {
    /// Size of overlap region in bytes (default: 32)
    overlap_size: usize,

    /// Enclosure suppressor for pattern detection
    pub(crate) enclosure_suppressor: Arc<dyn EnclosureSuppressor>,
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

    // Pattern detection methods (previously in PatternDetector)

    /// Determines the suppression reason based on character and context
    pub(crate) fn determine_suppression_reason(
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
        if ch == ')' && context.line_offset < MAX_LIST_ITEM_LINE_OFFSET {
            return SuppressionReason::ListItem;
        }

        // Default to cross-chunk pattern
        SuppressionReason::CrossChunkPattern {
            pattern: format!("{:?}", context.preceding_chars),
        }
    }

    /// Calculates confidence score for a suppression
    pub(crate) fn calculate_confidence(
        &self,
        context: &EnclosureContext,
        reason: &SuppressionReason,
    ) -> f32 {
        match reason {
            SuppressionReason::Contraction => {
                // High confidence if we have clear alphabetic characters on both sides
                if context.preceding_chars.len() >= 2 && !context.following_chars.is_empty() {
                    confidence::CONTRACTION_HIGH
                } else {
                    confidence::CONTRACTION_LOW
                }
            }
            SuppressionReason::Possessive => confidence::POSSESSIVE,
            SuppressionReason::Measurement => confidence::MEASUREMENT,
            SuppressionReason::ListItem => confidence::LIST_ITEM,
            SuppressionReason::CrossChunkPattern { .. } => confidence::GENERIC_PATTERN,
        }
    }

    /// Creates an enclosure context for a position in text
    pub(crate) fn create_enclosure_context<'a>(
        &self,
        text: &'a str,
        position: usize,
        ch: char,
    ) -> EnclosureContext<'a> {
        // Get preceding characters
        let preceding_chars: SmallVec<[char; CONTEXT_CHAR_COUNT]> = text[..position]
            .chars()
            .rev()
            .take(CONTEXT_CHAR_COUNT)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        // Get following characters
        // Use the character's UTF-8 length to safely skip it
        let skip_position = position + ch.len_utf8();
        let following_chars: SmallVec<[char; CONTEXT_CHAR_COUNT]> = if skip_position <= text.len() {
            text[skip_position..]
                .chars()
                .take(CONTEXT_CHAR_COUNT)
                .collect()
        } else {
            SmallVec::new()
        };

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

    /// Checks if a character could be an enclosure
    pub(crate) fn is_potential_enclosure(&self, ch: char) -> bool {
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
}
