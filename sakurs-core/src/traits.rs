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
    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool;

    /// Get the enclosure pair ID for a character (if any)
    ///
    /// Returns `(pair_id, is_opening)` for enclosure characters
    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)>;

    /// Check if a character is a sentence terminator
    fn is_terminator(&self, ch: char) -> bool {
        matches!(ch, '.' | '!' | '?')
    }

    /// Maximum number of enclosure pairs this language uses
    fn max_enclosure_pairs(&self) -> usize {
        8 // Default reasonable maximum
    }
}
