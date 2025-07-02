//! Japanese punctuation rule implementation
//!
//! This module handles Japanese sentence-ending punctuation patterns.
//! Japanese uses different punctuation marks than English:
//! - 。 (U+3002) - Period, main sentence ending
//! - ？ (U+FF1F) - Question mark, interrogative sentences  
//! - ！ (U+FF01) - Exclamation mark, exclamatory sentences
//! - 、 (U+3001) - Comma, phrase separator (NOT a sentence boundary)

use crate::domain::language::{BoundaryContext, BoundaryDecision};
use crate::domain::BoundaryFlags;

/// Japanese punctuation rule for sentence boundary detection
#[derive(Debug, Clone)]
pub struct JapanesePunctuationRule {
    /// Whether to be strict about sentence-ending punctuation
    strict_mode: bool,
}

impl JapanesePunctuationRule {
    /// Creates a new Japanese punctuation rule
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// Creates a new Japanese punctuation rule in strict mode
    pub fn new_strict() -> Self {
        Self { strict_mode: true }
    }

    /// Analyzes a potential boundary based on Japanese punctuation patterns
    pub fn analyze_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        match context.boundary_char {
            // Japanese period - strong sentence ending
            '。' => self.analyze_period(context),

            // Japanese question mark - strong sentence ending
            '？' => BoundaryDecision::Boundary(BoundaryFlags::STRONG),

            // Japanese exclamation mark - strong sentence ending
            '！' => BoundaryDecision::Boundary(BoundaryFlags::STRONG),

            // Japanese comma - NOT a sentence boundary
            '、' => BoundaryDecision::NotBoundary,

            // Other characters - delegate to context analysis
            _ => self.analyze_other_punctuation(context),
        }
    }

    /// Analyzes Japanese period context
    fn analyze_period(&self, context: &BoundaryContext) -> BoundaryDecision {
        // Check if this might be a decimal point (rare in Japanese but possible)
        if self.is_decimal_context(context) {
            return BoundaryDecision::NotBoundary;
        }

        // Japanese period is typically a strong boundary
        BoundaryDecision::Boundary(BoundaryFlags::STRONG)
    }

    /// Analyzes other punctuation marks
    fn analyze_other_punctuation(&self, context: &BoundaryContext) -> BoundaryDecision {
        match context.boundary_char {
            // English punctuation in Japanese text
            '.' => self.analyze_english_period(context),
            '!' => BoundaryDecision::Boundary(BoundaryFlags::STRONG),
            '?' => BoundaryDecision::Boundary(BoundaryFlags::STRONG),

            // Other marks are generally not sentence boundaries
            _ => {
                if self.strict_mode {
                    BoundaryDecision::NotBoundary
                } else {
                    BoundaryDecision::NeedsMoreContext
                }
            }
        }
    }

    /// Analyzes English period in Japanese context
    fn analyze_english_period(&self, context: &BoundaryContext) -> BoundaryDecision {
        // Check if this is part of an English abbreviation or decimal
        if self.is_decimal_context(context) || self.is_english_abbreviation(context) {
            return BoundaryDecision::NotBoundary;
        }

        // English period in Japanese text is usually a boundary but weaker
        BoundaryDecision::Boundary(BoundaryFlags::WEAK)
    }

    /// Checks if the period appears in a decimal number context
    fn is_decimal_context(&self, context: &BoundaryContext) -> bool {
        let preceding = &context.preceding_context;
        let following = &context.following_context;

        // Check for digit before and after
        let has_digit_before = preceding.chars().last().is_some_and(|c| c.is_ascii_digit());
        let has_digit_after = following.chars().next().is_some_and(|c| c.is_ascii_digit());

        has_digit_before && has_digit_after
    }

    /// Checks for English abbreviations in Japanese text
    fn is_english_abbreviation(&self, context: &BoundaryContext) -> bool {
        let preceding = &context.preceding_context;

        // Common English abbreviations that might appear in Japanese text
        let common_abbrevs = [
            "Dr", "Mr", "Ms", "Prof", "Inc", "Ltd", "Corp", "etc", "vs", "e.g", "i.e", "a.m", "p.m",
        ];

        common_abbrevs
            .iter()
            .any(|&abbrev| preceding.ends_with(abbrev))
    }
}

impl Default for JapanesePunctuationRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_period() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "これは文です。次の文です。".to_string(),
            position: 6,
            boundary_char: '。',
            preceding_context: "これは文です".to_string(),
            following_context: "次の文です。".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::STRONG) => {}
            other => panic!(
                "Expected strong boundary for Japanese period, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_japanese_question_mark() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "これは質問ですか？次の文です。".to_string(),
            position: 9,
            boundary_char: '？',
            preceding_context: "これは質問ですか".to_string(),
            following_context: "次の文です。".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::STRONG) => {}
            other => panic!(
                "Expected strong boundary for Japanese question mark, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_japanese_exclamation_mark() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "これは感嘆文です！次の文です。".to_string(),
            position: 9,
            boundary_char: '！',
            preceding_context: "これは感嘆文です".to_string(),
            following_context: "次の文です。".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::STRONG) => {}
            other => panic!(
                "Expected strong boundary for Japanese exclamation mark, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_japanese_comma_not_boundary() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "これは、テストです。".to_string(),
            position: 3,
            boundary_char: '、',
            preceding_context: "これは".to_string(),
            following_context: "テストです。".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::NotBoundary => {}
            other => panic!("Expected not boundary for Japanese comma, got {:?}", other),
        }
    }

    #[test]
    fn test_decimal_in_japanese_text() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "価格は3.14円です。".to_string(),
            position: 4,
            boundary_char: '.',
            preceding_context: "価格は3".to_string(),
            following_context: "14円です。".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::NotBoundary => {}
            other => panic!("Expected not boundary for decimal point, got {:?}", other),
        }
    }

    #[test]
    fn test_english_abbreviation_in_japanese() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "Dr. Smithさんが来ました。".to_string(),
            position: 2,
            boundary_char: '.',
            preceding_context: "Dr".to_string(),
            following_context: " Smithさんが来ました。".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::NotBoundary => {}
            other => panic!(
                "Expected not boundary for English abbreviation, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_english_period_in_japanese_text() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "Hello world. こんにちは世界。".to_string(),
            position: 11,
            boundary_char: '.',
            preceding_context: "Hello world".to_string(),
            following_context: " こんにちは世界。".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::WEAK) => {}
            other => panic!(
                "Expected weak boundary for English period in Japanese text, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn test_strict_mode() {
        let rule = JapanesePunctuationRule::new_strict();

        let context = BoundaryContext {
            text: "不明な文字→次の文".to_string(),
            position: 5,
            boundary_char: '→',
            preceding_context: "不明な文字".to_string(),
            following_context: "次の文".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::NotBoundary => {}
            other => panic!("Expected not boundary in strict mode, got {:?}", other),
        }
    }

    #[test]
    fn test_non_strict_mode() {
        let rule = JapanesePunctuationRule::new();

        let context = BoundaryContext {
            text: "不明な文字→次の文".to_string(),
            position: 5,
            boundary_char: '→',
            preceding_context: "不明な文字".to_string(),
            following_context: "次の文".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::NeedsMoreContext => {}
            other => panic!(
                "Expected needs more context in non-strict mode, got {:?}",
                other
            ),
        }
    }
}
