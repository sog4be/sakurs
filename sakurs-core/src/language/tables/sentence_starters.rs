//! Sentence starters detection for boundary decision after abbreviations
//!
//! Sentence starters are words commonly found at the beginning of sentences.
//! When an abbreviation is followed by a sentence starter, it indicates a
//! sentence boundary.

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::collections::{HashMap, HashSet};
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};

/// Sentence starters lookup table
#[derive(Debug, Clone)]
pub struct SentenceStarterTable {
    /// Set of sentence starters for O(1) lookup
    starters: HashSet<String>,
    /// Minimum word length to consider (optimization)
    min_length: usize,
    /// Maximum word length to consider (optimization)
    max_length: usize,
}

impl SentenceStarterTable {
    /// Create from categorized word lists
    pub fn from_categories(categories: HashMap<String, Vec<String>>) -> Self {
        let mut starters = HashSet::new();
        let mut min_length = usize::MAX;
        let mut max_length = 0;

        // Flatten all categories into a single set
        for (_category, words) in categories {
            for word in words {
                let len = word.len();
                min_length = min_length.min(len);
                max_length = max_length.max(len);
                starters.insert(word);
            }
        }

        // Handle empty case
        if starters.is_empty() {
            min_length = 0;
        }

        Self {
            starters,
            min_length,
            max_length,
        }
    }

    /// Create an empty table (no sentence starters)
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            starters: HashSet::new(),
            min_length: 0,
            max_length: 0,
        }
    }

    /// Check if a word is a sentence starter
    pub fn is_sentence_starter(&self, word: &str) -> bool {
        // Quick length check
        if word.len() < self.min_length || word.len() > self.max_length {
            return false;
        }

        self.starters.contains(word)
    }

    /// Get the next word after a position in text
    /// Returns None if no valid word found
    pub fn get_next_word(&self, text: &str, after_pos: usize) -> Option<String> {
        if after_pos >= text.len() {
            return None;
        }

        let remaining = &text[after_pos..];
        let trimmed = remaining.trim_start();

        if trimmed.is_empty() {
            return None;
        }

        // Find the word boundary
        let word_end = trimmed
            .char_indices()
            .find(|(_, ch)| !ch.is_alphabetic())
            .map(|(i, _)| i)
            .unwrap_or(trimmed.len());

        if word_end == 0 {
            return None;
        }

        Some(trimmed[..word_end].to_string())
    }

    /// Check if position after abbreviation should be a boundary
    /// based on whether it's followed by a sentence starter
    pub fn check_after_abbreviation(&self, text: &str, after_abbrev_pos: usize) -> bool {
        // If we're at end of text, it's a boundary
        if after_abbrev_pos >= text.len() {
            return true;
        }

        let remaining = &text[after_abbrev_pos..];

        // IMPORTANT: For multi-period abbreviations like "U.S.A.",
        // we need to check if there's whitespace before the next word.
        // If the next character is directly adjacent (no space), it's likely
        // part of a multi-period abbreviation, not a sentence starter.
        if !remaining.is_empty() && !remaining.chars().next().unwrap().is_whitespace() {
            // If there's no whitespace immediately after the abbreviation,
            // don't treat the next word as a sentence starter
            return false;
        }

        // Get the next word
        if let Some(next_word) = self.get_next_word(text, after_abbrev_pos) {
            // Check if it's a sentence starter
            self.is_sentence_starter(&next_word)
        } else {
            // No word found after abbreviation
            // This could be:
            // 1. End of text (only whitespace) -> boundary
            // 2. Punctuation followed by more text -> not a boundary

            // Check if there's only whitespace left
            remaining.trim().is_empty()
        }
    }

    /// Check if we have any sentence starters configured
    pub fn is_empty(&self) -> bool {
        self.starters.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_starter_detection() {
        let mut categories = HashMap::new();
        categories.insert(
            "pronouns".to_string(),
            vec!["I".to_string(), "He".to_string(), "She".to_string()],
        );
        categories.insert(
            "conjunctions".to_string(),
            vec!["However".to_string(), "But".to_string()],
        );

        let table = SentenceStarterTable::from_categories(categories);

        assert!(table.is_sentence_starter("I"));
        assert!(table.is_sentence_starter("However"));
        assert!(!table.is_sentence_starter("hello"));
        assert!(!table.is_sentence_starter("i")); // Case sensitive
    }

    #[test]
    fn test_get_next_word() {
        let table = SentenceStarterTable::empty();

        assert_eq!(
            table.get_next_word("Dr. Smith arrived.", 3),
            Some("Smith".to_string())
        );
        assert_eq!(
            table.get_next_word("Dr.  Smith", 3),
            Some("Smith".to_string())
        );
        assert_eq!(table.get_next_word("Dr.", 3), None);
        assert_eq!(table.get_next_word("Dr. ", 4), None);
    }

    #[test]
    fn test_check_after_abbreviation() {
        let mut categories = HashMap::new();
        categories.insert(
            "pronouns".to_string(),
            vec!["He".to_string(), "She".to_string()],
        );

        let table = SentenceStarterTable::from_categories(categories);

        // Followed by sentence starter = boundary
        assert!(table.check_after_abbreviation("Dr. He arrived.", 3));

        // Not followed by sentence starter = no boundary
        assert!(!table.check_after_abbreviation("Dr. Smith arrived.", 3));

        // End of text = boundary
        assert!(table.check_after_abbreviation("Dr.", 3));
    }
}
