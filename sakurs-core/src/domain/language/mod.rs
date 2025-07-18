//! Language-specific rules for sentence boundary detection
//!
//! This module provides the language rule system that enables the Delta-Stack
//! Monoid algorithm to work with different languages and their specific
//! sentence boundary detection requirements.
//!
//! # Architecture
//!
//! The language rule system is built on several key components:
//!
//! - **Traits**: Define the interface for language-specific logic
//! - **Rules**: Composable rule implementations for different aspects
//! - **Integration**: Connection points with the core domain layer
//!
//! # Usage
//!
//! ```rust
//! use sakurs_core::domain::language::{LanguageRules, ConfigurableLanguageRules, BoundaryContext};
//! use sakurs_core::domain::BoundaryFlags;
//!
//! // Create language rules from configuration
//! let rules = ConfigurableLanguageRules::from_code("en").unwrap();
//!
//! // Analyze a potential sentence boundary
//! let context = BoundaryContext {
//!     text: "Dr. Smith said hello. This is a test.".to_string(),
//!     position: 18,
//!     boundary_char: '.',
//!     preceding_context: "Smith said hello".to_string(),
//!     following_context: " This is a test.".to_string(),
//! };
//!
//! let decision = rules.detect_sentence_boundary(&context);
//! ```

pub mod config;
pub mod configurable;
pub mod rules;
pub mod traits;

// Re-export commonly used types
pub use traits::{
    AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRuleSet, LanguageRules,
    QuotationContext, QuotationDecision,
};

// Re-export configurable language rules as the primary implementation
pub use configurable::ConfigurableLanguageRules;

/// Default mock implementation for testing
///
/// This provides a simple implementation of LanguageRules that can be used
/// for testing the integration with the domain layer.
#[derive(Debug, Clone)]
pub struct MockLanguageRules {
    pub language_code: String,
    pub language_name: String,
}

impl MockLanguageRules {
    /// Create a new mock language rules instance
    pub fn new(language_code: &str, language_name: &str) -> Self {
        Self {
            language_code: language_code.to_string(),
            language_name: language_name.to_string(),
        }
    }

    /// Create an English mock instance
    pub fn english() -> Self {
        Self::new("en", "English")
    }

    /// Create a Japanese mock instance
    pub fn japanese() -> Self {
        Self::new("ja", "Japanese")
    }
}

impl LanguageRules for MockLanguageRules {
    fn detect_sentence_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        // Simple mock implementation
        match context.boundary_char {
            '.' | '!' | '?' => BoundaryDecision::Boundary(crate::domain::BoundaryFlags::WEAK),
            _ => BoundaryDecision::NotBoundary,
        }
    }

    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        // Simple mock implementation - detect common abbreviations
        let common_abbrs = ["Dr", "Mr", "Mrs", "Ms", "Inc", "Ltd"];

        for abbr in &common_abbrs {
            if position >= abbr.len() {
                let start = position - abbr.len();
                if position < text.len() && text.get(start..position) == Some(abbr) {
                    return AbbreviationResult {
                        is_abbreviation: true,
                        length: abbr.len(),
                        confidence: 0.9,
                    };
                }
            }
        }

        AbbreviationResult {
            is_abbreviation: false,
            length: 0,
            confidence: 0.0,
        }
    }

    fn handle_quotation(&self, _context: &QuotationContext) -> QuotationDecision {
        // Simple mock implementation
        QuotationDecision::QuoteStart
    }

    fn language_code(&self) -> &str {
        &self.language_code
    }

    fn language_name(&self) -> &str {
        &self.language_name
    }

    fn get_enclosure_char(&self, ch: char) -> Option<crate::domain::enclosure::EnclosureChar> {
        // Delegate to a simple implementation similar to English
        use crate::domain::enclosure::{EnclosureChar, EnclosureType};

        #[allow(unreachable_patterns)]
        match ch {
            '"' => Some(EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: true, // Ambiguous straight quote
                is_symmetric: true,
            }),
            '"' => Some(EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: true,
                is_symmetric: false,
            }),
            '"' => Some(EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: false,
                is_symmetric: false,
            }),
            '\'' | '\u{2018}' | '\u{2019}' => Some(EnclosureChar {
                enclosure_type: EnclosureType::SingleQuote,
                is_opening: matches!(ch, '\'' | '\u{2018}'),
                is_symmetric: ch == '\'',
            }),
            '(' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: true,
                is_symmetric: false,
            }),
            ')' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: false,
                is_symmetric: false,
            }),
            '「' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseQuote,
                is_opening: true,
                is_symmetric: false,
            }),
            '」' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseQuote,
                is_opening: false,
                is_symmetric: false,
            }),
            '『' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseDoubleQuote,
                is_opening: true,
                is_symmetric: false,
            }),
            '』' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseDoubleQuote,
                is_opening: false,
                is_symmetric: false,
            }),
            '〈' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseAngleBracket,
                is_opening: true,
                is_symmetric: false,
            }),
            '〉' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseAngleBracket,
                is_opening: false,
                is_symmetric: false,
            }),
            '《' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseDoubleAngleBracket,
                is_opening: true,
                is_symmetric: false,
            }),
            '》' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseDoubleAngleBracket,
                is_opening: false,
                is_symmetric: false,
            }),
            '【' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseLenticularBracket,
                is_opening: true,
                is_symmetric: false,
            }),
            '】' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseLenticularBracket,
                is_opening: false,
                is_symmetric: false,
            }),
            '〔' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseTortoiseShellBracket,
                is_opening: true,
                is_symmetric: false,
            }),
            '〕' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseTortoiseShellBracket,
                is_opening: false,
                is_symmetric: false,
            }),
            _ => None,
        }
    }

    fn get_enclosure_type_id(&self, ch: char) -> Option<usize> {
        use crate::domain::enclosure::EnclosureType;

        self.get_enclosure_char(ch)
            .map(|enc| match enc.enclosure_type {
                EnclosureType::DoubleQuote => 0,
                EnclosureType::SingleQuote => 1,
                EnclosureType::Parenthesis => 2,
                EnclosureType::SquareBracket => 3,
                EnclosureType::JapaneseQuote => 4,
                EnclosureType::JapaneseDoubleQuote => 5,
                EnclosureType::JapaneseAngleBracket => 6,
                EnclosureType::JapaneseDoubleAngleBracket => 7,
                EnclosureType::JapaneseLenticularBracket => 8,
                EnclosureType::JapaneseTortoiseShellBracket => 9,
                _ => 0,
            })
    }

    fn enclosure_type_count(&self) -> usize {
        10 // Support all bracket types including extended Japanese brackets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BoundaryFlags;

    #[test]
    fn test_mock_language_rules_english() {
        let rules = MockLanguageRules::english();

        assert_eq!(rules.language_code(), "en");
        assert_eq!(rules.language_name(), "English");

        // Test boundary detection
        let context = BoundaryContext {
            text: "Hello world. This is a test.".to_string(),
            position: 11,
            boundary_char: '.',
            preceding_context: "Hello world".to_string(),
            following_context: " This is a".to_string(),
        };

        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::WEAK) => {}
            other => panic!("Expected weak boundary, got {:?}", other),
        }
    }

    #[test]
    fn test_mock_language_rules_japanese() {
        let rules = MockLanguageRules::japanese();

        assert_eq!(rules.language_code(), "ja");
        assert_eq!(rules.language_name(), "Japanese");
    }

    #[test]
    fn test_abbreviation_processing() {
        let rules = MockLanguageRules::english();

        let result = rules.process_abbreviation("Dr. Smith", 2);
        assert!(result.is_abbreviation);
        assert_eq!(result.length, 2);
    }

    #[test]
    fn test_quotation_handling() {
        let rules = MockLanguageRules::english();

        let context = QuotationContext {
            text: "He said \"Hello\"".to_string(),
            position: 8,
            quote_char: '"',
            inside_quotes: false,
        };

        assert_eq!(
            rules.handle_quotation(&context),
            QuotationDecision::QuoteStart
        );
    }

    #[test]
    fn test_re_exports() {
        // Test that we can use the re-exported types
        let _decision = BoundaryDecision::NotBoundary;

        let _result = AbbreviationResult {
            is_abbreviation: true,
            length: 3,
            confidence: 0.9,
        };
    }

    #[test]
    fn test_configurable_language_rules_integration() {
        let rules = ConfigurableLanguageRules::from_code("en").unwrap();

        assert_eq!(rules.language_code(), "en");
        assert_eq!(rules.language_name(), "English");

        // Test comprehensive abbreviation handling
        let context = BoundaryContext {
            text: "Dr. Smith works at Apple Inc. and lives on Main St. in the city.".to_string(),
            position: 2,
            boundary_char: '.',
            preceding_context: "Dr".to_string(),
            following_context: " Smith works at Apple Inc. and lives on Main St. in the city."
                .to_string(),
        };

        // "Dr." followed by "Smith" - "Smith" starts with uppercase but is not in
        // the sentence starter list, and use_uppercase_fallback=false, so no boundary
        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NotBoundary
        );

        // Test normal sentence boundary
        let context = BoundaryContext {
            text: "Hello world. This is a test.".to_string(),
            position: 11,
            boundary_char: '.',
            preceding_context: "Hello world".to_string(),
            following_context: " This is a test.".to_string(),
        };

        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::WEAK) => {}
            other => panic!("Expected weak boundary, got {:?}", other),
        }
    }

    #[test]
    fn test_configurable_rules_complex_scenarios() {
        let rules = ConfigurableLanguageRules::from_code("en").unwrap();

        // Test decimal numbers
        let context = BoundaryContext {
            text: "The price is $29.99 for the item.".to_string(),
            position: 16,
            boundary_char: '.',
            preceding_context: "price is $29".to_string(),
            following_context: "99 for the item.".to_string(),
        };

        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NotBoundary
        );

        // Test quotation handling
        let context = BoundaryContext {
            text: "He said, \"Hello world.\" This is next.".to_string(),
            position: 20,
            boundary_char: '.',
            preceding_context: "said, \"Hello world".to_string(),
            following_context: "\" This is next.".to_string(),
        };

        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::Boundary(_) => {}
            other => panic!("Expected boundary after quoted speech, got {:?}", other),
        }

        // Test strong punctuation
        let context = BoundaryContext {
            text: "What a surprise! This is amazing.".to_string(),
            position: 15,
            boundary_char: '!',
            preceding_context: "What a surprise".to_string(),
            following_context: " This is amazing.".to_string(),
        };

        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::STRONG) => {}
            other => panic!(
                "Expected strong boundary after exclamation, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_configurable_abbreviation_processing() {
        let rules = ConfigurableLanguageRules::from_code("en").unwrap();

        // Test abbreviation detection at the end of known abbreviations
        let test_cases = vec![
            ("Dr. Smith", 2, true),      // After "Dr."
            ("Mr. Jones", 2, true),      // After "Mr."
            ("Ph.D. student", 4, true),  // After "Ph.D."
            ("Corp. building", 4, true), // After "Corp."
            ("Inc. company", 3, true),   // After "Inc."
            ("Hello. World", 5, false),  // After "Hello."
            ("Test. Next", 4, false),    // After "Test."
        ];

        for (text, pos, expected_is_abbr) in test_cases {
            let result = rules.process_abbreviation(text, pos);
            assert_eq!(
                result.is_abbreviation, expected_is_abbr,
                "Failed for text '{}' at position {}",
                text, pos
            );
        }
    }
}
