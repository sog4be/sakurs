use std::collections::HashMap;

/// Trie node for efficient abbreviation lookup
#[derive(Debug, Clone, Default)]
struct TrieNode {
    /// Child nodes indexed by character
    children: HashMap<char, TrieNode>,
    /// Whether this node represents the end of an abbreviation
    is_end: bool,
    /// Category of the abbreviation (if this is an end node)
    category: Option<String>,
}

/// Trie structure for fast abbreviation matching
#[derive(Debug, Clone)]
pub struct AbbreviationTrie {
    root: TrieNode,
    /// Case sensitivity flag
    case_sensitive: bool,
}

/// Result of abbreviation lookup
#[derive(Debug, Clone, PartialEq)]
pub struct AbbreviationMatch {
    /// The matched abbreviation
    pub abbreviation: String,
    /// Length of the match (in characters)
    pub length: usize,
    /// Category of the abbreviation
    pub category: Option<String>,
}

impl AbbreviationTrie {
    /// Create a new abbreviation trie
    pub fn new(case_sensitive: bool) -> Self {
        Self {
            root: TrieNode::default(),
            case_sensitive,
        }
    }

    /// Check if the trie is empty (contains no abbreviations)
    pub fn is_empty(&self) -> bool {
        self.root.children.is_empty()
    }

    /// Insert an abbreviation into the trie
    pub fn insert(&mut self, abbreviation: &str, category: Option<String>) {
        let normalized = if self.case_sensitive {
            abbreviation.to_string()
        } else {
            abbreviation.to_lowercase()
        };

        let mut current = &mut self.root;

        for ch in normalized.chars() {
            current = current.children.entry(ch).or_default();
        }

        current.is_end = true;
        current.category = category;
    }

    /// Insert multiple abbreviations from a category
    pub fn insert_category(&mut self, category: &str, abbreviations: &[String]) {
        for abbr in abbreviations {
            self.insert(abbr, Some(category.to_string()));
        }
    }

    /// Build from configuration categories
    pub fn from_categories(categories: HashMap<String, Vec<String>>, case_sensitive: bool) -> Self {
        let mut trie = Self::new(case_sensitive);

        for (category, abbreviations) in categories {
            trie.insert_category(&category, &abbreviations);
        }

        trie
    }

    /// Find the longest abbreviation ending at the given position
    pub fn find_at_position(&self, text: &str, position: usize) -> Option<AbbreviationMatch> {
        // Early exit if trie is empty
        if self.is_empty() {
            return None;
        }

        if position >= text.len() {
            return None;
        }

        // We need to search backwards from the position
        let text_chars: Vec<char> = text.chars().collect();
        let mut matches = Vec::new();

        // Try different starting positions to find all possible matches
        for start_offset in 0..=position.min(20) {
            // Limit search to reasonable abbreviation length
            if let Some(start_pos) = position.checked_sub(start_offset) {
                if let Some(abbr_match) =
                    self.match_from_position(&text_chars, start_pos, position + 1)
                {
                    matches.push(abbr_match);
                }
            }
        }

        // Return the longest match
        matches.into_iter().max_by_key(|m| m.length)
    }

    /// Match an abbreviation starting from a specific position
    fn match_from_position(
        &self,
        chars: &[char],
        start: usize,
        end: usize,
    ) -> Option<AbbreviationMatch> {
        if start >= end || end > chars.len() {
            return None;
        }

        let mut current = &self.root;
        let mut matched_text = String::new();

        for &original_ch in &chars[start..end] {
            let ch = if self.case_sensitive {
                original_ch
            } else {
                original_ch.to_lowercase().next()?
            };

            matched_text.push(original_ch);

            current = current.children.get(&ch)?;
        }

        if current.is_end {
            Some(AbbreviationMatch {
                abbreviation: matched_text,
                length: end - start,
                category: current.category.clone(),
            })
        } else {
            None
        }
    }

    /// Check if text starting at position matches any abbreviation
    pub fn is_abbreviation(&self, text: &str, start: usize) -> bool {
        if start >= text.len() {
            return false;
        }

        let chars: Vec<char> = text[start..].chars().collect();
        let mut current = &self.root;

        for ch in chars {
            let normalized_ch = if self.case_sensitive {
                ch
            } else {
                ch.to_lowercase().next().unwrap_or(ch)
            };

            match current.children.get(&normalized_ch) {
                Some(node) => {
                    current = node;
                    if current.is_end {
                        return true;
                    }
                }
                None => return false,
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbreviation_insertion_and_lookup() {
        let mut trie = AbbreviationTrie::new(false);

        trie.insert("Dr", Some("title".to_string()));
        trie.insert("Mr", Some("title".to_string()));
        trie.insert("Inc", Some("business".to_string()));

        // Test finding abbreviations
        let text = "Dr. Smith";
        let match_result = trie.find_at_position(text, 1); // Position after 'r' in "Dr"

        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.abbreviation, "Dr");
        assert_eq!(m.length, 2);
        assert_eq!(m.category, Some("title".to_string()));
    }

    #[test]
    fn test_case_insensitive_matching() {
        let mut trie = AbbreviationTrie::new(false);
        trie.insert("ph.d", Some("academic".to_string()));

        // Should match "Ph.D" even though we inserted "ph.d"
        let text = "Ph.D student";
        let match_result = trie.find_at_position(text, 3); // After 'D'

        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.abbreviation, "Ph.D");
        assert_eq!(m.category, Some("academic".to_string()));
    }

    #[test]
    fn test_from_categories() {
        let mut categories = HashMap::new();
        categories.insert(
            "titles".to_string(),
            vec!["Dr".to_string(), "Mr".to_string()],
        );
        categories.insert(
            "business".to_string(),
            vec!["Inc".to_string(), "Corp".to_string()],
        );

        let trie = AbbreviationTrie::from_categories(categories, false);

        assert!(trie.is_abbreviation("Dr", 0));
        assert!(trie.is_abbreviation("Corp", 0));
        assert!(!trie.is_abbreviation("Hello", 0));
    }

    #[test]
    fn test_longest_match() {
        let mut trie = AbbreviationTrie::new(false);
        trie.insert("U", Some("country".to_string()));
        trie.insert("U.S", Some("country".to_string()));
        trie.insert("U.S.A", Some("country".to_string()));

        let text = "U.S.A.";
        let match_result = trie.find_at_position(text, 4); // After the second 'A'

        assert!(match_result.is_some());
        let m = match_result.unwrap();
        assert_eq!(m.abbreviation, "U.S.A");
        assert_eq!(m.length, 5);
    }

    #[test]
    fn test_no_match() {
        let trie = AbbreviationTrie::new(false);

        let text = "Hello world";
        let match_result = trie.find_at_position(text, 4);

        assert!(match_result.is_none());
    }

    #[test]
    fn test_empty_trie() {
        let trie = AbbreviationTrie::new(false);

        // Verify trie is empty
        assert!(trie.is_empty());

        // Verify find_at_position returns None immediately for empty trie
        let text = "This is a test. With some text.";
        assert_eq!(trie.find_at_position(text, 14), None); // Position of first period
        assert_eq!(trie.find_at_position(text, 30), None); // Position of second period
    }

    #[test]
    fn test_is_empty() {
        let mut trie = AbbreviationTrie::new(false);

        // Initially empty
        assert!(trie.is_empty());

        // Not empty after insertion
        trie.insert("Dr", None);
        assert!(!trie.is_empty());

        // Still not empty with more insertions
        trie.insert("Mr", None);
        trie.insert("Mrs", None);
        assert!(!trie.is_empty());
    }
}
