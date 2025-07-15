//! Enclosure suppression logic for handling special punctuation patterns
//!
//! This module provides language-aware suppression of enclosure tracking for
//! patterns like contractions, inch marks, and list items that should not be
//! treated as opening/closing quotes or parentheses.

use smallvec::SmallVec;

/// Context information for enclosure suppression decisions
#[derive(Debug, Clone)]
pub struct EnclosureContext<'a> {
    /// Current position in the text (byte offset)
    pub position: usize,
    /// Characters preceding the current character (up to 3)
    pub preceding_chars: SmallVec<[char; 3]>,
    /// Characters following the current character (up to 3)
    pub following_chars: SmallVec<[char; 3]>,
    /// Offset from the beginning of the current line
    pub line_offset: usize,
    /// The full text of the current chunk (for complex pattern matching)
    pub chunk_text: &'a str,
}

/// Trait for determining when to suppress enclosure tracking
pub trait EnclosureSuppressor: Send + Sync {
    /// Determines if an enclosure character should be suppressed (not tracked)
    ///
    /// Returns true if the character should not affect enclosure depth tracking
    fn should_suppress_enclosure(&self, ch: char, context: &EnclosureContext) -> bool;

    /// Returns a description of this suppressor for debugging
    fn description(&self) -> &str {
        "Generic enclosure suppressor"
    }
}

/// English-specific enclosure suppression rules
#[derive(Debug, Clone, Default)]
pub struct EnglishEnclosureSuppressor;

impl EnglishEnclosureSuppressor {
    /// Creates a new English enclosure suppressor
    pub fn new() -> Self {
        Self
    }

    /// Checks if an apostrophe is part of a contraction
    fn is_contraction_apostrophe(&self, context: &EnclosureContext) -> bool {
        // Check if both sides have alphabetic characters
        let prev_is_alpha = context
            .preceding_chars
            .last()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false);

        let next_is_alpha = context
            .following_chars
            .first()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false);

        // Contractions have letters on both sides
        prev_is_alpha && next_is_alpha
    }

    /// Checks if an apostrophe is at the end of a word (possessive or plural)
    fn is_possessive_or_plural(&self, context: &EnclosureContext) -> bool {
        // Check for patterns like "90s'" or "James'"
        let prev_is_alnum = context
            .preceding_chars
            .last()
            .map(|c| c.is_alphanumeric())
            .unwrap_or(false);

        let next_is_space_or_punct = context
            .following_chars
            .first()
            .map(|c| c.is_whitespace() || c.is_ascii_punctuation())
            .unwrap_or(true); // End of text counts as space

        prev_is_alnum && next_is_space_or_punct
    }

    /// Checks if a quote is an inch/foot mark after numbers
    fn is_measurement_mark(&self, _ch: char, context: &EnclosureContext) -> bool {
        // Check for patterns like 5'9" or 45°30'
        let prev_char = context.preceding_chars.last();

        if let Some(prev) = prev_char {
            // Direct number before quote
            if prev.is_numeric() {
                return true;
            }

            // Degree symbol before quote (for minutes/seconds)
            if *prev == '°' || *prev == '\'' {
                return true;
            }

            // Check for space after number (e.g., "5 '")
            if prev.is_whitespace() && context.preceding_chars.len() >= 2 {
                if let Some(prev_prev) = context
                    .preceding_chars
                    .get(context.preceding_chars.len() - 2)
                {
                    if prev_prev.is_numeric() {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Checks if a closing parenthesis is part of a list item
    fn is_list_item_paren(&self, context: &EnclosureContext) -> bool {
        // Only check if we're near the beginning of a line
        if context.line_offset > 10 {
            return false;
        }

        // Look for alphanumeric characters before the paren
        let mut found_alnum = false;
        let mut found_only_space_before_alnum = true;

        for ch in context.preceding_chars.iter().rev() {
            if ch.is_alphanumeric() {
                found_alnum = true;
            } else if !ch.is_whitespace() && ch != &'.' && !found_alnum {
                // Found non-space, non-dot, non-alnum before any alnum
                found_only_space_before_alnum = false;
                break;
            }
        }

        // Pattern: whitespace* + alphanumeric+ + optional '.' + ')'
        found_alnum && found_only_space_before_alnum
    }

    /// Checks if an apostrophe is part of a year abbreviation (e.g., '90s, '60s)
    fn is_year_abbreviation(&self, context: &EnclosureContext) -> bool {
        // Check if preceded by whitespace or start of text
        let prev_is_space_or_start = context
            .preceding_chars
            .last()
            .map(|c| c.is_whitespace())
            .unwrap_or(true);

        // Check if followed by exactly 2 digits and 's'
        if context.following_chars.len() >= 3 {
            let follows_year_pattern = context.following_chars[0].is_ascii_digit()
                && context.following_chars[1].is_ascii_digit()
                && context.following_chars[2] == 's';

            prev_is_space_or_start && follows_year_pattern
        } else {
            false
        }
    }
}

impl EnclosureSuppressor for EnglishEnclosureSuppressor {
    fn should_suppress_enclosure(&self, ch: char, context: &EnclosureContext) -> bool {
        match ch {
            // Apostrophes and right single quotation mark
            '\'' | '\u{2019}' => {
                self.is_contraction_apostrophe(context)
                    || self.is_possessive_or_plural(context)
                    || self.is_measurement_mark(ch, context)
                    || self.is_year_abbreviation(context)
            }

            // Double quotes that might be inch marks
            '"' => self.is_measurement_mark(ch, context),

            // Closing parenthesis in list items
            ')' => self.is_list_item_paren(context),

            // TODO: Add backtick suppression for code blocks
            // TODO: Add phone number parenthesis suppression
            _ => false,
        }
    }

    fn description(&self) -> &str {
        "English enclosure suppressor (contractions, measurements, list items)"
    }
}

/// No-op suppressor that never suppresses any enclosures
#[derive(Debug, Clone, Default)]
pub struct NoOpEnclosureSuppressor;

impl EnclosureSuppressor for NoOpEnclosureSuppressor {
    fn should_suppress_enclosure(&self, _ch: char, _context: &EnclosureContext) -> bool {
        false
    }

    fn description(&self) -> &str {
        "No-op enclosure suppressor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_context<'a>(
        preceding: &str,
        following: &str,
        line_offset: usize,
        chunk_text: &'a str,
    ) -> EnclosureContext<'a> {
        let preceding_chars: SmallVec<[char; 3]> = preceding
            .chars()
            .rev()
            .take(3)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        let following_chars: SmallVec<[char; 3]> = following.chars().take(3).collect();

        EnclosureContext {
            position: 0,
            preceding_chars,
            following_chars,
            line_offset,
            chunk_text,
        }
    }

    #[test]
    fn test_contraction_detection() {
        let suppressor = EnglishEnclosureSuppressor::new();

        // Test contractions
        let context = create_context("isn", "t", 10, "isn't");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("don", "t", 10, "don't");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("I", "m", 5, "I'm");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        // Test non-contractions
        let context = create_context("", "Hello", 0, "'Hello");
        assert!(!suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("said ", "", 10, "said '");
        assert!(!suppressor.should_suppress_enclosure('\'', &context));
    }

    #[test]
    fn test_possessive_detection() {
        let suppressor = EnglishEnclosureSuppressor::new();

        // Test possessives
        let context = create_context("James", " ", 10, "James' ");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("90s", ".", 10, "90s'.");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        // Test year abbreviations (should NOT be suppressed by this function)
        let context = create_context("", "90s", 0, "'90s");
        assert!(!suppressor.is_possessive_or_plural(&context));
    }

    #[test]
    fn test_year_abbreviations() {
        let suppressor = EnglishEnclosureSuppressor::new();

        // Test year abbreviations that should be suppressed
        let context = create_context(" ", "90s", 10, " '90s");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("", "60s", 0, "'60s");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("\t", "20s", 5, "\t'20s");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        // Test Unicode apostrophe
        let context = create_context(" ", "90s", 10, " '90s");
        assert!(suppressor.should_suppress_enclosure('\u{2019}', &context));

        // Test cases that should NOT be suppressed
        let context = create_context("a", "90s", 10, "a'90s");
        assert!(!suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context(" ", "9s", 10, " '9s");
        assert!(!suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context(" ", "90a", 10, " '90a");
        assert!(!suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context(" ", "900s", 10, " '900s");
        assert!(!suppressor.should_suppress_enclosure('\'', &context));
    }

    #[test]
    fn test_measurement_marks() {
        let suppressor = EnglishEnclosureSuppressor::new();

        // Test inch marks
        let context = create_context("5", "9", 10, "5'9");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("9", "", 10, "5'9\"");
        assert!(suppressor.should_suppress_enclosure('"', &context));

        // Test degree/minute notation
        let context = create_context("45°", "30", 10, "45°'30");
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        let context = create_context("30'", "", 10, "45°30'\"");
        assert!(suppressor.should_suppress_enclosure('"', &context));
    }

    #[test]
    fn test_list_item_parentheses() {
        let suppressor = EnglishEnclosureSuppressor::new();

        // Test list items at line start
        let context = create_context("1", " ", 1, "1) ");
        assert!(suppressor.should_suppress_enclosure(')', &context));

        let context = create_context("  a", " ", 3, "  a) ");
        assert!(suppressor.should_suppress_enclosure(')', &context));

        let context = create_context("1.", " ", 2, "1.) ");
        assert!(suppressor.should_suppress_enclosure(')', &context));

        // Test non-list parentheses
        let context = create_context("(text", " ", 20, "(text) ");
        assert!(!suppressor.should_suppress_enclosure(')', &context));

        // Test far from line start
        let context = create_context("1", " ", 20, "some text 1) ");
        assert!(!suppressor.should_suppress_enclosure(')', &context));
    }

    #[test]
    fn test_unicode_apostrophes() {
        let suppressor = EnglishEnclosureSuppressor::new();

        // Test Unicode right single quotation mark in contractions
        let context = create_context("isn", "t", 10, "isn't");
        assert!(suppressor.should_suppress_enclosure('\u{2019}', &context));

        let context = create_context("I", "m", 5, "I'm");
        assert!(suppressor.should_suppress_enclosure('\u{2019}', &context));
    }
}
