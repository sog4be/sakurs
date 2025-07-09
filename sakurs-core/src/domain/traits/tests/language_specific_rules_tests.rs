//! Tests for language-specific rules trait and types

use crate::domain::traits::language_specific_rules::*;

/// Mock implementation of LanguageSpecificRules for testing
struct MockLanguageRules {
    /// Known abbreviations
    abbreviations: Vec<&'static str>,
    /// Language code
    lang_code: &'static str,
    /// Custom quote behavior
    custom_quote_behavior: Option<QuoteBehavior>,
}

impl MockLanguageRules {
    fn english() -> Self {
        Self {
            abbreviations: vec![
                "Dr", "Mr", "Mrs", "Ms", "Prof", "Inc", "Ltd", "etc", "vs", "i.e", "e.g",
            ],
            lang_code: "en",
            custom_quote_behavior: None,
        }
    }

    fn japanese() -> Self {
        Self {
            // Japanese doesn't use period-based abbreviations in SBD context
            // Real Japanese implementation handles English abbreviations within Japanese text
            abbreviations: vec![],
            lang_code: "ja",
            custom_quote_behavior: Some(QuoteBehavior::SuppressBoundaries),
        }
    }

    fn minimal() -> Self {
        Self {
            abbreviations: vec![],
            lang_code: "test",
            custom_quote_behavior: None,
        }
    }
}

impl LanguageSpecificRules for MockLanguageRules {
    fn is_abbreviation(&self, word: &str) -> bool {
        self.abbreviations.contains(&word)
    }

    fn quote_behavior(&self, quote_type: QuoteType) -> QuoteBehavior {
        if let Some(behavior) = self.custom_quote_behavior {
            behavior
        } else {
            match quote_type {
                QuoteType::Single | QuoteType::Double => QuoteBehavior::AllowBoundaries,
                QuoteType::JapaneseCorner | QuoteType::JapaneseDoubleCorner => {
                    QuoteBehavior::SuppressBoundaries
                }
                QuoteType::Other => QuoteBehavior::Contextual,
            }
        }
    }

    fn language_code(&self) -> &str {
        self.lang_code
    }
}

#[cfg(test)]
mod quote_type_tests {
    use super::*;

    #[test]
    fn test_quote_type_variants() {
        // Test all variants are distinct
        let variants = vec![
            QuoteType::Single,
            QuoteType::Double,
            QuoteType::JapaneseCorner,
            QuoteType::JapaneseDoubleCorner,
            QuoteType::Other,
        ];

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
    fn test_quote_type_classification() {
        // Test mapping of actual quote characters to types
        fn classify_quote(ch: char) -> QuoteType {
            match ch {
                '\'' | '\u{2018}' | '\u{2019}' => QuoteType::Single, // apostrophe, left single quote, right single quote
                '"' | '\u{201C}' | '\u{201D}' => QuoteType::Double, // quotation mark, left double quote, right double quote
                '「' | '」' => QuoteType::JapaneseCorner,
                '『' | '』' => QuoteType::JapaneseDoubleCorner,
                _ => QuoteType::Other,
            }
        }

        assert_eq!(classify_quote('\''), QuoteType::Single);
        assert_eq!(classify_quote('\u{2019}'), QuoteType::Single); // right single quote
        assert_eq!(classify_quote('"'), QuoteType::Double);
        assert_eq!(classify_quote('\u{201D}'), QuoteType::Double); // right double quote
        assert_eq!(classify_quote('「'), QuoteType::JapaneseCorner);
        assert_eq!(classify_quote('』'), QuoteType::JapaneseDoubleCorner);
        assert_eq!(classify_quote('«'), QuoteType::Other);
        assert_eq!(classify_quote('»'), QuoteType::Other);
    }
}

#[cfg(test)]
mod quote_behavior_tests {
    use super::*;

    #[test]
    fn test_quote_behavior_variants() {
        // Test all behavior variants
        let behaviors = vec![
            QuoteBehavior::AllowBoundaries,
            QuoteBehavior::SuppressBoundaries,
            QuoteBehavior::Contextual,
        ];

        // Test equality and inequality
        for (i, b1) in behaviors.iter().enumerate() {
            for (j, b2) in behaviors.iter().enumerate() {
                if i == j {
                    assert_eq!(b1, b2);
                } else {
                    assert_ne!(b1, b2);
                }
            }
        }
    }

    #[test]
    fn test_behavior_semantics() {
        // Test the semantic meaning of each behavior
        let allow = QuoteBehavior::AllowBoundaries;
        let suppress = QuoteBehavior::SuppressBoundaries;
        let contextual = QuoteBehavior::Contextual;

        // These tests verify our understanding of the behaviors
        assert_ne!(allow, suppress);
        assert_ne!(allow, contextual);
        assert_ne!(suppress, contextual);
    }
}

#[cfg(test)]
mod language_specific_rules_tests {
    use super::*;

    #[test]
    fn test_is_abbreviation_english() {
        let rules = MockLanguageRules::english();

        // Known abbreviations
        assert!(rules.is_abbreviation("Dr"));
        assert!(rules.is_abbreviation("Mr"));
        assert!(rules.is_abbreviation("Mrs"));
        assert!(rules.is_abbreviation("Prof"));
        assert!(rules.is_abbreviation("Inc"));
        assert!(rules.is_abbreviation("etc"));
        assert!(rules.is_abbreviation("vs"));
        assert!(rules.is_abbreviation("i.e"));
        assert!(rules.is_abbreviation("e.g"));

        // Not abbreviations
        assert!(!rules.is_abbreviation("Doctor"));
        assert!(!rules.is_abbreviation("Mister"));
        assert!(!rules.is_abbreviation("hello"));
        assert!(!rules.is_abbreviation(""));
        assert!(!rules.is_abbreviation("Dr.")); // Without period
    }

    #[test]
    fn test_is_abbreviation_japanese() {
        let rules = MockLanguageRules::japanese();

        // Japanese doesn't use period-based abbreviations in SBD context
        // This test verifies that Japanese rules don't have abbreviations
        assert!(!rules.is_abbreviation("株"));
        assert!(!rules.is_abbreviation("有"));
        assert!(!rules.is_abbreviation("社"));
        assert!(!rules.is_abbreviation("氏"));
        assert!(!rules.is_abbreviation("会社"));
        assert!(!rules.is_abbreviation("株式会社"));
        assert!(!rules.is_abbreviation("さん"));
    }

    #[test]
    fn test_quote_behavior_default() {
        let rules = MockLanguageRules::english();

        // English quotes allow boundaries
        assert_eq!(
            rules.quote_behavior(QuoteType::Single),
            QuoteBehavior::AllowBoundaries
        );
        assert_eq!(
            rules.quote_behavior(QuoteType::Double),
            QuoteBehavior::AllowBoundaries
        );

        // Japanese quotes suppress boundaries
        assert_eq!(
            rules.quote_behavior(QuoteType::JapaneseCorner),
            QuoteBehavior::SuppressBoundaries
        );
        assert_eq!(
            rules.quote_behavior(QuoteType::JapaneseDoubleCorner),
            QuoteBehavior::SuppressBoundaries
        );

        // Other quotes are contextual
        assert_eq!(
            rules.quote_behavior(QuoteType::Other),
            QuoteBehavior::Contextual
        );
    }

    #[test]
    fn test_quote_behavior_custom() {
        let rules = MockLanguageRules::japanese();

        // Japanese rules suppress all boundaries in quotes
        assert_eq!(
            rules.quote_behavior(QuoteType::Single),
            QuoteBehavior::SuppressBoundaries
        );
        assert_eq!(
            rules.quote_behavior(QuoteType::Double),
            QuoteBehavior::SuppressBoundaries
        );
        assert_eq!(
            rules.quote_behavior(QuoteType::JapaneseCorner),
            QuoteBehavior::SuppressBoundaries
        );
    }

    #[test]
    fn test_language_code() {
        let english = MockLanguageRules::english();
        assert_eq!(english.language_code(), "en");

        let japanese = MockLanguageRules::japanese();
        assert_eq!(japanese.language_code(), "ja");

        let test = MockLanguageRules::minimal();
        assert_eq!(test.language_code(), "test");
    }

    #[test]
    fn test_is_abbreviation_context_default_impl() {
        let rules = MockLanguageRules::english();

        // Known abbreviation followed by space
        assert!(rules.is_abbreviation_context("Dr", Some(' ')));

        // Known abbreviation followed by lowercase
        assert!(rules.is_abbreviation_context("Dr", Some('s')));

        // Known abbreviation followed by uppercase (might be sentence boundary)
        assert!(!rules.is_abbreviation_context("Dr", Some('S')));

        // Known abbreviation at end of text
        assert!(rules.is_abbreviation_context("Dr", None));

        // Unknown word
        assert!(!rules.is_abbreviation_context("Hello", Some(' ')));
        assert!(!rules.is_abbreviation_context("Hello", Some('W')));
    }

    #[test]
    fn test_should_suppress_boundary_default_impl() {
        let rules = MockLanguageRules::minimal();

        // Followed by uppercase - don't suppress
        assert!(!rules.should_suppress_boundary(" Hello"));
        assert!(!rules.should_suppress_boundary("  Hello"));
        assert!(!rules.should_suppress_boundary("\nHello"));
        assert!(!rules.should_suppress_boundary("\tHello"));

        // Followed by lowercase - suppress
        assert!(rules.should_suppress_boundary(" hello"));
        assert!(rules.should_suppress_boundary("  hello"));

        // Empty or whitespace only - suppress
        assert!(rules.should_suppress_boundary(""));
        assert!(rules.should_suppress_boundary(" "));
        assert!(rules.should_suppress_boundary("   "));
        assert!(rules.should_suppress_boundary("\n"));
        assert!(rules.should_suppress_boundary("\t"));

        // Followed by number or punctuation - suppress
        assert!(rules.should_suppress_boundary(" 123"));
        assert!(rules.should_suppress_boundary(" - "));
        assert!(rules.should_suppress_boundary(" ("));
    }

    #[test]
    fn test_empty_abbreviation_list() {
        let rules = MockLanguageRules::minimal();

        assert!(!rules.is_abbreviation("Dr"));
        assert!(!rules.is_abbreviation(""));
        assert!(!rules.is_abbreviation("anything"));
    }

    #[test]
    fn test_case_sensitive_abbreviations() {
        let rules = MockLanguageRules::english();

        // Abbreviations are case-sensitive in our mock
        assert!(rules.is_abbreviation("Dr"));
        assert!(!rules.is_abbreviation("dr"));
        assert!(!rules.is_abbreviation("DR"));

        assert!(rules.is_abbreviation("Inc"));
        assert!(!rules.is_abbreviation("inc"));
        assert!(!rules.is_abbreviation("INC"));
    }
}
