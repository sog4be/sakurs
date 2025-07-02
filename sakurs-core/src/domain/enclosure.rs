//! Enclosure type system for tracking nested delimiters in text.
//!
//! This module provides types and traits for managing enclosures (quotes, parentheses, brackets)
//! during text parsing. It supports language-specific enclosure rules and proper nesting validation.

use std::collections::HashMap;

/// Represents different types of enclosures that can appear in text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnclosureType {
    /// Double quotes: " "
    DoubleQuote,
    /// Single quotes: ' '
    SingleQuote,
    /// Parentheses: ( )
    Parenthesis,
    /// Square brackets: [ ]
    SquareBracket,
    /// Curly braces: { }
    CurlyBrace,
    /// Japanese quotation marks: 「」
    JapaneseQuote,
    /// Japanese double quotation marks: 『』
    JapaneseDoubleQuote,
    /// Japanese angle brackets: 〈〉
    JapaneseAngleBracket,
    /// Japanese double angle brackets: 《》
    JapaneseDoubleAngleBracket,
    /// Japanese lenticular brackets: 【】
    JapaneseLenticularBracket,
    /// Japanese tortoise shell brackets: 〔〕
    JapaneseTortoiseShellBracket,
    /// French quotation marks: « »
    FrenchQuote,
    /// German quotation marks: „ "
    GermanQuote,
    /// Custom enclosure type for language-specific needs
    Custom(u8),
}

/// Represents an enclosure character and its properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnclosureChar {
    /// The type of enclosure this character belongs to
    pub enclosure_type: EnclosureType,
    /// Whether this is an opening delimiter (true) or closing delimiter (false)
    pub is_opening: bool,
}

/// Trait for language-specific enclosure rules.
pub trait EnclosureRules: Send + Sync {
    /// Identifies if a character is an enclosure delimiter and returns its properties.
    fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar>;

    /// Checks if two characters form a matching enclosure pair.
    fn is_matching_pair(&self, open: char, close: char) -> bool;

    /// Returns true if the enclosure type can contain sentence boundaries.
    /// For example, parentheses typically can contain full sentences.
    fn can_contain_sentences(&self, enclosure_type: EnclosureType) -> bool {
        matches!(
            enclosure_type,
            EnclosureType::Parenthesis
                | EnclosureType::SquareBracket
                | EnclosureType::CurlyBrace
                | EnclosureType::JapaneseQuote
                | EnclosureType::JapaneseDoubleQuote
                | EnclosureType::JapaneseAngleBracket
                | EnclosureType::JapaneseDoubleAngleBracket
                | EnclosureType::JapaneseLenticularBracket
                | EnclosureType::JapaneseTortoiseShellBracket
        )
    }
}

/// Standard enclosure rules for most Western languages.
#[derive(Debug, Clone, Default)]
pub struct StandardEnclosureRules {
    /// Mapping of characters to their enclosure properties
    char_map: HashMap<char, EnclosureChar>,
    /// Mapping of opening to closing characters
    pair_map: HashMap<char, char>,
}

impl StandardEnclosureRules {
    /// Creates a new instance with standard Western punctuation rules.
    pub fn new() -> Self {
        let mut char_map = HashMap::new();
        let mut pair_map = HashMap::new();

        // Double quotes
        char_map.insert(
            '"',
            EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: true,
            },
        );
        char_map.insert(
            '"',
            EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: false,
            },
        );
        pair_map.insert('"', '"');

        // Single quotes
        char_map.insert(
            '\'',
            EnclosureChar {
                enclosure_type: EnclosureType::SingleQuote,
                is_opening: true,
            },
        );
        char_map.insert(
            '\'',
            EnclosureChar {
                enclosure_type: EnclosureType::SingleQuote,
                is_opening: false,
            },
        );
        pair_map.insert('\'', '\'');

        // Smart quotes (curly quotes)
        char_map.insert(
            '"',
            EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: true,
            },
        );
        char_map.insert(
            '"',
            EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: false,
            },
        );
        pair_map.insert('"', '"');

        char_map.insert(
            '\u{2018}', // Left single quotation mark
            EnclosureChar {
                enclosure_type: EnclosureType::SingleQuote,
                is_opening: true,
            },
        );
        char_map.insert(
            '\u{2019}', // Right single quotation mark
            EnclosureChar {
                enclosure_type: EnclosureType::SingleQuote,
                is_opening: false,
            },
        );
        pair_map.insert('\u{2018}', '\u{2019}');

        // Parentheses
        char_map.insert(
            '(',
            EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: true,
            },
        );
        char_map.insert(
            ')',
            EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: false,
            },
        );
        pair_map.insert('(', ')');

        // Square brackets
        char_map.insert(
            '[',
            EnclosureChar {
                enclosure_type: EnclosureType::SquareBracket,
                is_opening: true,
            },
        );
        char_map.insert(
            ']',
            EnclosureChar {
                enclosure_type: EnclosureType::SquareBracket,
                is_opening: false,
            },
        );
        pair_map.insert('[', ']');

        // Curly braces
        char_map.insert(
            '{',
            EnclosureChar {
                enclosure_type: EnclosureType::CurlyBrace,
                is_opening: true,
            },
        );
        char_map.insert(
            '}',
            EnclosureChar {
                enclosure_type: EnclosureType::CurlyBrace,
                is_opening: false,
            },
        );
        pair_map.insert('{', '}');

        Self { char_map, pair_map }
    }

    /// Adds support for language-specific enclosures.
    pub fn with_extended_quotes(mut self) -> Self {
        // French quotes
        self.char_map.insert(
            '«',
            EnclosureChar {
                enclosure_type: EnclosureType::FrenchQuote,
                is_opening: true,
            },
        );
        self.char_map.insert(
            '»',
            EnclosureChar {
                enclosure_type: EnclosureType::FrenchQuote,
                is_opening: false,
            },
        );
        self.pair_map.insert('«', '»');

        // German quotes
        self.char_map.insert(
            '„',
            EnclosureChar {
                enclosure_type: EnclosureType::GermanQuote,
                is_opening: true,
            },
        );
        self.char_map.insert(
            '"',
            EnclosureChar {
                enclosure_type: EnclosureType::GermanQuote,
                is_opening: false,
            },
        );
        self.pair_map.insert('„', '"');

        self
    }
}

impl EnclosureRules for StandardEnclosureRules {
    fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        // For ambiguous characters like " and ', we need context to determine
        // if they're opening or closing. For now, return as opening by default.
        // The parser will need to handle this ambiguity.
        match ch {
            '"' | '\'' => Some(EnclosureChar {
                enclosure_type: if ch == '"' {
                    EnclosureType::DoubleQuote
                } else {
                    EnclosureType::SingleQuote
                },
                is_opening: true, // Parser will determine actual direction
            }),
            _ => self.char_map.get(&ch).copied(),
        }
    }

    fn is_matching_pair(&self, open: char, close: char) -> bool {
        self.pair_map.get(&open).is_some_and(|&c| c == close)
    }
}

/// Tracks the state of enclosures during parsing.
#[derive(Debug, Clone, Default)]
pub struct EnclosureStack {
    /// Stack of currently open enclosures
    stack: Vec<(EnclosureType, char)>,
}

impl EnclosureStack {
    /// Creates a new empty enclosure stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current nesting depth.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Pushes an opening enclosure onto the stack.
    pub fn push(&mut self, enclosure_type: EnclosureType, open_char: char) {
        self.stack.push((enclosure_type, open_char));
    }

    /// Attempts to close an enclosure, returning true if successful.
    pub fn close(&mut self, enclosure_type: EnclosureType) -> bool {
        if let Some((last_type, _)) = self.stack.last() {
            if *last_type == enclosure_type {
                self.stack.pop();
                return true;
            }
        }
        false
    }

    /// Returns the type of the innermost enclosure, if any.
    pub fn current_enclosure(&self) -> Option<EnclosureType> {
        self.stack.last().map(|(t, _)| *t)
    }

    /// Checks if we're currently inside any enclosure.
    pub fn is_enclosed(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Clears all enclosures (used for error recovery).
    pub fn clear(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_enclosure_rules() {
        let rules = StandardEnclosureRules::new();

        // Test basic enclosure detection
        assert!(rules.get_enclosure_char('(').is_some());
        assert!(rules.get_enclosure_char(')').is_some());
        assert!(rules.get_enclosure_char('[').is_some());
        assert!(rules.get_enclosure_char('a').is_none());

        // Test matching pairs
        assert!(rules.is_matching_pair('(', ')'));
        assert!(rules.is_matching_pair('[', ']'));
        assert!(rules.is_matching_pair('{', '}'));
        assert!(!rules.is_matching_pair('(', ']'));
    }

    #[test]
    fn test_enclosure_stack() {
        let mut stack = EnclosureStack::new();

        assert_eq!(stack.depth(), 0);
        assert!(!stack.is_enclosed());

        // Push some enclosures
        stack.push(EnclosureType::Parenthesis, '(');
        assert_eq!(stack.depth(), 1);
        assert!(stack.is_enclosed());

        stack.push(EnclosureType::DoubleQuote, '"');
        assert_eq!(stack.depth(), 2);

        // Close in correct order
        assert!(stack.close(EnclosureType::DoubleQuote));
        assert_eq!(stack.depth(), 1);

        assert!(stack.close(EnclosureType::Parenthesis));
        assert_eq!(stack.depth(), 0);

        // Try to close non-existent enclosure
        assert!(!stack.close(EnclosureType::SquareBracket));
    }

    #[test]
    fn test_extended_quotes() {
        let rules = StandardEnclosureRules::new().with_extended_quotes();

        assert!(rules.get_enclosure_char('«').is_some());
        assert!(rules.get_enclosure_char('»').is_some());
        assert!(rules.is_matching_pair('«', '»'));
    }
}
