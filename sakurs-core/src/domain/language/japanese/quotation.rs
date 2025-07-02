//! Japanese quotation rule implementation
//!
//! This module handles Japanese quotation marks and their nesting patterns.
//! Japanese uses different quotation marks than English:
//! - 「」 (U+300C, U+300D) - Single quotes (kakko) for direct speech and emphasis
//! - 『』 (U+300E, U+300F) - Double quotes (nijuu kakko) for titles, nested quotes
//!
//! Nesting pattern: 「outer『inner』outer」

use crate::domain::enclosure::{EnclosureChar, EnclosureType};
use crate::domain::language::{QuotationContext, QuotationDecision};

/// Japanese quotation rule for quote handling
#[derive(Debug, Clone)]
pub struct JapaneseQuotationRule {
    /// Whether to strictly enforce proper quote pairing
    strict_pairing: bool,
}

impl JapaneseQuotationRule {
    /// Creates a new Japanese quotation rule
    pub fn new() -> Self {
        Self {
            strict_pairing: true,
        }
    }

    /// Creates a new Japanese quotation rule with relaxed pairing
    pub fn new_relaxed() -> Self {
        Self {
            strict_pairing: false,
        }
    }

    /// Classifies a quotation mark based on Japanese patterns
    pub fn classify_quote(&self, context: &QuotationContext) -> QuotationDecision {
        match context.quote_char {
            // Japanese single quote opening
            '「' => self.handle_kakko_opening(context),

            // Japanese single quote closing
            '」' => self.handle_kakko_closing(context),

            // Japanese double quote opening
            '『' => self.handle_nijuu_kakko_opening(context),

            // Japanese double quote closing
            '』' => self.handle_nijuu_kakko_closing(context),

            // English quotes in Japanese text
            '"' => self.handle_english_quote(context),
            '\'' => self.handle_english_single_quote(context),
            '`' => self.handle_backtick(context),

            // Other characters
            _ => QuotationDecision::Regular,
        }
    }

    /// Gets the enclosure character for a quote
    pub fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        match ch {
            '「' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseQuote,
                is_opening: true,
            }),
            '」' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseQuote,
                is_opening: false,
            }),
            '『' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseDoubleQuote,
                is_opening: true,
            }),
            '』' => Some(EnclosureChar {
                enclosure_type: EnclosureType::JapaneseDoubleQuote,
                is_opening: false,
            }),
            _ => None,
        }
    }

    /// Handles Japanese single quote opening (「)
    fn handle_kakko_opening(&self, _context: &QuotationContext) -> QuotationDecision {
        // 「 is always a quote start in Japanese
        QuotationDecision::QuoteStart
    }

    /// Handles Japanese single quote closing (」)
    fn handle_kakko_closing(&self, context: &QuotationContext) -> QuotationDecision {
        // 」 is always a quote end, but check if we're inside quotes
        if context.inside_quotes {
            QuotationDecision::QuoteEnd
        } else if self.strict_pairing {
            // Strict mode: closing quote without opening is an error
            QuotationDecision::Regular
        } else {
            // Relaxed mode: allow unmatched closing quotes
            QuotationDecision::QuoteEnd
        }
    }

    /// Handles Japanese double quote opening (『)
    fn handle_nijuu_kakko_opening(&self, _context: &QuotationContext) -> QuotationDecision {
        // 『 is always a quote start in Japanese
        QuotationDecision::QuoteStart
    }

    /// Handles Japanese double quote closing (』)
    fn handle_nijuu_kakko_closing(&self, context: &QuotationContext) -> QuotationDecision {
        // 』 is always a quote end, but check if we're inside quotes
        if context.inside_quotes {
            QuotationDecision::QuoteEnd
        } else if self.strict_pairing {
            // Strict mode: closing quote without opening is an error
            QuotationDecision::Regular
        } else {
            // Relaxed mode: allow unmatched closing quotes
            QuotationDecision::QuoteEnd
        }
    }

    /// Handles English straight quotes in Japanese text
    fn handle_english_quote(&self, context: &QuotationContext) -> QuotationDecision {
        // English quotes are ambiguous - could be opening or closing
        if context.inside_quotes {
            QuotationDecision::QuoteEnd
        } else {
            QuotationDecision::QuoteStart
        }
    }

    /// Handles English single quotes in Japanese text
    fn handle_english_single_quote(&self, context: &QuotationContext) -> QuotationDecision {
        // Check for contractions first (less common in Japanese but possible)
        if self.is_contraction_context(context) {
            return QuotationDecision::Regular;
        }

        // Otherwise treat as quote
        if context.inside_quotes {
            QuotationDecision::QuoteEnd
        } else {
            QuotationDecision::QuoteStart
        }
    }

    /// Handles backticks (often used for code in Japanese tech writing)
    fn handle_backtick(&self, context: &QuotationContext) -> QuotationDecision {
        // Backticks are often used for code snippets
        if context.inside_quotes {
            QuotationDecision::QuoteEnd
        } else {
            QuotationDecision::QuoteStart
        }
    }

    /// Checks if the single quote is part of a contraction
    fn is_contraction_context(&self, context: &QuotationContext) -> bool {
        let text = &context.text;
        let pos = context.position;

        // Check for common English contractions that might appear in Japanese text
        if pos == 0 || pos >= text.len() - 1 {
            return false;
        }

        let chars: Vec<char> = text.chars().collect();
        if pos >= chars.len() {
            return false;
        }

        // Check for patterns like "don't", "can't", "it's"
        let before = chars.get(pos.saturating_sub(1));
        let after = chars.get(pos + 1);

        match (before, after) {
            (Some('n'), Some('t')) => true,                          // n't
            (Some('l'), Some('l')) => true,                          // 'll
            (Some('r'), Some('e')) => true,                          // 're
            (Some('v'), Some('e')) => true,                          // 've
            (Some(c), Some('s')) if c.is_ascii_alphabetic() => true, // 's
            _ => false,
        }
    }

    /// Analyzes quote nesting depth for proper handling
    pub fn analyze_nesting(&self, text: &str, position: usize) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let mut depth: usize = 0;

        for (i, &ch) in chars.iter().enumerate() {
            if i >= position {
                break;
            }

            match ch {
                '「' | '『' => depth += 1,
                '」' | '』' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }

        depth
    }

    /// Validates quote pairing in the given text
    pub fn validate_pairing(&self, text: &str) -> Result<(), String> {
        let mut stack = Vec::new();

        for ch in text.chars() {
            match ch {
                '「' => stack.push('「'),
                '『' => stack.push('『'),
                '」' => {
                    match stack.pop() {
                        Some('「') => {} // Correct pairing
                        Some(other) => {
                            return Err(format!(
                                "Mismatched quote: expected closing for {other}, found 」"
                            ))
                        }
                        None => return Err("Unmatched closing 」 quote".to_string()),
                    }
                }
                '』' => {
                    match stack.pop() {
                        Some('『') => {} // Correct pairing
                        Some(other) => {
                            return Err(format!(
                                "Mismatched quote: expected closing for {other}, found 』"
                            ))
                        }
                        None => return Err("Unmatched closing 』 quote".to_string()),
                    }
                }
                _ => {}
            }
        }

        if !stack.is_empty() {
            return Err(format!("Unmatched opening quotes: {stack:?}"));
        }

        Ok(())
    }
}

impl Default for JapaneseQuotationRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kakko_opening() {
        let rule = JapaneseQuotationRule::new();

        let context = QuotationContext {
            text: "彼は「こんにちは」と言った。".to_string(),
            position: 2,
            quote_char: '「',
            inside_quotes: false,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteStart);
    }

    #[test]
    fn test_kakko_closing() {
        let rule = JapaneseQuotationRule::new();

        let context = QuotationContext {
            text: "彼は「こんにちは」と言った。".to_string(),
            position: 8,
            quote_char: '」',
            inside_quotes: true,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteEnd);
    }

    #[test]
    fn test_nijuu_kakko_opening() {
        let rule = JapaneseQuotationRule::new();

        let context = QuotationContext {
            text: "本のタイトルは『吾輩は猫である』です。".to_string(),
            position: 8,
            quote_char: '『',
            inside_quotes: false,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteStart);
    }

    #[test]
    fn test_nijuu_kakko_closing() {
        let rule = JapaneseQuotationRule::new();

        let context = QuotationContext {
            text: "本のタイトルは『吾輩は猫である』です。".to_string(),
            position: 16,
            quote_char: '』',
            inside_quotes: true,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteEnd);
    }

    #[test]
    fn test_nested_quotes() {
        let rule = JapaneseQuotationRule::new();
        let text = "彼は「友達が『面白い』と言った」と報告した。";

        // Check nesting depth at various positions
        assert_eq!(rule.analyze_nesting(text, 0), 0); // Before any quotes
        assert_eq!(rule.analyze_nesting(text, 3), 1); // After first 「
        assert_eq!(rule.analyze_nesting(text, 8), 2); // After 『
        assert_eq!(rule.analyze_nesting(text, 12), 1); // After 』
        assert_eq!(rule.analyze_nesting(text, 18), 0); // After final 」
    }

    #[test]
    fn test_english_quotes_in_japanese() {
        let rule = JapaneseQuotationRule::new();

        let context = QuotationContext {
            text: "彼は\"Hello\"と言った。".to_string(),
            position: 2,
            quote_char: '"',
            inside_quotes: false,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteStart);

        let context = QuotationContext {
            text: "彼は\"Hello\"と言った。".to_string(),
            position: 8,
            quote_char: '"',
            inside_quotes: true,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteEnd);
    }

    #[test]
    fn test_contraction_detection() {
        let rule = JapaneseQuotationRule::new();

        let context = QuotationContext {
            text: "I don't know in Japanese: 知らない".to_string(),
            position: 5, // Position of apostrophe in "don't"
            quote_char: '\'',
            inside_quotes: false,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::Regular);
    }

    #[test]
    fn test_quote_pairing_validation() {
        let rule = JapaneseQuotationRule::new();

        // Valid pairing
        assert!(rule
            .validate_pairing("彼は「こんにちは」と言った。")
            .is_ok());
        assert!(rule
            .validate_pairing("本は『吾輩は猫である』です。")
            .is_ok());
        assert!(rule.validate_pairing("「外側『内側』外側」").is_ok());

        // Invalid pairing
        assert!(rule.validate_pairing("彼は「こんにちはと言った。").is_err()); // Missing closing
        assert!(rule.validate_pairing("彼はこんにちは」と言った。").is_err()); // Missing opening
        assert!(rule.validate_pairing("「『」』").is_err()); // Wrong nesting order
    }

    #[test]
    fn test_enclosure_char_mapping() {
        let rule = JapaneseQuotationRule::new();

        let kakko_open = rule.get_enclosure_char('「').unwrap();
        assert_eq!(kakko_open.enclosure_type, EnclosureType::JapaneseQuote);
        assert!(kakko_open.is_opening);

        let kakko_close = rule.get_enclosure_char('」').unwrap();
        assert_eq!(kakko_close.enclosure_type, EnclosureType::JapaneseQuote);
        assert!(!kakko_close.is_opening);

        let nijuu_open = rule.get_enclosure_char('『').unwrap();
        assert_eq!(
            nijuu_open.enclosure_type,
            EnclosureType::JapaneseDoubleQuote
        );
        assert!(nijuu_open.is_opening);

        let nijuu_close = rule.get_enclosure_char('』').unwrap();
        assert_eq!(
            nijuu_close.enclosure_type,
            EnclosureType::JapaneseDoubleQuote
        );
        assert!(!nijuu_close.is_opening);
    }

    #[test]
    fn test_strict_vs_relaxed_mode() {
        let strict_rule = JapaneseQuotationRule::new();
        let relaxed_rule = JapaneseQuotationRule::new_relaxed();

        let context = QuotationContext {
            text: "unmatched quote」at the end".to_string(),
            position: 15,
            quote_char: '」',
            inside_quotes: false,
        };

        // Strict mode should reject unmatched closing quote
        assert_eq!(
            strict_rule.classify_quote(&context),
            QuotationDecision::Regular
        );

        // Relaxed mode should allow it
        assert_eq!(
            relaxed_rule.classify_quote(&context),
            QuotationDecision::QuoteEnd
        );
    }
}
