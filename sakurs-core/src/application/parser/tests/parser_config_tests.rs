//! Tests for parser configuration

use crate::application::parser::{ParserConfig, TextParser};
use crate::domain::enclosure::{EnclosureChar, EnclosureRules, EnclosureType};

/// Mock enclosure rules for testing
struct MockEnclosureRules {
    /// Whether to treat all brackets as enclosures
    all_brackets: bool,
}

impl EnclosureRules for MockEnclosureRules {
    fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        match ch {
            '(' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: true,
            }),
            ')' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: false,
            }),
            '[' => Some(EnclosureChar {
                enclosure_type: EnclosureType::SquareBracket,
                is_opening: true,
            }),
            ']' => Some(EnclosureChar {
                enclosure_type: EnclosureType::SquareBracket,
                is_opening: false,
            }),
            '{' if self.all_brackets => Some(EnclosureChar {
                enclosure_type: EnclosureType::CurlyBrace,
                is_opening: true,
            }),
            '}' if self.all_brackets => Some(EnclosureChar {
                enclosure_type: EnclosureType::CurlyBrace,
                is_opening: false,
            }),
            '<' if self.all_brackets => Some(EnclosureChar {
                enclosure_type: EnclosureType::Custom(0),
                is_opening: true,
            }),
            '>' if self.all_brackets => Some(EnclosureChar {
                enclosure_type: EnclosureType::Custom(0),
                is_opening: false,
            }),
            _ => None,
        }
    }

    fn is_matching_pair(&self, open: char, close: char) -> bool {
        match (open, close) {
            ('(', ')') => true,
            ('[', ']') => true,
            ('{', '}') if self.all_brackets => true,
            ('<', '>') if self.all_brackets => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        // Default config should have standard enclosure rules
        let open_paren = config.enclosure_rules.get_enclosure_char('(');
        assert!(open_paren.is_some());
        assert!(open_paren.unwrap().is_opening);

        let close_paren = config.enclosure_rules.get_enclosure_char(')');
        assert!(close_paren.is_some());
        assert!(!close_paren.unwrap().is_opening);
    }

    #[test]
    fn test_parser_with_custom_config() {
        let config = ParserConfig {
            enclosure_rules: Box::new(MockEnclosureRules { all_brackets: true }),
        };

        let parser = TextParser::with_config(config);

        // Parser should be created with custom config
        // (We can't directly test the config field as it's private,
        // but we can verify the parser was created successfully)
        drop(parser); // Ensure it's valid
    }

    #[test]
    fn test_text_parser_creation() {
        // Test default creation
        let parser = TextParser::new();
        drop(parser); // Ensure it's valid

        // Test Default trait
        let parser = TextParser::default();
        drop(parser); // Ensure it's valid
    }

    #[test]
    fn test_parser_config_with_different_rules() {
        // Test with minimal brackets
        let config1 = ParserConfig {
            enclosure_rules: Box::new(MockEnclosureRules {
                all_brackets: false,
            }),
        };
        assert!(config1.enclosure_rules.get_enclosure_char('(').is_some());
        assert!(config1.enclosure_rules.get_enclosure_char('{').is_none());

        // Test with all brackets
        let config2 = ParserConfig {
            enclosure_rules: Box::new(MockEnclosureRules { all_brackets: true }),
        };
        assert!(config2.enclosure_rules.get_enclosure_char('(').is_some());
        assert!(config2.enclosure_rules.get_enclosure_char('{').is_some());
        assert!(config2.enclosure_rules.get_enclosure_char('<').is_some());
    }

    #[test]
    fn test_enclosure_rules_trait_object() {
        // Test that enclosure rules can be used as trait objects
        let rules: Box<dyn EnclosureRules> = Box::new(MockEnclosureRules { all_brackets: true });

        // Test trait methods
        let open_paren = rules.get_enclosure_char('(');
        assert!(open_paren.is_some());
        assert!(open_paren.unwrap().is_opening);

        let close_paren = rules.get_enclosure_char(')');
        assert!(close_paren.is_some());
        assert!(!close_paren.unwrap().is_opening);

        assert!(rules.is_matching_pair('(', ')'));
        assert!(rules.is_matching_pair('[', ']'));
        assert!(rules.is_matching_pair('{', '}'));
        assert!(!rules.is_matching_pair('(', ']'));
    }

    #[test]
    fn test_enclosure_type_mapping() {
        let rules = MockEnclosureRules { all_brackets: true };

        // Test parentheses
        if let Some(enc) = rules.get_enclosure_char('(') {
            assert_eq!(enc.enclosure_type, EnclosureType::Parenthesis);
            assert!(enc.is_opening);
        }

        // Test square brackets
        if let Some(enc) = rules.get_enclosure_char(']') {
            assert_eq!(enc.enclosure_type, EnclosureType::SquareBracket);
            assert!(!enc.is_opening);
        }

        // Test curly braces
        if let Some(enc) = rules.get_enclosure_char('{') {
            assert_eq!(enc.enclosure_type, EnclosureType::CurlyBrace);
            assert!(enc.is_opening);
        }

        // Test custom type
        if let Some(enc) = rules.get_enclosure_char('<') {
            assert_eq!(enc.enclosure_type, EnclosureType::Custom(0));
            assert!(enc.is_opening);
        }
    }

    #[test]
    fn test_can_contain_sentences() {
        let rules = MockEnclosureRules { all_brackets: true };

        // Test default implementation behavior
        assert!(rules.can_contain_sentences(EnclosureType::Parenthesis));
        assert!(rules.can_contain_sentences(EnclosureType::SquareBracket));
        assert!(rules.can_contain_sentences(EnclosureType::CurlyBrace));
        assert!(!rules.can_contain_sentences(EnclosureType::DoubleQuote));
        assert!(!rules.can_contain_sentences(EnclosureType::SingleQuote));
    }
}
