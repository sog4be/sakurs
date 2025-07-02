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
//! use sakurs_core::domain::language::{LanguageRules, EnglishLanguageRules, BoundaryContext};
//! use sakurs_core::domain::BoundaryFlags;
//!
//! // Create English-specific language rules
//! let rules = EnglishLanguageRules::new();
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

pub mod english;
pub mod rules;
pub mod traits;

// Re-export commonly used types
pub use traits::{
    AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRuleSet, LanguageRules,
    QuotationContext, QuotationDecision,
};

pub use rules::{AbbreviationRule, CompositeRule, PunctuationRule, QuotationRule};

pub use english::{
    CapitalizationAnalysis, EnglishAbbreviationRule, EnglishCapitalizationRule,
    EnglishLanguageRules, EnglishNumberRule, EnglishQuotationRule,
};

/// Default mock implementation for testing
///
/// This provides a simple implementation of LanguageRules that can be used
/// for testing the integration with the domain layer.
#[derive(Debug, Clone)]
pub struct MockLanguageRules {
    pub language_code: String,
    pub language_name: String,
    pub composite_rule: CompositeRule,
}

impl MockLanguageRules {
    /// Create a new mock language rules instance
    pub fn new(language_code: &str, language_name: &str) -> Self {
        Self {
            language_code: language_code.to_string(),
            language_name: language_name.to_string(),
            composite_rule: CompositeRule::new(),
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
        self.composite_rule.analyze_boundary(context)
    }

    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        self.composite_rule
            .abbreviation_rule
            .detect_abbreviation(text, position)
    }

    fn handle_quotation(&self, context: &QuotationContext) -> QuotationDecision {
        self.composite_rule.quotation_rule.classify_quote(context)
    }

    fn language_code(&self) -> &str {
        &self.language_code
    }

    fn language_name(&self) -> &str {
        &self.language_name
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
        let _rule = CompositeRule::new();
        let _decision = BoundaryDecision::NotBoundary;

        let _result = AbbreviationResult {
            is_abbreviation: true,
            length: 3,
            confidence: 0.9,
        };
    }

    #[test]
    fn test_english_language_rules_integration() {
        let rules = EnglishLanguageRules::new();

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

        // Should not detect boundary after "Dr."
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
    fn test_english_rules_complex_scenarios() {
        let rules = EnglishLanguageRules::new();

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
    fn test_english_rules_with_partial_state() {
        use crate::domain::PartialState;

        let rules = EnglishLanguageRules::new();

        // Test complex English text with abbreviations, numbers, and normal sentences
        let text = "Dr. Smith earned his Ph.D. in 1999. He now works at Tech Corp. The company is valued at $2.5 billion! What an achievement.";
        let state = PartialState::from_text_with_rules(text, 0, &rules);

        let boundary_positions: Vec<usize> = state.boundaries.iter().map(|b| b.offset).collect();

        // Should not have boundaries after abbreviations
        assert!(!boundary_positions.contains(&2)); // After "Dr."
        assert!(!boundary_positions.contains(&23)); // After "Ph.D." (first dot)
        assert!(!boundary_positions.contains(&25)); // After "Ph.D." (second dot)
        assert!(!boundary_positions.contains(&61)); // After "Corp."
        assert!(!boundary_positions.contains(&90)); // After "$2.5" (decimal)

        // Should have boundaries after real sentence endings
        assert!(boundary_positions.contains(&34)); // After "1999."
        assert!(boundary_positions.contains(&100)); // After "billion!"
        assert!(boundary_positions.contains(&121)); // After "achievement."
    }
}
