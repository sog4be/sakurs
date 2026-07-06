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
    /// Length of the match in bytes
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

    /// Find the longest abbreviation ending at the given byte position.
    ///
    /// `position` is the byte offset of the last character of the candidate
    /// abbreviation (i.e. the byte just before the trailing period). If it
    /// falls inside a multi-byte character it is snapped back to that
    /// character's start. The returned `length` is in bytes.
    pub fn find_at_position(&self, text: &str, position: usize) -> Option<AbbreviationMatch> {
        // Early exit if trie is empty
        if self.is_empty() {
            return None;
        }

        if position >= text.len() {
            return None;
        }

        // Snap to the start of the character containing `position`.
        let mut position = position;
        while position > 0 && !text.is_char_boundary(position) {
            position -= 1;
        }

        // Byte offset just past the character at `position`.
        let end = position
            + text[position..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(0);

        // Try every start within the last 21 characters (abbreviations are
        // short) and keep the longest match. Iteration moves the start
        // backwards, so each successful match is longer than the previous
        // one and can simply overwrite it.
        let mut best: Option<AbbreviationMatch> = None;
        for (start, _) in text[..end].char_indices().rev().take(21) {
            let candidate = &text[start..end];
            if let Some(node) = self.walk(candidate) {
                if node.is_end {
                    best = Some(AbbreviationMatch {
                        abbreviation: candidate.to_string(),
                        length: end - start,
                        category: node.category.clone(),
                    });
                }
            }
        }

        best
    }

    /// Walk the trie over the given candidate string, returning the final
    /// node if every character has a transition.
    fn walk(&self, candidate: &str) -> Option<&TrieNode> {
        let mut current = &self.root;
        for original_ch in candidate.chars() {
            let ch = if self.case_sensitive {
                original_ch
            } else {
                original_ch.to_lowercase().next()?
            };
            current = current.children.get(&ch)?;
        }
        Some(current)
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
