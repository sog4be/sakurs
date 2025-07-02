//! Basic rule implementations for sentence boundary detection
//!
//! This module provides fundamental rule structures that can be composed
//! to create language-specific boundary detection systems.

use super::traits::{
    AbbreviationResult, BoundaryContext, BoundaryDecision, QuotationContext, QuotationDecision,
};
use crate::domain::BoundaryFlags;
use std::collections::HashSet;

/// Basic punctuation-based sentence boundary rule
///
/// Detects sentence boundaries based on punctuation marks like '.', '!', '?'
#[derive(Debug, Clone)]
pub struct PunctuationRule {
    /// Set of characters that can end sentences
    pub sentence_endings: HashSet<char>,
    /// Set of characters that indicate strong boundaries
    pub strong_endings: HashSet<char>,
}

impl PunctuationRule {
    /// Create a new punctuation rule with default English sentence endings
    pub fn new() -> Self {
        let mut sentence_endings = HashSet::new();
        sentence_endings.insert('.');
        sentence_endings.insert('!');
        sentence_endings.insert('?');

        let mut strong_endings = HashSet::new();
        strong_endings.insert('!');
        strong_endings.insert('?');

        Self {
            sentence_endings,
            strong_endings,
        }
    }

    /// Create a punctuation rule with custom ending characters
    pub fn with_endings(sentence_endings: HashSet<char>, strong_endings: HashSet<char>) -> Self {
        Self {
            sentence_endings,
            strong_endings,
        }
    }

    /// Check if a character is a sentence ending
    pub fn is_sentence_ending(&self, ch: char) -> bool {
        self.sentence_endings.contains(&ch)
    }

    /// Determine the boundary strength for a character
    pub fn boundary_strength(&self, ch: char) -> Option<BoundaryFlags> {
        if self.strong_endings.contains(&ch) {
            Some(BoundaryFlags::STRONG)
        } else if self.sentence_endings.contains(&ch) {
            Some(BoundaryFlags::WEAK)
        } else {
            None
        }
    }
}

impl Default for PunctuationRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule for handling abbreviations
///
/// Prevents false sentence boundaries after known abbreviations like "Dr.", "Mrs.", etc.
#[derive(Debug, Clone)]
pub struct AbbreviationRule {
    /// Set of known abbreviations (without the period)
    pub abbreviations: HashSet<String>,
    /// Minimum confidence threshold for abbreviation detection
    pub confidence_threshold: f32,
}

impl AbbreviationRule {
    /// Create a new abbreviation rule with common English abbreviations
    pub fn new() -> Self {
        let mut abbreviations = HashSet::new();

        // Common titles
        abbreviations.insert("Dr".to_string());
        abbreviations.insert("Mr".to_string());
        abbreviations.insert("Mrs".to_string());
        abbreviations.insert("Ms".to_string());
        abbreviations.insert("Prof".to_string());

        // Common abbreviations
        abbreviations.insert("etc".to_string());
        abbreviations.insert("vs".to_string());
        abbreviations.insert("i.e".to_string());
        abbreviations.insert("e.g".to_string());

        Self {
            abbreviations,
            confidence_threshold: 0.8,
        }
    }

    /// Create an abbreviation rule with custom abbreviations
    pub fn with_abbreviations(abbreviations: HashSet<String>) -> Self {
        Self {
            abbreviations,
            confidence_threshold: 0.8,
        }
    }

    /// Check if text at position matches a known abbreviation
    pub fn detect_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        if position == 0 {
            return AbbreviationResult {
                is_abbreviation: false,
                length: 0,
                confidence: 0.0,
            };
        }

        // Look backward from position to find potential abbreviation
        let start_pos = text[..position]
            .rfind(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .map(|p| p + 1)
            .unwrap_or(0);

        if start_pos >= position {
            return AbbreviationResult {
                is_abbreviation: false,
                length: 0,
                confidence: 0.0,
            };
        }

        let potential_abbrev = &text[start_pos..position];

        if self.abbreviations.contains(potential_abbrev) {
            AbbreviationResult {
                is_abbreviation: true,
                length: potential_abbrev.len(),
                confidence: 1.0,
            }
        } else {
            // Check for partial matches or capitalized words (heuristic)
            let confidence = if potential_abbrev
                .chars()
                .all(|c| c.is_ascii_uppercase() || c == '.')
                && potential_abbrev.len() <= 5
            {
                0.6
            } else {
                0.0
            };

            AbbreviationResult {
                is_abbreviation: confidence >= self.confidence_threshold,
                length: if confidence >= self.confidence_threshold {
                    potential_abbrev.len()
                } else {
                    0
                },
                confidence,
            }
        }
    }
}

impl Default for AbbreviationRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule for handling quotation marks
///
/// Manages how quotation marks affect sentence boundary detection
#[derive(Debug, Clone)]
pub struct QuotationRule {
    /// Set of opening quotation marks
    pub opening_quotes: HashSet<char>,
    /// Set of closing quotation marks
    pub closing_quotes: HashSet<char>,
    /// Whether to treat quotes as sentence boundaries
    pub quotes_create_boundaries: bool,
}

impl QuotationRule {
    /// Create a new quotation rule with standard English quotes
    pub fn new() -> Self {
        let mut opening_quotes = HashSet::new();
        opening_quotes.insert('"');
        opening_quotes.insert('\'');
        opening_quotes.insert('"');
        opening_quotes.insert('\'');

        let mut closing_quotes = HashSet::new();
        closing_quotes.insert('"');
        closing_quotes.insert('\'');
        closing_quotes.insert('"');
        closing_quotes.insert('\'');

        Self {
            opening_quotes,
            closing_quotes,
            quotes_create_boundaries: false,
        }
    }

    /// Determine the type of quotation mark
    pub fn classify_quote(&self, context: &QuotationContext) -> QuotationDecision {
        let quote_char = context.quote_char;

        if self.opening_quotes.contains(&quote_char) && !context.inside_quotes {
            QuotationDecision::QuoteStart
        } else if self.closing_quotes.contains(&quote_char) && context.inside_quotes {
            QuotationDecision::QuoteEnd
        } else {
            QuotationDecision::Regular
        }
    }
}

impl Default for QuotationRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Composite rule that combines multiple rule types
///
/// This allows for complex language-specific behavior by composing simpler rules
#[derive(Debug, Clone)]
pub struct CompositeRule {
    pub punctuation_rule: PunctuationRule,
    pub abbreviation_rule: AbbreviationRule,
    pub quotation_rule: QuotationRule,
}

impl CompositeRule {
    /// Create a new composite rule with default sub-rules
    pub fn new() -> Self {
        Self {
            punctuation_rule: PunctuationRule::new(),
            abbreviation_rule: AbbreviationRule::new(),
            quotation_rule: QuotationRule::new(),
        }
    }

    /// Perform comprehensive boundary analysis using all rules
    pub fn analyze_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        // First check if this is a sentence-ending punctuation
        if let Some(strength) = self
            .punctuation_rule
            .boundary_strength(context.boundary_char)
        {
            // Check if this might be an abbreviation
            let abbrev_result = self
                .abbreviation_rule
                .detect_abbreviation(&context.text, context.position);

            if abbrev_result.is_abbreviation && abbrev_result.confidence > 0.7 {
                // Likely an abbreviation, not a sentence boundary
                BoundaryDecision::NotBoundary
            } else {
                // Check following context for capitalization or other indicators
                let following_trimmed = context.following_context.trim_start();
                if following_trimmed.is_empty() {
                    // End of text
                    BoundaryDecision::Boundary(strength)
                } else if following_trimmed
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_uppercase()
                {
                    // Next sentence starts with capital letter
                    BoundaryDecision::Boundary(strength)
                } else {
                    // Might need more context
                    BoundaryDecision::NeedsMoreContext
                }
            }
        } else {
            BoundaryDecision::NotBoundary
        }
    }
}

impl Default for CompositeRule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_punctuation_rule() {
        let rule = PunctuationRule::new();

        assert!(rule.is_sentence_ending('.'));
        assert!(rule.is_sentence_ending('!'));
        assert!(rule.is_sentence_ending('?'));
        assert!(!rule.is_sentence_ending(','));

        assert_eq!(rule.boundary_strength('.'), Some(BoundaryFlags::WEAK));
        assert_eq!(rule.boundary_strength('!'), Some(BoundaryFlags::STRONG));
        assert_eq!(rule.boundary_strength('?'), Some(BoundaryFlags::STRONG));
        assert_eq!(rule.boundary_strength(','), None);
    }

    #[test]
    fn test_abbreviation_rule() {
        let rule = AbbreviationRule::new();

        let result = rule.detect_abbreviation("Dr. Smith", 2);
        assert!(result.is_abbreviation);
        assert_eq!(result.length, 2);
        assert_eq!(result.confidence, 1.0);

        let result = rule.detect_abbreviation("Hello world.", 11);
        assert!(!result.is_abbreviation);
    }

    #[test]
    fn test_quotation_rule() {
        let rule = QuotationRule::new();

        let context = QuotationContext {
            text: "He said \"Hello\"".to_string(),
            position: 8,
            quote_char: '"',
            inside_quotes: false,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteStart);

        let context = QuotationContext {
            text: "He said \"Hello\"".to_string(),
            position: 14,
            quote_char: '"',
            inside_quotes: true,
        };

        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteEnd);
    }

    #[test]
    fn test_composite_rule() {
        let rule = CompositeRule::new();

        // Test normal sentence boundary
        let context = BoundaryContext {
            text: "Hello world. This is a test.".to_string(),
            position: 11,
            boundary_char: '.',
            preceding_context: "Hello world".to_string(),
            following_context: " This is a".to_string(),
        };

        match rule.analyze_boundary(&context) {
            BoundaryDecision::Boundary(flags) => assert_eq!(flags, BoundaryFlags::WEAK),
            _ => panic!("Expected boundary decision"),
        }

        // Test abbreviation (should not be a boundary)
        let context = BoundaryContext {
            text: "Dr. Smith is here.".to_string(),
            position: 2,
            boundary_char: '.',
            preceding_context: "Dr".to_string(),
            following_context: " Smith is ".to_string(),
        };

        assert_eq!(
            rule.analyze_boundary(&context),
            BoundaryDecision::NotBoundary
        );
    }
}
