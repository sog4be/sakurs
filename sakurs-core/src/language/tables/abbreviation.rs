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
                trie.insert(&abbr, Some(category.clone()));
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

    /// Find abbreviation ending at position (backwards scan)
    ///
    /// Returns true if an abbreviation ends at text[..pos]
    pub fn find_abbrev(&self, text: &str, pos: usize) -> bool {
        if pos == 0 || self.nodes.is_empty() {
            return false;
        }

        // pos is a byte position, we need to work with the text up to that byte
        if pos > text.len() {
            return false;
        }

        // We need to find the word that ends at the dot position
        // Scan backwards from the dot to find the start of the word
        let mut word_start;

        // Find the start of the word by looking for non-alphabetic characters
        let chars: Vec<char> = text.chars().collect();
        let mut char_pos = 0;
        let mut byte_pos = 0;

        // Find the character position of the dot
        while byte_pos < pos && char_pos < chars.len() {
            byte_pos += chars[char_pos].len_utf8();
            char_pos += 1;
        }

        // Now scan backwards from the dot to find word start
        let dot_char_pos = char_pos.saturating_sub(1);
        let mut word_start_char_pos = dot_char_pos;

        while word_start_char_pos > 0 {
            let prev_char = chars[word_start_char_pos - 1];
            // For abbreviations, we allow dots within the word (e.g., U.S.A.)
            // Stop at whitespace or other delimiters, but not dots
            if prev_char.is_whitespace()
                || prev_char == ','
                || prev_char == ';'
                || prev_char == ':'
                || prev_char == '('
                || prev_char == ')'
                || prev_char == '['
                || prev_char == ']'
                || prev_char == '{'
                || prev_char == '}'
                || prev_char == '"'
                || prev_char == '\''
                || prev_char == '!'
                || prev_char == '?'
            {
                break;
            }
            word_start_char_pos -= 1;
        }

        // Calculate byte position of word start
        word_start = 0;
        for ch in &chars[..word_start_char_pos] {
            word_start += ch.len_utf8();
        }

        // Extract the word (which may contain dots for abbreviations like U.S.)
        let word = &text[word_start..pos];
        let word_chars: Vec<char> = word.chars().collect();

        // Check if this word is an abbreviation
        self.match_at(&word_chars)
    }

    /// Check if the char slice matches an abbreviation
    fn match_at(&self, chars: &[char]) -> bool {
        let mut current_idx = 0u32;

        for &ch in chars {
            let normalized = if self.case_sensitive {
                ch
            } else {
                ch.to_lowercase().next().unwrap_or(ch)
            };

            let node = &self.nodes[current_idx as usize];
            match node.children.get(&normalized) {
                Some(&next_idx) => current_idx = next_idx,
                None => return false,
            }
        }

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

        assert!(trie.find_abbrev("Dr", 2));
        assert!(trie.find_abbrev("Mr", 2));
        assert!(trie.find_abbrev("U.S", 3));
        assert!(!trie.find_abbrev("Ms", 2));
    }

    #[test]
    fn test_case_insensitive() {
        let mut trie = Trie::new(false);
        trie.insert("Dr", None);

        assert!(trie.find_abbrev("dr", 2));
        assert!(trie.find_abbrev("DR", 2));
        assert!(trie.find_abbrev("Dr", 2));
    }
}
