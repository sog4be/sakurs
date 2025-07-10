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

    // ============================================================================
    // Mock Implementations for Testing
    // ============================================================================

    /// Mock implementation of CharacterClassifier for testing
    struct MockCharacterClassifier {
        custom_terminals: Vec<char>,
    }

    impl MockCharacterClassifier {
        fn new() -> Self {
            Self {
                custom_terminals: vec!['.', '?', '!', '。', '？', '！'],
            }
        }

        fn with_custom_terminals(terminals: Vec<char>) -> Self {
            Self {
                custom_terminals: terminals,
            }
        }
    }

    impl CharacterClassifier for MockCharacterClassifier {
        fn classify(&self, ch: char) -> CharacterClass {
            if self.custom_terminals.contains(&ch) {
                CharacterClass::SentenceTerminal
            } else if matches!(
                ch,
                '(' | '['
                    | '{'
                    | '"'
                    | '\''
                    | '「'
                    | '『'
                    | '（'
                    | '［'
                    | '｛'
                    | '〔'
                    | '【'
                    | '〈'
                    | '《'
            ) {
                CharacterClass::DelimiterOpen
            } else if matches!(
                ch,
                ')' | ']'
                    | '}'
                    | '"'
                    | '\''
                    | '」'
                    | '』'
                    | '）'
                    | '］'
                    | '｝'
                    | '〕'
                    | '】'
                    | '〉'
                    | '》'
            ) {
                CharacterClass::DelimiterClose
            } else if ch.is_whitespace() {
                CharacterClass::Whitespace
            } else if ch.is_alphabetic() {
                CharacterClass::Alphabetic
            } else if ch.is_numeric() {
                CharacterClass::Numeric
            } else if ch.is_ascii_punctuation() {
                CharacterClass::OtherPunctuation
            } else {
                CharacterClass::Other
            }
        }
    }

    // ============================================================================
    // CharacterClass Tests
    // ============================================================================

    #[test]
    fn test_character_class_variants() {
        // Ensure all variants are distinct
        let variants = vec![
            CharacterClass::SentenceTerminal,
            CharacterClass::DelimiterOpen,
            CharacterClass::DelimiterClose,
            CharacterClass::Whitespace,
            CharacterClass::Alphabetic,
            CharacterClass::Numeric,
            CharacterClass::OtherPunctuation,
            CharacterClass::Other,
        ];

        // Test equality and distinctness
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
    fn test_character_class_debug() {
        // Ensure Debug is implemented and produces expected format
        assert_eq!(
            format!("{:?}", CharacterClass::SentenceTerminal),
            "SentenceTerminal"
        );
        assert_eq!(
            format!("{:?}", CharacterClass::DelimiterOpen),
            "DelimiterOpen"
        );
    }

    // ============================================================================
    // CharacterClassifier Tests
    // ============================================================================

    #[test]
    fn test_basic_classification() {
        let classifier = MockCharacterClassifier::new();

        // Test sentence terminals
        assert_eq!(classifier.classify('.'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('?'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('!'), CharacterClass::SentenceTerminal);

        // Test whitespace
        assert_eq!(classifier.classify(' '), CharacterClass::Whitespace);
        assert_eq!(classifier.classify('\t'), CharacterClass::Whitespace);
        assert_eq!(classifier.classify('\n'), CharacterClass::Whitespace);

        // Test alphabetic
        assert_eq!(classifier.classify('a'), CharacterClass::Alphabetic);
        assert_eq!(classifier.classify('Z'), CharacterClass::Alphabetic);

        // Test numeric
        assert_eq!(classifier.classify('0'), CharacterClass::Numeric);
        assert_eq!(classifier.classify('9'), CharacterClass::Numeric);

        // Test delimiters
        assert_eq!(classifier.classify('('), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify(')'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('['), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify(']'), CharacterClass::DelimiterClose);
    }

    #[test]
    fn test_japanese_character_classification() {
        let classifier = MockCharacterClassifier::new();

        // Japanese sentence terminals
        assert_eq!(classifier.classify('。'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('？'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('！'), CharacterClass::SentenceTerminal);

        // Japanese brackets
        assert_eq!(classifier.classify('「'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('」'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('『'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('』'), CharacterClass::DelimiterClose);
        assert_eq!(classifier.classify('（'), CharacterClass::DelimiterOpen);
        assert_eq!(classifier.classify('）'), CharacterClass::DelimiterClose);

        // Japanese alphabetic (hiragana, katakana, kanji)
        assert_eq!(classifier.classify('あ'), CharacterClass::Alphabetic);
        assert_eq!(classifier.classify('ア'), CharacterClass::Alphabetic);
        assert_eq!(classifier.classify('漢'), CharacterClass::Alphabetic);
    }

    #[test]
    fn test_unicode_edge_cases() {
        let classifier = MockCharacterClassifier::new();

        // Zero-width characters
        assert_eq!(classifier.classify('\u{200B}'), CharacterClass::Other); // Zero-width space
        assert_eq!(classifier.classify('\u{200C}'), CharacterClass::Other); // Zero-width non-joiner
        assert_eq!(classifier.classify('\u{200D}'), CharacterClass::Other); // Zero-width joiner

        // Combining marks
        assert_eq!(classifier.classify('\u{0301}'), CharacterClass::Other); // Combining acute accent

        // Emojis
        assert_eq!(classifier.classify('😀'), CharacterClass::Other);
        assert_eq!(classifier.classify('🎉'), CharacterClass::Other);

        // Control characters
        assert_eq!(classifier.classify('\u{0000}'), CharacterClass::Other);
        assert_eq!(classifier.classify('\u{001F}'), CharacterClass::Other);
    }

    #[test]
    fn test_helper_methods_consistency() {
        let classifier = MockCharacterClassifier::new();

        // Test is_sentence_terminal
        assert!(classifier.is_sentence_terminal('.'));
        assert!(classifier.is_sentence_terminal('?'));
        assert!(classifier.is_sentence_terminal('！'));
        assert!(!classifier.is_sentence_terminal('a'));
        assert!(!classifier.is_sentence_terminal(' '));

        // Test is_delimiter_open
        assert!(classifier.is_delimiter_open('('));
        assert!(classifier.is_delimiter_open('「'));
        assert!(!classifier.is_delimiter_open(')'));
        assert!(!classifier.is_delimiter_open('a'));

        // Test is_delimiter_close
        assert!(classifier.is_delimiter_close(')'));
        assert!(classifier.is_delimiter_close('」'));
        assert!(!classifier.is_delimiter_close('('));
        assert!(!classifier.is_delimiter_close('a'));

        // Test is_whitespace
        assert!(classifier.is_whitespace(' '));
        assert!(classifier.is_whitespace('\t'));
        assert!(classifier.is_whitespace('\n'));
        assert!(!classifier.is_whitespace('a'));
    }

    #[test]
    fn test_delimiter_matching() {
        let classifier = MockCharacterClassifier::new();

        // ASCII delimiters
        assert_eq!(classifier.get_matching_delimiter('('), Some(')'));
        assert_eq!(classifier.get_matching_delimiter('['), Some(']'));
        assert_eq!(classifier.get_matching_delimiter('{'), Some('}'));
        assert_eq!(classifier.get_matching_delimiter('"'), Some('"'));
        assert_eq!(classifier.get_matching_delimiter('\''), Some('\''));

        // Japanese delimiters
        assert_eq!(classifier.get_matching_delimiter('「'), Some('」'));
        assert_eq!(classifier.get_matching_delimiter('『'), Some('』'));
        assert_eq!(classifier.get_matching_delimiter('（'), Some('）'));
        assert_eq!(classifier.get_matching_delimiter('［'), Some('］'));
        assert_eq!(classifier.get_matching_delimiter('【'), Some('】'));

        // Non-delimiters
        assert_eq!(classifier.get_matching_delimiter('a'), None);
        assert_eq!(classifier.get_matching_delimiter('.'), None);
        assert_eq!(classifier.get_matching_delimiter(' '), None);
    }

    #[test]
    fn test_custom_terminals() {
        let classifier = MockCharacterClassifier::with_custom_terminals(vec!['§', '¶']);

        assert_eq!(classifier.classify('§'), CharacterClass::SentenceTerminal);
        assert_eq!(classifier.classify('¶'), CharacterClass::SentenceTerminal);
        assert_ne!(classifier.classify('.'), CharacterClass::SentenceTerminal);
    }

    // ============================================================================
    // QuoteType and QuoteBehavior Tests
    // ============================================================================

    #[test]
    fn test_quote_type_variants() {
        let variants = vec![
            QuoteType::Single,
            QuoteType::Double,
            QuoteType::JapaneseCorner,
            QuoteType::JapaneseDoubleCorner,
            QuoteType::Other,
        ];

        // Test distinctness
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
    fn test_quote_behavior_variants() {
        let variants = vec![
            QuoteBehavior::AllowBoundaries,
            QuoteBehavior::SuppressBoundaries,
            QuoteBehavior::Contextual,
        ];

        // Test distinctness
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

    // ============================================================================
    // LanguageSpecificRules Tests
    // ============================================================================

    struct MockLanguageRules {
        abbreviations: Vec<String>,
        language: String,
    }

    impl MockLanguageRules {
        fn new_english() -> Self {
            Self {
                abbreviations: vec![
                    "Dr".to_string(),
                    "Mr".to_string(),
                    "Mrs".to_string(),
                    "Ms".to_string(),
                    "Prof".to_string(),
                    "Inc".to_string(),
                    "Ltd".to_string(),
                ],
                language: "en".to_string(),
            }
        }

        fn new_japanese() -> Self {
            Self {
                abbreviations: vec![],
                language: "ja".to_string(),
            }
        }
    }

    impl LanguageSpecificRules for MockLanguageRules {
        fn is_abbreviation(&self, word: &str) -> bool {
            self.abbreviations.iter().any(|abbr| abbr == word)
        }

        fn quote_behavior(&self, quote_type: QuoteType) -> QuoteBehavior {
            match (self.language.as_str(), quote_type) {
                ("en", QuoteType::Double) => QuoteBehavior::AllowBoundaries,
                ("ja", QuoteType::JapaneseCorner) => QuoteBehavior::SuppressBoundaries,
                _ => QuoteBehavior::Contextual,
            }
        }

        fn language_code(&self) -> &str {
            &self.language
        }
    }

    #[test]
    fn test_abbreviation_detection() {
        let rules = MockLanguageRules::new_english();

        // Known abbreviations
        assert!(rules.is_abbreviation("Dr"));
        assert!(rules.is_abbreviation("Mr"));
        assert!(rules.is_abbreviation("Inc"));

        // Not abbreviations
        assert!(!rules.is_abbreviation("Doctor"));
        assert!(!rules.is_abbreviation("Mister"));
        assert!(!rules.is_abbreviation(""));

        // Case sensitivity
        assert!(!rules.is_abbreviation("dr")); // Mock is case-sensitive
        assert!(!rules.is_abbreviation("DR"));
    }

    #[test]
    fn test_abbreviation_context() {
        let rules = MockLanguageRules::new_english();

        // Known abbreviation followed by space
        assert!(rules.is_abbreviation_context("Dr", Some(' ')));

        // Known abbreviation followed by uppercase (potential sentence boundary)
        assert!(!rules.is_abbreviation_context("Dr", Some('S')));

        // Not an abbreviation
        assert!(!rules.is_abbreviation_context("Hello", Some(' ')));
        assert!(!rules.is_abbreviation_context("Hello", Some('W')));

        // Edge cases
        assert!(rules.is_abbreviation_context("Dr", None));
        assert!(!rules.is_abbreviation_context("", Some(' ')));
    }

    #[test]
    fn test_should_suppress_boundary() {
        let rules = MockLanguageRules::new_english();

        // Should not suppress if followed by uppercase
        assert!(!rules.should_suppress_boundary(" World"));
        assert!(!rules.should_suppress_boundary("  Hello"));

        // Should suppress if followed by lowercase
        assert!(rules.should_suppress_boundary(" world"));
        assert!(rules.should_suppress_boundary("  hello"));

        // Should suppress if nothing follows
        assert!(rules.should_suppress_boundary(""));
        assert!(rules.should_suppress_boundary("   "));
    }

    #[test]
    fn test_language_specific_quote_behavior() {
        let en_rules = MockLanguageRules::new_english();
        let ja_rules = MockLanguageRules::new_japanese();

        // English rules
        assert_eq!(
            en_rules.quote_behavior(QuoteType::Double),
            QuoteBehavior::AllowBoundaries
        );
        assert_eq!(
            en_rules.quote_behavior(QuoteType::JapaneseCorner),
            QuoteBehavior::Contextual
        );

        // Japanese rules
        assert_eq!(
            ja_rules.quote_behavior(QuoteType::JapaneseCorner),
            QuoteBehavior::SuppressBoundaries
        );
        assert_eq!(
            ja_rules.quote_behavior(QuoteType::Double),
            QuoteBehavior::Contextual
        );
    }

    // ============================================================================
    // BoundaryAnalyzer Tests
    // ============================================================================

    #[test]
    fn test_boundary_context_creation() {
        let context = BoundaryContext {
            text_before: "Hello".to_string(),
            text_after: " World".to_string(),
            position: 5,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        assert_eq!(context.text_before, "Hello");
        assert_eq!(context.text_after, " World");
        assert_eq!(context.position, 5);
        assert_eq!(context.boundary_char, '.');
        assert_eq!(context.enclosure_depth, 0);
    }

    #[test]
    fn test_boundary_context_clone() {
        let context = BoundaryContext {
            text_before: "Test".to_string(),
            text_after: " text".to_string(),
            position: 4,
            boundary_char: '!',
            enclosure_depth: 2,
        };

        let cloned = context.clone();
        assert_eq!(cloned.text_before, context.text_before);
        assert_eq!(cloned.text_after, context.text_after);
        assert_eq!(cloned.position, context.position);
        assert_eq!(cloned.boundary_char, context.boundary_char);
        assert_eq!(cloned.enclosure_depth, context.enclosure_depth);
    }

    #[test]
    fn test_boundary_context_with_unicode() {
        let context = BoundaryContext {
            text_before: "こんにちは".to_string(),
            text_after: "　世界".to_string(),
            position: 15, // byte position
            boundary_char: '。',
            enclosure_depth: 0,
        };

        assert_eq!(context.text_before, "こんにちは");
        assert_eq!(context.text_after, "　世界");
        assert_eq!(context.boundary_char, '。');
    }

    #[test]
    fn test_boundary_marker_types() {
        let period = BoundaryMarkerType::Period;
        let question = BoundaryMarkerType::Question;
        let _exclamation = BoundaryMarkerType::Exclamation;
        let other = BoundaryMarkerType::Other('…');

        // Test equality
        assert_eq!(period, BoundaryMarkerType::Period);
        assert_ne!(period, question);

        // Test Debug
        assert_eq!(format!("{:?}", period), "Period");
        assert_eq!(format!("{:?}", other), "Other('…')");
    }

    #[test]
    fn test_boundary_decision_variants() {
        let confirmed = BoundaryDecision::Confirmed { confidence: 0.9 };
        let rejected = BoundaryDecision::Rejected {
            reason: RejectionReason::Abbreviation,
        };
        let pending = BoundaryDecision::Pending;

        // Test pattern matching
        match confirmed {
            BoundaryDecision::Confirmed { confidence } => assert_eq!(confidence, 0.9),
            _ => panic!("Wrong variant"),
        }

        match rejected {
            BoundaryDecision::Rejected { reason } => {
                assert_eq!(reason, RejectionReason::Abbreviation)
            }
            _ => panic!("Wrong variant"),
        }

        match pending {
            BoundaryDecision::Pending => (),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_rejection_reasons() {
        let reasons = vec![
            RejectionReason::Abbreviation,
            RejectionReason::InsideEnclosure,
            RejectionReason::InvalidFollowing,
            RejectionReason::LanguageSpecific("Custom reason".to_string()),
        ];

        // Test equality
        assert_eq!(reasons[0], RejectionReason::Abbreviation);
        assert_ne!(reasons[0], reasons[1]);

        // Test LanguageSpecific with different strings
        let ls1 = RejectionReason::LanguageSpecific("A".to_string());
        let ls2 = RejectionReason::LanguageSpecific("B".to_string());
        let ls3 = RejectionReason::LanguageSpecific("A".to_string());
        assert_ne!(ls1, ls2);
        assert_eq!(ls1, ls3);
    }

    struct MockBoundaryAnalyzer;

    impl BoundaryAnalyzer for MockBoundaryAnalyzer {
        fn analyze_candidate(&self, context: &BoundaryContext) -> BoundaryCandidateInfo {
            let marker_type = match context.boundary_char {
                '.' | '。' => BoundaryMarkerType::Period,
                '?' | '？' => BoundaryMarkerType::Question,
                '!' | '！' => BoundaryMarkerType::Exclamation,
                ch => BoundaryMarkerType::Other(ch),
            };

            let base_confidence = match marker_type {
                BoundaryMarkerType::Question | BoundaryMarkerType::Exclamation => 0.9,
                BoundaryMarkerType::Period => 0.7,
                BoundaryMarkerType::Other(_) => 0.5,
            };

            BoundaryCandidateInfo {
                position: context.position,
                confidence: base_confidence,
                context: context.clone(),
                marker_type,
            }
        }

        fn evaluate_boundary(
            &self,
            candidate: &BoundaryCandidateInfo,
            _state: &PartialState,
        ) -> BoundaryDecision {
            if candidate.context.enclosure_depth > 0 {
                BoundaryDecision::Rejected {
                    reason: RejectionReason::InsideEnclosure,
                }
            } else if candidate.confidence >= 0.8 {
                BoundaryDecision::Confirmed {
                    confidence: candidate.confidence,
                }
            } else if candidate.confidence >= 0.5 {
                BoundaryDecision::Pending
            } else {
                BoundaryDecision::Rejected {
                    reason: RejectionReason::InvalidFollowing,
                }
            }
        }
    }

    #[test]
    fn test_boundary_analyzer_implementation() {
        let analyzer = MockBoundaryAnalyzer;

        // Test is_potential_boundary
        assert!(analyzer.is_potential_boundary('.'));
        assert!(analyzer.is_potential_boundary('?'));
        assert!(analyzer.is_potential_boundary('!'));
        assert!(analyzer.is_potential_boundary('。'));
        assert!(analyzer.is_potential_boundary('？'));
        assert!(analyzer.is_potential_boundary('！'));
        assert!(analyzer.is_potential_boundary('…'));
        assert!(analyzer.is_potential_boundary('‥'));
        assert!(!analyzer.is_potential_boundary('a'));
        assert!(!analyzer.is_potential_boundary(' '));
    }

    #[test]
    fn test_boundary_candidate_analysis() {
        let analyzer = MockBoundaryAnalyzer;

        let context = BoundaryContext {
            text_before: "Hello".to_string(),
            text_after: " World".to_string(),
            position: 5,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        let candidate = analyzer.analyze_candidate(&context);
        assert_eq!(candidate.position, 5);
        assert_eq!(candidate.confidence, 0.7); // Period gets 0.7
        assert!(matches!(candidate.marker_type, BoundaryMarkerType::Period));

        // Test with question mark
        let context_q = BoundaryContext {
            boundary_char: '?',
            ..context.clone()
        };
        let candidate_q = analyzer.analyze_candidate(&context_q);
        assert_eq!(candidate_q.confidence, 0.9); // Question gets 0.9
    }

    #[test]
    fn test_boundary_evaluation_with_enclosure() {
        let analyzer = MockBoundaryAnalyzer;
        let state = PartialState::default();

        // Inside enclosure - should be rejected
        let context = BoundaryContext {
            text_before: "Hello".to_string(),
            text_after: " World".to_string(),
            position: 5,
            boundary_char: '.',
            enclosure_depth: 1,
        };

        let candidate = analyzer.analyze_candidate(&context);
        let decision = analyzer.evaluate_boundary(&candidate, &state);

        match decision {
            BoundaryDecision::Rejected { reason } => {
                assert_eq!(reason, RejectionReason::InsideEnclosure);
            }
            _ => panic!("Expected rejection due to enclosure"),
        }
    }

    #[test]
    fn test_boundary_evaluation_confidence_thresholds() {
        let analyzer = MockBoundaryAnalyzer;
        let state = PartialState::default();

        // High confidence - should be confirmed
        let context_high = BoundaryContext {
            text_before: "Hello".to_string(),
            text_after: " World".to_string(),
            position: 5,
            boundary_char: '!', // 0.9 confidence
            enclosure_depth: 0,
        };

        let candidate_high = analyzer.analyze_candidate(&context_high);
        let decision_high = analyzer.evaluate_boundary(&candidate_high, &state);

        match decision_high {
            BoundaryDecision::Confirmed { confidence } => {
                assert_eq!(confidence, 0.9);
            }
            _ => panic!("Expected confirmation with high confidence"),
        }

        // Medium confidence - should be pending
        let context_med = BoundaryContext {
            boundary_char: '.', // 0.7 confidence
            ..context_high.clone()
        };

        let candidate_med = analyzer.analyze_candidate(&context_med);
        let decision_med = analyzer.evaluate_boundary(&candidate_med, &state);

        match decision_med {
            BoundaryDecision::Pending => (),
            _ => panic!("Expected pending decision"),
        }
    }
}
