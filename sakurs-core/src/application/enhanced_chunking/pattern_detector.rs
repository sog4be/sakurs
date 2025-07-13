//! Shared pattern detection utilities for cross-chunk processing

use super::{
    constants::{confidence, CONTEXT_CHAR_COUNT, MAX_LIST_ITEM_LINE_OFFSET},
    types::*,
};
use crate::domain::enclosure_suppressor::EnclosureContext;
use smallvec::SmallVec;

/// Common pattern detection logic used by both EnhancedChunkManager and OverlapProcessor
pub struct PatternDetector;

impl PatternDetector {
    /// Determines the suppression reason based on character and context
    pub fn determine_suppression_reason(ch: char, context: &EnclosureContext) -> SuppressionReason {
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
    pub fn calculate_confidence(context: &EnclosureContext, reason: &SuppressionReason) -> f32 {
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
    pub fn create_enclosure_context<'a>(
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
    pub fn is_potential_enclosure(ch: char) -> bool {
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
