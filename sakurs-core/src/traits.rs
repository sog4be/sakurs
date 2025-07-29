//! Core traits for language-specific rules

use crate::types::Class;

/// Language-specific boundary detection rules
///
/// This trait provides a minimal interface for language-specific
/// logic without any I/O or configuration concerns.
pub trait LanguageRules: Send + Sync + 'static {
    /// Classify a character for fast lookup
    fn classify_char(&self, ch: char) -> Class;

    /// Check if character sequence matches an abbreviation
    ///
    /// # Arguments
    /// * `text` - The text buffer to check
    /// * `dot_pos` - Position of the dot character in the text
    ///
    /// # Returns
    /// `true` if the text before the dot is a known abbreviation
    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool;

    /// Check if a specific abbreviation pattern matches
    ///
    /// This is a more granular version of is_abbreviation that can be used
    /// for streaming scenarios where we only have partial text.
    ///
    /// # Arguments
    /// * `abbrev` - The potential abbreviation (without the dot)
    ///
    /// # Returns
    /// `true` if this is a known abbreviation
    fn abbrev_match(&self, abbrev: &str) -> bool {
        // Default implementation delegates to is_abbreviation
        if abbrev.is_empty() {
            return false;
        }
        let with_dot = format!("{abbrev}.");
        self.is_abbreviation(&with_dot, abbrev.len())
    }

    /// Get the enclosure pair ID for a character (if any)
    ///
    /// Returns `(pair_id, is_opening)` for enclosure characters
    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)>;

    /// Get pair ID for a character without direction info
    ///
    /// Used for symmetric enclosures where open/close is context-dependent
    ///
    /// # Returns
    /// Some(pair_id) if this is an enclosure character, None otherwise
    fn pair_id(&self, ch: char) -> Option<u8> {
        self.get_enclosure_pair(ch).map(|(id, _)| id)
    }

    /// Check if a character is a sentence terminator
    fn is_terminator(&self, ch: char) -> bool {
        matches!(ch, '.' | '!' | '?')
    }

    /// Maximum number of enclosure pairs this language uses
    fn max_enclosure_pairs(&self) -> usize {
        8 // Default reasonable maximum
    }
}
