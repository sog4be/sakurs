//! Domain traits for sentence boundary detection
//!
//! This module consolidates all pure domain traits that represent business logic
//! without any execution or infrastructure concerns.

use crate::domain::state::PartialState;

// ============================================================================
// Character Classification
// ============================================================================

/// Classification of characters for boundary detection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterClass {
    /// Sentence-ending punctuation
    SentenceTerminal,
    /// Opening delimiter (quote, parenthesis, etc.)
    DelimiterOpen,
    /// Closing delimiter
    DelimiterClose,
    /// Whitespace character
    Whitespace,
    /// Alphabetic character
    Alphabetic,
    /// Numeric character
    Numeric,
    /// Other punctuation
    OtherPunctuation,
    /// Other character type
    Other,
}

/// Pure character classification logic
pub trait CharacterClassifier: Send + Sync {
    /// Classify a character
    fn classify(&self, ch: char) -> CharacterClass;

    /// Check if character is a sentence terminal
    fn is_sentence_terminal(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::SentenceTerminal)
    }

    /// Check if character is an opening delimiter
    fn is_delimiter_open(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::DelimiterOpen)
    }

    /// Check if character is a closing delimiter
    fn is_delimiter_close(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::DelimiterClose)
    }

    /// Check if character is whitespace
    fn is_whitespace(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::Whitespace)
    }

    /// Get the matching closing delimiter for an opening delimiter
    fn get_matching_delimiter(&self, open: char) -> Option<char> {
        match open {
            '(' => Some(')'),
            '[' => Some(']'),
            '{' => Some('}'),
            '"' => Some('"'),
            '\'' => Some('\''),
            '「' => Some('」'),
            '『' => Some('』'),
            '（' => Some('）'),
            '［' => Some('］'),
            '｛' => Some('｝'),
            '〔' => Some('〕'),
            '【' => Some('】'),
            '〈' => Some('〉'),
            '《' => Some('》'),
            _ => None,
        }
    }
}

// ============================================================================
// Language-Specific Rules
// ============================================================================

/// Type of quotation mark
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteType {
    /// Single quote (')
    Single,
    /// Double quote (")
    Double,
    /// Japanese corner bracket (「」)
    JapaneseCorner,
    /// Japanese double corner bracket (『』)
    JapaneseDoubleCorner,
    /// Other quote type
    Other,
}

/// Behavior for quotes in sentence boundary detection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteBehavior {
    /// Quote can contain sentence boundaries
    AllowBoundaries,
    /// Quote suppresses internal boundaries
    SuppressBoundaries,
    /// Context-dependent behavior
    Contextual,
}

/// Language-specific rules (pure logic)
pub trait LanguageSpecificRules: Send + Sync {
    /// Check if a word is an abbreviation
    fn is_abbreviation(&self, word: &str) -> bool;

    /// Get quote behavior for a quote type
    fn quote_behavior(&self, quote_type: QuoteType) -> QuoteBehavior;

    /// Check if a period after a word likely indicates abbreviation
    fn is_abbreviation_context(&self, word_before: &str, char_after: Option<char>) -> bool {
        // Default implementation
        if self.is_abbreviation(word_before) {
            // If known abbreviation, check what follows
            match char_after {
                Some(ch) if ch.is_uppercase() => false, // "Dr. Smith" - might be sentence boundary
                Some(ch) if ch.is_whitespace() => true, // Abbreviation with space
                _ => true,                              // Default to abbreviation
            }
        } else {
            false
        }
    }

    /// Check if sentence boundary should be suppressed based on following context
    fn should_suppress_boundary(&self, text_after: &str) -> bool {
        // Default: don't suppress if followed by uppercase or significant whitespace
        if let Some(first_non_ws) = text_after.chars().find(|c| !c.is_whitespace()) {
            !first_non_ws.is_uppercase()
        } else {
            true // Suppress if nothing follows
        }
    }

    /// Get language code
    fn language_code(&self) -> &str;
}

// ============================================================================
// Boundary Analysis
// ============================================================================

/// Context for boundary analysis
#[derive(Clone, Debug)]
pub struct BoundaryContext {
    /// Text content around the potential boundary
    pub text_before: String,
    pub text_after: String,
    /// Position in the original text
    pub position: usize,
    /// Character at the boundary position
    pub boundary_char: char,
    /// Current enclosure depth
    pub enclosure_depth: i32,
}

/// Represents a potential sentence boundary (trait version)
#[derive(Clone, Debug)]
pub struct BoundaryCandidateInfo {
    /// Position in the text
    pub position: usize,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Context at this position
    pub context: BoundaryContext,
    /// Type of boundary marker
    pub marker_type: BoundaryMarkerType,
}

/// Types of boundary markers
#[derive(Clone, Debug, PartialEq)]
pub enum BoundaryMarkerType {
    /// Period (.)
    Period,
    /// Question mark (?)
    Question,
    /// Exclamation mark (!)
    Exclamation,
    /// Other punctuation
    Other(char),
}

/// Decision about a boundary candidate
#[derive(Clone, Debug, PartialEq)]
pub enum BoundaryDecision {
    /// Confirmed sentence boundary
    Confirmed { confidence: f32 },
    /// Rejected as boundary
    Rejected { reason: RejectionReason },
    /// Needs more context to decide
    Pending,
}

/// Reasons for rejecting a boundary
#[derive(Clone, Debug, PartialEq)]
pub enum RejectionReason {
    /// Part of an abbreviation
    Abbreviation,
    /// Inside quotes or parentheses
    InsideEnclosure,
    /// Not followed by appropriate spacing/capitalization
    InvalidFollowing,
    /// Other language-specific reason
    LanguageSpecific(String),
}

/// Pure boundary detection logic
pub trait BoundaryAnalyzer: Send + Sync {
    /// Analyze a potential boundary position
    fn analyze_candidate(&self, context: &BoundaryContext) -> BoundaryCandidateInfo;

    /// Evaluate a boundary candidate with accumulated state
    fn evaluate_boundary(
        &self,
        candidate: &BoundaryCandidateInfo,
        state: &PartialState,
    ) -> BoundaryDecision;

    /// Check if a character could be a sentence boundary
    fn is_potential_boundary(&self, ch: char) -> bool {
        matches!(ch, '.' | '?' | '!' | '。' | '？' | '！' | '…' | '‥')
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Character classifier tests
    #[test]
    fn test_character_classification() {
        // Basic test to ensure traits compile
        struct TestClassifier;

        impl CharacterClassifier for TestClassifier {
            fn classify(&self, ch: char) -> CharacterClass {
                match ch {
                    '.' | '?' | '!' => CharacterClass::SentenceTerminal,
                    '(' | '[' | '{' => CharacterClass::DelimiterOpen,
                    ')' | ']' | '}' => CharacterClass::DelimiterClose,
                    ' ' | '\t' | '\n' => CharacterClass::Whitespace,
                    'a'..='z' | 'A'..='Z' => CharacterClass::Alphabetic,
                    '0'..='9' => CharacterClass::Numeric,
                    _ => CharacterClass::Other,
                }
            }
        }

        let classifier = TestClassifier;
        assert!(classifier.is_sentence_terminal('.'));
        assert!(!classifier.is_sentence_terminal('a'));
        assert_eq!(classifier.get_matching_delimiter('('), Some(')'));
    }

    // Language-specific rules tests
    #[test]
    fn test_language_rules() {
        struct TestRules;

        impl LanguageSpecificRules for TestRules {
            fn is_abbreviation(&self, word: &str) -> bool {
                matches!(word, "Dr" | "Mr" | "Ms")
            }

            fn quote_behavior(&self, _quote_type: QuoteType) -> QuoteBehavior {
                QuoteBehavior::AllowBoundaries
            }

            fn language_code(&self) -> &str {
                "en"
            }
        }

        let rules = TestRules;
        assert!(rules.is_abbreviation("Dr"));
        assert!(!rules.is_abbreviation("Doctor"));
        assert!(rules.is_abbreviation_context("Dr", Some(' ')));
    }

    // Boundary analyzer tests
    #[test]
    fn test_boundary_analysis() {
        struct TestAnalyzer;

        impl BoundaryAnalyzer for TestAnalyzer {
            fn analyze_candidate(&self, context: &BoundaryContext) -> BoundaryCandidateInfo {
                BoundaryCandidateInfo {
                    position: context.position,
                    confidence: 0.8,
                    context: context.clone(),
                    marker_type: BoundaryMarkerType::Period,
                }
            }

            fn evaluate_boundary(
                &self,
                _candidate: &BoundaryCandidateInfo,
                _state: &PartialState,
            ) -> BoundaryDecision {
                BoundaryDecision::Confirmed { confidence: 0.8 }
            }
        }

        let analyzer = TestAnalyzer;
        assert!(analyzer.is_potential_boundary('.'));
        assert!(analyzer.is_potential_boundary('?'));
        assert!(!analyzer.is_potential_boundary('a'));
    }
}
