use crate::domain::language::traits::{BoundaryContext, BoundaryDecision};
use crate::domain::BoundaryFlags;
use std::collections::HashSet;

/// Pattern context for terminator analysis
#[derive(Debug, Clone)]
pub struct PatternContext<'a> {
    pub text: &'a str,
    pub position: usize,
    pub current_char: char,
    pub next_char: Option<char>,
    pub prev_char: Option<char>,
}

/// Recognized terminator patterns (e.g., "!?" or "?!")
#[derive(Debug, Clone, PartialEq)]
pub enum TerminatorPattern {
    /// Surprised question mark pattern (!?)
    SurprisedQuestion,
    /// Questioning exclamation pattern (?!)
    QuestioningExclamation,
}

/// Rules for handling terminator characters
#[derive(Debug, Clone)]
pub struct TerminatorRules {
    /// Set of terminator characters
    chars: HashSet<char>,
    /// ASCII lookup table for performance
    ascii_lookup: [bool; 128],
    /// Recognized patterns
    patterns: Vec<(String, TerminatorPattern)>,
}

impl TerminatorRules {
    /// Get the configured patterns
    pub fn patterns(&self) -> &[(String, TerminatorPattern)] {
        &self.patterns
    }

    /// Create new terminator rules from configuration
    pub fn new(chars: Vec<char>, patterns: Vec<(String, String)>) -> Self {
        let char_set: HashSet<char> = chars.into_iter().collect();

        // Build ASCII lookup table
        let mut ascii_lookup = [false; 128];
        for &ch in &char_set {
            if ch.is_ascii() {
                ascii_lookup[ch as usize] = true;
            }
        }

        // Parse patterns
        let parsed_patterns: Vec<(String, TerminatorPattern)> = patterns
            .into_iter()
            .filter_map(|(pattern, name)| {
                let pattern_type = match name.as_str() {
                    "surprised_question" => Some(TerminatorPattern::SurprisedQuestion),
                    "questioning_exclamation" => Some(TerminatorPattern::QuestioningExclamation),
                    _ => None,
                };
                pattern_type.map(|pt| (pattern, pt))
            })
            .collect();

        Self {
            chars: char_set,
            ascii_lookup,
            patterns: parsed_patterns,
        }
    }

    /// Check if a character is a terminator
    #[inline]
    pub fn is_terminator(&self, ch: char) -> bool {
        if ch.is_ascii() {
            self.ascii_lookup[ch as usize]
        } else {
            self.chars.contains(&ch)
        }
    }

    /// Match patterns at the given position
    pub fn match_pattern(&self, context: &PatternContext) -> Option<TerminatorPattern> {
        // Only check patterns if current char is a terminator
        if !self.is_terminator(context.current_char) {
            return None;
        }

        // Check each pattern
        for (pattern_str, pattern_type) in &self.patterns {
            if self.check_pattern_at_position(context, pattern_str) {
                return Some(pattern_type.clone());
            }
        }

        None
    }

    /// Check if a specific pattern matches at the position
    fn check_pattern_at_position(&self, context: &PatternContext, pattern: &str) -> bool {
        let pattern_bytes = pattern.as_bytes();
        let text_bytes = context.text.as_bytes();

        // Calculate the start position for pattern matching
        let pattern_len = pattern.len();
        if pattern_len == 0 || context.position + 1 < pattern_len {
            return false;
        }

        let start_pos = context.position + 1 - pattern_len;

        // Check if we have enough bytes
        if start_pos + pattern_len > text_bytes.len() {
            return false;
        }

        // Compare the pattern
        &text_bytes[start_pos..start_pos + pattern_len] == pattern_bytes
    }

    /// Evaluate a single terminator (when not part of a pattern)
    pub fn evaluate_single_terminator(&self, context: &BoundaryContext) -> BoundaryDecision {
        match context.boundary_char {
            '!' | '?' | '！' | '？' => BoundaryDecision::Boundary(BoundaryFlags::STRONG),
            '.' | '。' => {
                // Check if this is a decimal number (digit before and after the dot)
                let has_digit_before = context
                    .preceding_context
                    .chars()
                    .last()
                    .map(|ch| ch.is_ascii_digit())
                    .unwrap_or(false);
                let has_digit_after = context
                    .following_context
                    .chars()
                    .next()
                    .map(|ch| ch.is_ascii_digit())
                    .unwrap_or(false);

                if has_digit_before && has_digit_after {
                    // This is a decimal number - not a sentence boundary
                    BoundaryDecision::NotBoundary
                } else {
                    BoundaryDecision::Boundary(BoundaryFlags::WEAK)
                }
            }
            _ => BoundaryDecision::NotBoundary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminator_detection() {
        let rules = TerminatorRules::new(vec!['.', '!', '?'], vec![]);

        assert!(rules.is_terminator('.'));
        assert!(rules.is_terminator('!'));
        assert!(rules.is_terminator('?'));
        assert!(!rules.is_terminator(','));
        assert!(!rules.is_terminator('a'));
    }

    #[test]
    fn test_ascii_optimization() {
        let rules = TerminatorRules::new(vec!['.', '!', '?', '。'], vec![]);

        // ASCII characters should use lookup table
        assert!(rules.is_terminator('.'));
        assert!(rules.is_terminator('!'));

        // Non-ASCII should fall back to HashSet
        assert!(rules.is_terminator('。'));
    }

    #[test]
    fn test_pattern_matching() {
        let rules = TerminatorRules::new(
            vec!['.', '!', '?'],
            vec![
                ("!?".to_string(), "surprised_question".to_string()),
                ("?!".to_string(), "questioning_exclamation".to_string()),
            ],
        );

        let text = "What!? Really?!";

        // Test at position 5 (after "What!?")
        let context = PatternContext {
            text,
            position: 5,
            current_char: '?',
            next_char: Some(' '),
            prev_char: Some('!'),
        };

        assert_eq!(
            rules.match_pattern(&context),
            Some(TerminatorPattern::SurprisedQuestion)
        );

        // Test at position 14 (after "Really?!")
        let context = PatternContext {
            text,
            position: 14,
            current_char: '!',
            next_char: None,
            prev_char: Some('?'),
        };

        assert_eq!(
            rules.match_pattern(&context),
            Some(TerminatorPattern::QuestioningExclamation)
        );
    }

    #[test]
    fn test_evaluate_single_terminator() {
        let rules = TerminatorRules::new(vec!['.', '!', '?'], vec![]);

        // Test strong boundary
        let context = BoundaryContext {
            text: "Hello!".to_string(),
            position: 5,
            boundary_char: '!',
            preceding_context: "Hello".to_string(),
            following_context: "".to_string(),
        };

        match rules.evaluate_single_terminator(&context) {
            BoundaryDecision::Boundary(flags) => assert_eq!(flags, BoundaryFlags::STRONG),
            _ => panic!("Expected strong boundary"),
        }

        // Test weak boundary
        let context = BoundaryContext {
            text: "Hello.".to_string(),
            position: 5,
            boundary_char: '.',
            preceding_context: "Hello".to_string(),
            following_context: "".to_string(),
        };

        match rules.evaluate_single_terminator(&context) {
            BoundaryDecision::Boundary(flags) => assert_eq!(flags, BoundaryFlags::WEAK),
            _ => panic!("Expected weak boundary"),
        }
    }
}
