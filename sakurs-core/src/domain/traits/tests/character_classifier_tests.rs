//! Tests for character classifier trait and types

use crate::domain::traits::character_classifier::*;

/// Mock implementation of CharacterClassifier for testing
struct MockCharacterClassifier {
    /// Custom classifications for testing
    custom_terminals: Vec<char>,
}

impl MockCharacterClassifier {
    fn new() -> Self {
        Self {
            custom_terminals: vec!['.', '?', '!', '。', '？', '！'],
        }
    }

    fn with_custom_terminals(terminals: Vec<char>) -> Self {
        Self {
            custom_terminals: terminals,
        }
    }
}

impl CharacterClassifier for MockCharacterClassifier {
    fn classify(&self, ch: char) -> CharacterClass {
        if self.custom_terminals.contains(&ch) {
            CharacterClass::SentenceTerminal
        } else if matches!(
            ch,
            '(' | '['
                | '{'
                | '"'
                | '\''
                | '「'
                | '『'
                | '（'
                | '［'
                | '｛'
                | '〔'
                | '【'
                | '〈'
                | '《'
        ) {
            CharacterClass::DelimiterOpen
        } else if matches!(
            ch,
            ')' | ']'
                | '}'
                | '"'
                | '\''
                | '」'
                | '』'
                | '）'
                | '］'
                | '｝'
                | '〕'
                | '】'
                | '〉'
                | '》'
        ) {
            CharacterClass::DelimiterClose
        } else if ch.is_whitespace() {
            CharacterClass::Whitespace
        } else if ch.is_alphabetic() {
            CharacterClass::Alphabetic
        } else if ch.is_numeric() {
            CharacterClass::Numeric
        } else if ch.is_ascii_punctuation() {
            CharacterClass::OtherPunctuation
        } else {
            CharacterClass::Other
        }
    }
}

#[cfg(test)]
mod character_class_tests {
    use super::*;

    #[test]
    fn test_character_class_variants() {
        // Ensure all variants are distinct
        let variants = vec![
            CharacterClass::SentenceTerminal,
            CharacterClass::DelimiterOpen,
            CharacterClass::DelimiterClose,
            CharacterClass::Whitespace,
            CharacterClass::Alphabetic,
            CharacterClass::Numeric,
            CharacterClass::OtherPunctuation,
            CharacterClass::Other,
        ];

        // Test equality
        for (i, v1) in variants.iter().enumerate() {
            for (j, v2) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(v1, v2);
                } else {
                    assert_ne!(v1, v2);
                }
            }
        }
    }

    #[test]
    fn test_ascii_character_classification() {
        let classifier = MockCharacterClassifier::new();

        // Sentence terminals
        assert_eq!(classifier.classify('.'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('?'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('!'), CharacterClass::SentenceTerminal);

        // Opening delimiters
        assert_eq!(classifier.classify('('), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('['), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('{'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('"'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('\''), CharacterClass::DelimiterOpen);

        // Closing delimiters
        assert_eq!(classifier.classify(')'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify(']'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('}'), CharacterClass::DelimiterClose);

        // Whitespace
        assert_eq!(classifier.classify(' '), CharacterClass::Whitespace);
        assert_eq!(classifier.classify('\t'), CharacterClass::Whitespace);
        assert_eq!(classifier.classify('\n'), CharacterClass::Whitespace);
        assert_eq!(classifier.classify('\r'), CharacterClass::Whitespace);

        // Alphabetic
        assert_eq!(classifier.classify('a'), CharacterClass::Alphabetic);
        assert_eq!(classifier.classify('Z'), CharacterClass::Alphabetic);

        // Numeric
        assert_eq!(classifier.classify('0'), CharacterClass::Numeric);
        assert_eq!(classifier.classify('9'), CharacterClass::Numeric);

        // Other punctuation
        assert_eq!(classifier.classify(','), CharacterClass::OtherPunctuation);
        assert_eq!(classifier.classify(';'), CharacterClass::OtherPunctuation);
        assert_eq!(classifier.classify(':'), CharacterClass::OtherPunctuation);
    }

    #[test]
    fn test_unicode_character_classification() {
        let classifier = MockCharacterClassifier::new();

        // Japanese punctuation
        assert_eq!(classifier.classify('。'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('？'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('！'), CharacterClass::SentenceTerminal);

        // Japanese delimiters
        assert_eq!(classifier.classify('「'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('『'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('（'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('［'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('｛'), CharacterClass::DelimiterOpen);

        assert_eq!(classifier.classify('」'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('』'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('）'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('］'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('｝'), CharacterClass::DelimiterClose);

        // Unicode alphabetic
        assert_eq!(classifier.classify('あ'), CharacterClass::Alphabetic);
        assert_eq!(classifier.classify('漢'), CharacterClass::Alphabetic);
        assert_eq!(classifier.classify('α'), CharacterClass::Alphabetic);

        // Unicode whitespace
        assert_eq!(classifier.classify('\u{3000}'), CharacterClass::Whitespace); // Ideographic space
        assert_eq!(classifier.classify('\u{00A0}'), CharacterClass::Whitespace); // Non-breaking space

        // Emoji (classified as Other)
        assert_eq!(classifier.classify('😀'), CharacterClass::Other);
        assert_eq!(classifier.classify('🎉'), CharacterClass::Other);
    }

    #[test]
    fn test_edge_case_characters() {
        let classifier = MockCharacterClassifier::new();

        // Control characters
        assert_eq!(classifier.classify('\0'), CharacterClass::Other);
        assert_eq!(classifier.classify('\u{0001}'), CharacterClass::Other);
        assert_eq!(classifier.classify('\u{007F}'), CharacterClass::Other);

        // Zero-width characters
        assert_eq!(classifier.classify('\u{200B}'), CharacterClass::Other); // Zero-width space
        assert_eq!(classifier.classify('\u{FEFF}'), CharacterClass::Other); // Zero-width no-break space

        // Combining marks (usually classified as Other)
        assert_eq!(classifier.classify('\u{0301}'), CharacterClass::Other); // Combining acute accent
    }

    #[test]
    fn test_custom_terminal_classification() {
        let custom_terminals = vec!['。', '．', ':', ';'];
        let classifier = MockCharacterClassifier::with_custom_terminals(custom_terminals);

        assert_eq!(classifier.classify('。'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('．'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify(':'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify(';'), CharacterClass::SentenceTerminal);

        // Regular terminals not in custom list
        assert_ne!(classifier.classify('.'), CharacterClass::SentenceTerminal);
        assert_ne!(classifier.classify('?'), CharacterClass::SentenceTerminal);
    }
}

#[cfg(test)]
mod get_matching_delimiter_tests {
    use super::*;

    #[test]
    fn test_standard_delimiter_pairs() {
        let classifier = MockCharacterClassifier::new();

        // Standard ASCII pairs
        assert_eq!(classifier.get_matching_delimiter('('), Some(')'));
        assert_eq!(classifier.get_matching_delimiter('['), Some(']'));
        assert_eq!(classifier.get_matching_delimiter('{'), Some('}'));
        assert_eq!(classifier.get_matching_delimiter('"'), Some('"'));
        assert_eq!(classifier.get_matching_delimiter('\''), Some('\''));
    }

    #[test]
    fn test_japanese_delimiter_pairs() {
        let classifier = MockCharacterClassifier::new();

        // Japanese quotation marks
        assert_eq!(classifier.get_matching_delimiter('「'), Some('」'));
        assert_eq!(classifier.get_matching_delimiter('『'), Some('』'));

        // Full-width parentheses and brackets
        assert_eq!(classifier.get_matching_delimiter('（'), Some('）'));
        assert_eq!(classifier.get_matching_delimiter('［'), Some('］'));
        assert_eq!(classifier.get_matching_delimiter('｛'), Some('｝'));

        // Other Japanese brackets
        assert_eq!(classifier.get_matching_delimiter('〔'), Some('〕'));
        assert_eq!(classifier.get_matching_delimiter('【'), Some('】'));
        assert_eq!(classifier.get_matching_delimiter('〈'), Some('〉'));
        assert_eq!(classifier.get_matching_delimiter('《'), Some('》'));
    }

    #[test]
    fn test_non_delimiter_characters() {
        let classifier = MockCharacterClassifier::new();

        // Non-delimiter characters should return None
        assert_eq!(classifier.get_matching_delimiter('a'), None);
        assert_eq!(classifier.get_matching_delimiter('1'), None);
        assert_eq!(classifier.get_matching_delimiter('.'), None);
        assert_eq!(classifier.get_matching_delimiter(' '), None);
        assert_eq!(classifier.get_matching_delimiter('あ'), None);

        // Closing delimiters should also return None
        assert_eq!(classifier.get_matching_delimiter(')'), None);
        assert_eq!(classifier.get_matching_delimiter(']'), None);
        assert_eq!(classifier.get_matching_delimiter('}'), None);
        assert_eq!(classifier.get_matching_delimiter('」'), None);
        assert_eq!(classifier.get_matching_delimiter('』'), None);
    }

    #[test]
    fn test_all_delimiter_pairs_consistency() {
        let classifier = MockCharacterClassifier::new();

        // List of all opening delimiters that should have matches
        let opening_delimiters = vec![
            '(', '[', '{', '"', '\'', '「', '『', '（', '［', '｛', '〔', '【', '〈', '《',
        ];

        // Each opening delimiter should have a matching closing delimiter
        for open_delim in opening_delimiters {
            let close_delim = classifier.get_matching_delimiter(open_delim);
            assert!(
                close_delim.is_some(),
                "No matching delimiter for '{}'",
                open_delim
            );

            // For most delimiters, the closing delimiter should be classified as DelimiterClose
            // Exception: quotes (", ') can serve as both opening and closing
            if let Some(close) = close_delim {
                if close == '"' || close == '\'' {
                    // Quotes can be either opening or closing depending on context
                    assert!(
                        matches!(
                            classifier.classify(close),
                            CharacterClass::DelimiterOpen | CharacterClass::DelimiterClose
                        ),
                        "Quote '{}' should be classified as delimiter",
                        close
                    );
                } else {
                    // All other closing delimiters should be classified as DelimiterClose
                    assert_eq!(
                        classifier.classify(close),
                        CharacterClass::DelimiterClose,
                        "Matching delimiter '{}' for '{}' is not classified as DelimiterClose",
                        close,
                        open_delim
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod classifier_method_tests {
    use super::*;

    #[test]
    fn test_is_sentence_terminal() {
        let classifier = MockCharacterClassifier::new();

        // True cases
        assert!(classifier.is_sentence_terminal('.'));
        assert!(classifier.is_sentence_terminal('?'));
        assert!(classifier.is_sentence_terminal('!'));
        assert!(classifier.is_sentence_terminal('。'));
        assert!(classifier.is_sentence_terminal('？'));
        assert!(classifier.is_sentence_terminal('！'));

        // False cases
        assert!(!classifier.is_sentence_terminal(','));
        assert!(!classifier.is_sentence_terminal('a'));
        assert!(!classifier.is_sentence_terminal(' '));
        assert!(!classifier.is_sentence_terminal('('));
    }

    #[test]
    fn test_is_delimiter_open() {
        let classifier = MockCharacterClassifier::new();

        // True cases
        assert!(classifier.is_delimiter_open('('));
        assert!(classifier.is_delimiter_open('['));
        assert!(classifier.is_delimiter_open('{'));
        assert!(classifier.is_delimiter_open('「'));
        assert!(classifier.is_delimiter_open('『'));

        // False cases
        assert!(!classifier.is_delimiter_open(')'));
        assert!(!classifier.is_delimiter_open(']'));
        assert!(!classifier.is_delimiter_open('.'));
        assert!(!classifier.is_delimiter_open('a'));
    }

    #[test]
    fn test_is_delimiter_close() {
        let classifier = MockCharacterClassifier::new();

        // True cases
        assert!(classifier.is_delimiter_close(')'));
        assert!(classifier.is_delimiter_close(']'));
        assert!(classifier.is_delimiter_close('}'));
        assert!(classifier.is_delimiter_close('」'));
        assert!(classifier.is_delimiter_close('』'));

        // False cases
        assert!(!classifier.is_delimiter_close('('));
        assert!(!classifier.is_delimiter_close('['));
        assert!(!classifier.is_delimiter_close('.'));
        assert!(!classifier.is_delimiter_close('a'));
    }

    #[test]
    fn test_is_whitespace() {
        let classifier = MockCharacterClassifier::new();

        // True cases
        assert!(classifier.is_whitespace(' '));
        assert!(classifier.is_whitespace('\t'));
        assert!(classifier.is_whitespace('\n'));
        assert!(classifier.is_whitespace('\r'));
        assert!(classifier.is_whitespace('\u{3000}')); // Ideographic space
        assert!(classifier.is_whitespace('\u{00A0}')); // Non-breaking space

        // False cases
        assert!(!classifier.is_whitespace('a'));
        assert!(!classifier.is_whitespace('.'));
        assert!(!classifier.is_whitespace('0'));
        assert!(!classifier.is_whitespace('あ'));
    }

    #[test]
    fn test_comprehensive_classification() {
        let classifier = MockCharacterClassifier::new();

        // Test that every character gets exactly one classification
        let test_chars = vec![
            '.', '?', '!', '。', '？', '！', // Terminals
            '(', '[', '{', '「', '『', // Open delimiters
            ')', ']', '}', '」', '』', // Close delimiters
            ' ', '\t', '\n', // Whitespace
            'a', 'Z', 'あ', '漢', // Alphabetic
            '0', '9', // Numeric
            ',', ';', ':', '-', // Other punctuation
            '😀', '\0', '\u{200B}', // Other
        ];

        for ch in test_chars {
            let class = classifier.classify(ch);

            // Verify that helper methods are consistent with classify()
            assert_eq!(
                classifier.is_sentence_terminal(ch),
                matches!(class, CharacterClass::SentenceTerminal),
                "Inconsistent is_sentence_terminal for '{}'",
                ch
            );
            assert_eq!(
                classifier.is_delimiter_open(ch),
                matches!(class, CharacterClass::DelimiterOpen),
                "Inconsistent is_delimiter_open for '{}'",
                ch
            );
            assert_eq!(
                classifier.is_delimiter_close(ch),
                matches!(class, CharacterClass::DelimiterClose),
                "Inconsistent is_delimiter_close for '{}'",
                ch
            );
            assert_eq!(
                classifier.is_whitespace(ch),
                matches!(class, CharacterClass::Whitespace),
                "Inconsistent is_whitespace for '{}'",
                ch
            );
        }
    }
}
