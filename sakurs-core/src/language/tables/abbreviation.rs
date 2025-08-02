//! Abbreviation trie for fast backward scanning
//!
//! This module implements a compact trie for abbreviation matching.
//! Optimized for cache-friendly access and zero allocations during lookup.

use std::collections::HashMap;

/// Compact trie node using array indices instead of pointers
#[derive(Debug, Clone)]
struct TrieNode {
    /// Child nodes: char -> node index
    children: HashMap<char, u32>,
    /// Whether this node marks end of abbreviation
    is_end: bool,
    /// Abbreviation category (if end node)
    category: Option<String>,
}

/// High-performance abbreviation trie
///
/// Memory layout optimized for cache locality:
/// - Nodes stored in contiguous array
/// - Small nodes (typically 8-16 bytes each)
/// - Case-insensitive by default
#[derive(Debug, Clone)]
pub struct Trie {
    /// All nodes in contiguous storage
    nodes: Vec<TrieNode>,
    /// Case sensitivity flag
    case_sensitive: bool,
}

impl Trie {
    /// Create empty trie
    pub fn new(case_sensitive: bool) -> Self {
        Self {
            nodes: vec![TrieNode {
                children: HashMap::new(),
                is_end: false,
                category: None,
            }],
            case_sensitive,
        }
    }

    /// Build from configuration categories
    pub fn from_categories(categories: HashMap<String, Vec<String>>, case_sensitive: bool) -> Self {
        let mut trie = Self::new(case_sensitive);

        for (category, abbreviations) in categories {
            for abbr in abbreviations {
                // Insert the full abbreviation
                trie.insert(&abbr, Some(category.clone()));

                // For multi-period abbreviations like "U.S.A", also insert prefixes
                // This allows detection at intermediate dots
                if abbr.contains('.') {
                    let parts: Vec<&str> = abbr.split('.').collect();
                    if parts.len() > 1 {
                        // Build and insert each prefix
                        for i in 1..parts.len() {
                            let prefix = parts[..i].join(".");
                            if !prefix.is_empty() {
                                trie.insert(&prefix, Some(format!("{category}_prefix")));
                            }
                        }
                    }
                }
            }
        }

        trie
    }

    /// Insert abbreviation into trie
    pub fn insert(&mut self, abbreviation: &str, category: Option<String>) {
        let normalized = if self.case_sensitive {
            abbreviation.to_string()
        } else {
            abbreviation.to_lowercase()
        };

        let mut current_idx = 0u32;

        for ch in normalized.chars() {
            let node = &self.nodes[current_idx as usize];
            let next_idx = if let Some(&child_idx) = node.children.get(&ch) {
                child_idx
            } else {
                // Create new node
                let new_idx = self.nodes.len() as u32;
                self.nodes.push(TrieNode {
                    children: HashMap::new(),
                    is_end: false,
                    category: None,
                });

                // Update parent
                self.nodes[current_idx as usize]
                    .children
                    .insert(ch, new_idx);
                new_idx
            };

            current_idx = next_idx;
        }

        // Mark end node
        let node = &mut self.nodes[current_idx as usize];
        node.is_end = true;
        node.category = category;
    }

    /// Find abbreviation ending at position (backwards scan) - O(n) SLOW VERSION
    ///
    /// Returns true if an abbreviation ends at text[..pos]
    /// WARNING: This method has O(n) complexity and should not be used in hot paths
    pub fn find_abbrev(&self, text: &str, pos: usize) -> bool {
        if pos == 0 || self.nodes.is_empty() {
            return false;
        }

        // pos is a byte position, we need to work with the text up to that byte
        if pos > text.len() {
            return false;
        }

        // Check if the character before pos is a dot
        let before_pos = if pos > 0 {
            &text[..pos]
        } else {
            return false;
        };
        if !before_pos.ends_with('.') {
            return false;
        }

        // Find the start of the abbreviation by scanning backwards efficiently
        let dot_pos = pos - 1; // Position of the dot itself

        // Scan backwards from the dot to find word boundary
        let before_dot = &text[..dot_pos];
        let mut word_start = dot_pos;

        // Scan backwards character by character
        let chars_iter = before_dot.chars().rev();
        let mut current_pos = dot_pos;

        for ch in chars_iter {
            // Stop at delimiters but allow dots and letters
            if ch.is_whitespace()
                || ch == ','
                || ch == ';'
                || ch == ':'
                || ch == '('
                || ch == ')'
                || ch == '['
                || ch == ']'
                || ch == '{'
                || ch == '}'
                || ch == '"'
                || ch == '\''
                || ch == '!'
                || ch == '?'
            {
                break;
            }

            // Move word_start back by this character's byte length
            current_pos -= ch.len_utf8();
            word_start = current_pos;
        }

        // Extract the word up to the current dot (excluding the dot)
        let word = &text[word_start..dot_pos];

        // Check if this word is an abbreviation using string matching
        self.match_word(word)
    }

    /// Efficient O(1) abbreviation check using character window
    ///
    /// This method uses the extended character window to detect abbreviations
    /// up to 5 characters long without expensive text scanning
    pub fn find_abbrev_efficient(&self, window: &crate::character_window::CharacterWindow) -> bool {
        if self.nodes.is_empty() {
            return false;
        }

        // Check if we're at a dot
        if window.current_char() != Some('.') {
            return false;
        }

        // Build the word backwards from the dot position
        let mut word = String::new();
        let mut found_word_start = false;

        // Look back up to 5 characters
        for i in 1..=5 {
            if let Some(ch) = window.prev_char_at(i) {
                if ch.is_alphabetic() {
                    // Prepend to build word in correct order
                    word.insert(0, ch);
                } else if ch == '.' {
                    // Multi-period abbreviation (e.g., "U.S.A.")
                    // Check if what we have so far is an abbreviation
                    if !word.is_empty() && self.match_word(&word) {
                        return true;
                    }
                    // Continue looking for more parts
                    word.clear();
                } else {
                    // Non-alphabetic, non-period character - word boundary
                    found_word_start = true;
                    break;
                }
            } else {
                // Beginning of text
                found_word_start = true;
                break;
            }
        }

        // Check if we found a complete word (hit a boundary)
        if found_word_start && !word.is_empty() {
            return self.match_word(&word);
        }

        // Special case: check if we're in the middle of a longer abbreviation
        // by looking at the 5th character back
        if !found_word_start && word.len() == 5 {
            // We've collected 5 alphabetic characters and haven't hit a boundary
            // This might be a longer abbreviation, so check what we have
            return self.match_word(&word);
        }

        // Handle single-letter abbreviations
        if word.len() == 1 {
            return self.match_word(&word);
        }

        // Handle common 2-4 letter abbreviations
        if word.len() >= 2 && word.len() <= 4 {
            return self.match_word(&word);
        }

        false
    }

    /// Check if the word matches an abbreviation (efficient string version)
    fn match_word(&self, word: &str) -> bool {
        let mut current_idx = 0u32;

        for ch in word.chars() {
            let normalized = if self.case_sensitive {
                ch
            } else {
                ch.to_lowercase().next().unwrap_or(ch)
            };

            let node = &self.nodes[current_idx as usize];

            if let Some(&next_idx) = node.children.get(&normalized) {
                current_idx = next_idx;
            } else {
                return false; // No transition for this character
            }
        }

        // Check if we're at an end node
        self.nodes[current_idx as usize].is_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_abbreviations() {
        let mut trie = Trie::new(false);
        trie.insert("Dr", None);
        trie.insert("Mr", None);
        trie.insert("U.S", None);

        // Test single-word abbreviations (pos is after the dot)
        assert!(trie.find_abbrev("Dr.", 3));
        assert!(trie.find_abbrev("Mr.", 3));
        assert!(trie.find_abbrev("Hello Dr.", 9));
        assert!(!trie.find_abbrev("Ms.", 3));
    }

    #[test]
    fn test_multi_period_abbreviations() {
        let mut categories = HashMap::new();
        categories.insert(
            "locations".to_string(),
            vec!["U.S".to_string(), "U.K".to_string(), "U.S.A".to_string()],
        );

        let trie = Trie::from_categories(categories, false);

        // Test "U.S." - should detect at both dot positions
        assert!(trie.find_abbrev("U.", 2), "Should detect 'U' at first dot");
        assert!(
            trie.find_abbrev("U.S.", 4),
            "Should detect 'U.S' at second dot"
        );

        // Test "U.S.A." - should detect at all three dot positions
        assert!(trie.find_abbrev("U.", 2), "Should detect 'U' at first dot");
        assert!(
            trie.find_abbrev("U.S.", 4),
            "Should detect 'U.S' at second dot"
        );
        assert!(
            trie.find_abbrev("U.S.A.", 6),
            "Should detect 'U.S.A' at third dot"
        );

        // Test in context
        assert!(
            trie.find_abbrev("from the U.S.", 13),
            "Should detect in context"
        );
        assert!(
            trie.find_abbrev("from the U.S.A.", 15),
            "Should detect U.S.A in context"
        );
    }

    #[test]
    fn test_case_insensitive() {
        let mut trie = Trie::new(false);
        trie.insert("Dr", None);

        assert!(trie.find_abbrev("dr.", 3));
        assert!(trie.find_abbrev("DR.", 3));
        assert!(trie.find_abbrev("Dr.", 3));
    }
}
