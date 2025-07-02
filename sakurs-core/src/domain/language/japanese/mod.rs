//! Japanese language rule implementation
//!
//! This module provides comprehensive Japanese language support for sentence boundary detection.
//! It implements the LanguageRules trait with Japanese-specific patterns and behaviors.
//!
//! # Features
//!
//! - **Punctuation handling**: Japanese punctuation marks (。？！、)
//! - **Quote processing**: Japanese quotation marks (「」『』) with proper nesting
//! - **Mixed text support**: Handles English text within Japanese context
//! - **Enclosure integration**: Works with the existing enclosure system
//!
//! # Usage
//!
//! ```rust
//! use sakurs_core::domain::language::japanese::JapaneseLanguageRules;
//! use sakurs_core::domain::language::LanguageRules;
//!
//! let rules = JapaneseLanguageRules::new();
//! assert_eq!(rules.language_code(), "ja");
//! assert_eq!(rules.language_name(), "Japanese");
//! ```

pub mod punctuation;
pub mod quotation;

use crate::domain::enclosure::{EnclosureChar, EnclosureType};
use crate::domain::language::{
    AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRules, QuotationContext,
    QuotationDecision,
};

#[cfg(test)]
use crate::domain::BoundaryFlags;

pub use punctuation::JapanesePunctuationRule;
pub use quotation::JapaneseQuotationRule;

/// Main Japanese language rules implementation
#[derive(Debug, Clone)]
pub struct JapaneseLanguageRules {
    /// Punctuation rule for sentence boundary detection
    punctuation_rule: JapanesePunctuationRule,

    /// Quotation rule for quote handling
    quotation_rule: JapaneseQuotationRule,

    /// Language metadata
    language_code: String,
    language_name: String,
}

impl JapaneseLanguageRules {
    /// Creates a new Japanese language rules instance
    pub fn new() -> Self {
        Self {
            punctuation_rule: JapanesePunctuationRule::new(),
            quotation_rule: JapaneseQuotationRule::new(),
            language_code: "ja".to_string(),
            language_name: "Japanese".to_string(),
        }
    }

    /// Creates a new Japanese language rules instance with strict modes
    pub fn new_strict() -> Self {
        Self {
            punctuation_rule: JapanesePunctuationRule::new_strict(),
            quotation_rule: JapaneseQuotationRule::new(),
            language_code: "ja".to_string(),
            language_name: "Japanese (Strict)".to_string(),
        }
    }

    /// Creates a new Japanese language rules instance with relaxed modes
    pub fn new_relaxed() -> Self {
        Self {
            punctuation_rule: JapanesePunctuationRule::new(),
            quotation_rule: JapaneseQuotationRule::new_relaxed(),
            language_code: "ja".to_string(),
            language_name: "Japanese (Relaxed)".to_string(),
        }
    }

    /// Gets access to the punctuation rule
    pub fn punctuation_rule(&self) -> &JapanesePunctuationRule {
        &self.punctuation_rule
    }

    /// Gets access to the quotation rule
    pub fn quotation_rule(&self) -> &JapaneseQuotationRule {
        &self.quotation_rule
    }

    /// Validates quote pairing in the given text
    pub fn validate_quote_pairing(&self, text: &str) -> Result<(), String> {
        self.quotation_rule.validate_pairing(text)
    }

    /// Analyzes quote nesting depth at a position
    pub fn analyze_quote_nesting(&self, text: &str, position: usize) -> usize {
        self.quotation_rule.analyze_nesting(text, position)
    }
}

impl LanguageRules for JapaneseLanguageRules {
    fn detect_sentence_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        // Delegate to punctuation rule for boundary detection
        self.punctuation_rule.analyze_boundary(context)
    }

    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        // Japanese has fewer traditional abbreviations than English
        // Most abbreviations are handled by the punctuation rule

        if position >= text.len() {
            return AbbreviationResult {
                is_abbreviation: false,
                length: 0,
                confidence: 0.0,
            };
        }

        // Check for basic Japanese abbreviation patterns
        let chars: Vec<char> = text.chars().collect();

        // Check if current position is a Japanese abbreviation
        if let Some(&current_char) = chars.get(position) {
            if matches!(current_char, '株' | '有' | '会' | '社') {
                return AbbreviationResult {
                    is_abbreviation: true,
                    length: 1,
                    confidence: 0.8,
                };
            }
        }

        // Check if the character before the position is a Japanese abbreviation
        if position > 0 {
            if let Some(&prev_char) = chars.get(position - 1) {
                if matches!(prev_char, '株' | '有' | '会' | '社') {
                    return AbbreviationResult {
                        is_abbreviation: true,
                        length: 1,
                        confidence: 0.8,
                    };
                }
            }
        }

        // For English abbreviations in Japanese text, use basic detection
        if position > 0 {
            let preceding_chars: String = chars
                .iter()
                .take(position)
                .rev()
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            let common_english_abbrevs = ["Dr", "Mr", "Ms", "Prof", "Inc", "Ltd", "Corp"];

            for abbrev in &common_english_abbrevs {
                if preceding_chars.ends_with(abbrev) {
                    return AbbreviationResult {
                        is_abbreviation: true,
                        length: abbrev.len(),
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

    fn handle_quotation(&self, context: &QuotationContext) -> QuotationDecision {
        // Delegate to quotation rule
        self.quotation_rule.classify_quote(context)
    }

    fn language_code(&self) -> &str {
        &self.language_code
    }

    fn language_name(&self) -> &str {
        &self.language_name
    }

    fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        // Handle Japanese quotes first
        if let Some(enclosure) = self.quotation_rule.get_enclosure_char(ch) {
            return Some(enclosure);
        }

        // Handle other common enclosures
        match ch {
            // English quotes in Japanese text
            '"' => Some(EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: true, // Ambiguous - context needed
            }),
            '\'' => Some(EnclosureChar {
                enclosure_type: EnclosureType::SingleQuote,
                is_opening: true, // Ambiguous - context needed
            }),

            // Parentheses (same as English)
            '(' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: true,
            }),
            ')' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: false,
            }),

            // Full-width parentheses (common in Japanese)
            '（' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: true,
            }),
            '）' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: false,
            }),

            _ => None,
        }
    }

    fn get_enclosure_type_id(&self, ch: char) -> Option<usize> {
        self.get_enclosure_char(ch)
            .map(|enc| match enc.enclosure_type {
                EnclosureType::DoubleQuote => 0,
                EnclosureType::SingleQuote => 1,
                EnclosureType::Parenthesis => 2,
                EnclosureType::JapaneseQuote => 3,
                EnclosureType::JapaneseDoubleQuote => 4,
                _ => 0,
            })
    }

    fn enclosure_type_count(&self) -> usize {
        5 // DoubleQuote, SingleQuote, Parenthesis, JapaneseQuote, JapaneseDoubleQuote
    }
}

impl Default for JapaneseLanguageRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_language_rules_creation() {
        let rules = JapaneseLanguageRules::new();
        assert_eq!(rules.language_code(), "ja");
        assert_eq!(rules.language_name(), "Japanese");
        assert_eq!(rules.enclosure_type_count(), 5);
    }

    #[test]
    fn test_strict_and_relaxed_modes() {
        let strict = JapaneseLanguageRules::new_strict();
        let relaxed = JapaneseLanguageRules::new_relaxed();

        assert!(strict.language_name().contains("Strict"));
        assert!(relaxed.language_name().contains("Relaxed"));
    }

    #[test]
    fn test_sentence_boundary_detection() {
        let rules = JapaneseLanguageRules::new();

        // Japanese period
        let context = BoundaryContext {
            text: "これは文です。次の文です。".to_string(),
            position: 6,
            boundary_char: '。',
            preceding_context: "これは文です".to_string(),
            following_context: "次の文です。".to_string(),
        };

        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::STRONG) => {}
            other => panic!(
                "Expected strong boundary for Japanese period, got {:?}",
                other
            ),
        }

        // Japanese comma (not a boundary)
        let context = BoundaryContext {
            text: "これは、テストです。".to_string(),
            position: 3,
            boundary_char: '、',
            preceding_context: "これは".to_string(),
            following_context: "テストです。".to_string(),
        };

        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::NotBoundary => {}
            other => panic!("Expected not boundary for Japanese comma, got {:?}", other),
        }
    }

    #[test]
    fn test_quotation_handling() {
        let rules = JapaneseLanguageRules::new();

        // Japanese opening quote
        let context = QuotationContext {
            text: "彼は「こんにちは」と言った。".to_string(),
            position: 2,
            quote_char: '「',
            inside_quotes: false,
        };

        assert_eq!(
            rules.handle_quotation(&context),
            QuotationDecision::QuoteStart
        );

        // Japanese closing quote
        let context = QuotationContext {
            text: "彼は「こんにちは」と言った。".to_string(),
            position: 8,
            quote_char: '」',
            inside_quotes: true,
        };

        assert_eq!(
            rules.handle_quotation(&context),
            QuotationDecision::QuoteEnd
        );
    }

    #[test]
    fn test_abbreviation_processing() {
        let rules = JapaneseLanguageRules::new();

        // Japanese company abbreviation
        let result = rules.process_abbreviation("トヨタ株", 4);
        assert!(result.is_abbreviation);
        assert_eq!(result.length, 1);
        assert!(result.confidence > 0.7);

        // English abbreviation in Japanese text
        let result = rules.process_abbreviation("Dr. Smith", 2);
        assert!(result.is_abbreviation);
        assert_eq!(result.length, 2);
        assert!(result.confidence > 0.8);

        // Not an abbreviation
        let result = rules.process_abbreviation("普通の文", 3);
        assert!(!result.is_abbreviation);
    }

    #[test]
    fn test_enclosure_character_recognition() {
        let rules = JapaneseLanguageRules::new();

        // Japanese quotes
        let kakko_open = rules.get_enclosure_char('「').unwrap();
        assert_eq!(kakko_open.enclosure_type, EnclosureType::JapaneseQuote);
        assert!(kakko_open.is_opening);

        let kakko_close = rules.get_enclosure_char('」').unwrap();
        assert_eq!(kakko_close.enclosure_type, EnclosureType::JapaneseQuote);
        assert!(!kakko_close.is_opening);

        // Full-width parentheses
        let paren_open = rules.get_enclosure_char('（').unwrap();
        assert_eq!(paren_open.enclosure_type, EnclosureType::Parenthesis);
        assert!(paren_open.is_opening);

        let paren_close = rules.get_enclosure_char('）').unwrap();
        assert_eq!(paren_close.enclosure_type, EnclosureType::Parenthesis);
        assert!(!paren_close.is_opening);
    }

    #[test]
    fn test_enclosure_type_mapping() {
        let rules = JapaneseLanguageRules::new();

        assert_eq!(rules.get_enclosure_type_id('「'), Some(3)); // JapaneseQuote
        assert_eq!(rules.get_enclosure_type_id('』'), Some(4)); // JapaneseDoubleQuote
        assert_eq!(rules.get_enclosure_type_id('('), Some(2)); // Parenthesis
        assert_eq!(rules.get_enclosure_type_id('"'), Some(0)); // DoubleQuote
        assert_eq!(rules.get_enclosure_type_id('\''), Some(1)); // SingleQuote
    }

    #[test]
    fn test_quote_pairing_validation() {
        let rules = JapaneseLanguageRules::new();

        // Valid pairing
        assert!(rules
            .validate_quote_pairing("彼は「こんにちは」と言った。")
            .is_ok());
        assert!(rules
            .validate_quote_pairing("「外側『内側』外側」の構造")
            .is_ok());

        // Invalid pairing
        assert!(rules
            .validate_quote_pairing("彼は「こんにちはと言った。")
            .is_err());
        assert!(rules
            .validate_quote_pairing("彼はこんにちは」と言った。")
            .is_err());
    }

    #[test]
    fn test_quote_nesting_analysis() {
        let rules = JapaneseLanguageRules::new();
        let text = "「外側『内側』外側」";

        assert_eq!(rules.analyze_quote_nesting(text, 0), 0); // Before quotes
        assert_eq!(rules.analyze_quote_nesting(text, 1), 1); // Inside outer quote
        assert_eq!(rules.analyze_quote_nesting(text, 4), 2); // Inside nested quote
        assert_eq!(rules.analyze_quote_nesting(text, 7), 1); // Back to outer quote
        assert_eq!(rules.analyze_quote_nesting(text, 10), 0); // After all quotes
    }
}
