//! Adapters to bridge new traits with existing LanguageRules

use super::{
    BoundaryAnalyzer, BoundaryCandidateInfo, BoundaryMarkerType, CharacterClass,
    CharacterClassifier, LanguageSpecificRules, QuoteBehavior, QuoteType, RejectionReason,
    TraitBoundaryContext as NewBoundaryContext, TraitBoundaryDecision as NewBoundaryDecision,
};
use crate::domain::{
    language::{
        BoundaryContext as OldBoundaryContext, BoundaryDecision as OldBoundaryDecision,
        LanguageRules,
    },
    state::PartialState,
    BoundaryFlags,
};

/// Adapter to use existing LanguageRules with new trait system
pub struct LanguageRulesAdapter<T: LanguageRules> {
    inner: T,
}

impl<T: LanguageRules> LanguageRulesAdapter<T> {
    /// Create a new adapter
    pub fn new(rules: T) -> Self {
        Self { inner: rules }
    }
}

impl<T: LanguageRules> CharacterClassifier for LanguageRulesAdapter<T> {
    fn classify(&self, ch: char) -> CharacterClass {
        match ch {
            '.' | '?' | '!' | '。' | '？' | '！' | '…' | '‥' => {
                CharacterClass::SentenceTerminal
            }
            ch if ch.is_whitespace() => CharacterClass::Whitespace,
            ch if ch.is_alphabetic() => CharacterClass::Alphabetic,
            ch if ch.is_numeric() => CharacterClass::Numeric,
            ch if self.inner.get_enclosure_char(ch).is_some() => {
                // Check if it's opening or closing
                if matches!(
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
                } else {
                    CharacterClass::DelimiterClose
                }
            }
            ch if ch.is_ascii_punctuation() => CharacterClass::OtherPunctuation,
            _ => CharacterClass::Other,
        }
    }
}

impl<T: LanguageRules> LanguageSpecificRules for LanguageRulesAdapter<T> {
    fn is_abbreviation(&self, word: &str) -> bool {
        // Use existing abbreviation detection
        let result = self.inner.process_abbreviation(word, word.len());
        result.is_abbreviation
    }

    fn quote_behavior(&self, _quote_type: QuoteType) -> QuoteBehavior {
        // Default behavior - can be enhanced based on language
        QuoteBehavior::Contextual
    }

    fn language_code(&self) -> &str {
        self.inner.language_code()
    }
}

impl<T: LanguageRules> BoundaryAnalyzer for LanguageRulesAdapter<T> {
    fn analyze_candidate(&self, context: &NewBoundaryContext) -> BoundaryCandidateInfo {
        // Determine marker type
        let marker_type = match context.boundary_char {
            '.' | '。' => BoundaryMarkerType::Period,
            '?' | '？' => BoundaryMarkerType::Question,
            '!' | '！' => BoundaryMarkerType::Exclamation,
            ch => BoundaryMarkerType::Other(ch),
        };

        // Initial confidence based on marker type
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
    ) -> NewBoundaryDecision {
        // Convert to old context format
        let old_context = OldBoundaryContext {
            text: format!(
                "{}{}",
                candidate.context.text_before, candidate.context.text_after
            ),
            position: candidate.context.text_before.len(),
            boundary_char: candidate.context.boundary_char,
            preceding_context: candidate.context.text_before.clone(),
            following_context: candidate.context.text_after.clone(),
        };

        // Use existing language rules
        match self.inner.detect_sentence_boundary(&old_context) {
            OldBoundaryDecision::Boundary(flags) => {
                let confidence = if flags.contains(BoundaryFlags::STRONG) {
                    1.0
                } else {
                    0.75
                };
                NewBoundaryDecision::Confirmed { confidence }
            }
            OldBoundaryDecision::NotBoundary => {
                // Determine reason
                let reason = if self.is_abbreviation(&candidate.context.text_before) {
                    RejectionReason::Abbreviation
                } else if candidate.context.enclosure_depth > 0 {
                    RejectionReason::InsideEnclosure
                } else {
                    RejectionReason::InvalidFollowing
                };
                NewBoundaryDecision::Rejected { reason }
            }
            OldBoundaryDecision::NeedsMoreContext => NewBoundaryDecision::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::english::EnglishLanguageRules;

    #[test]
    fn test_language_rules_adapter_character_classification() {
        let rules = EnglishLanguageRules::new();
        let adapter = LanguageRulesAdapter::new(rules);

        // Test sentence terminals
        assert_eq!(adapter.classify('.'), CharacterClass::SentenceTerminal);
        assert_eq!(adapter.classify('?'), CharacterClass::SentenceTerminal);
        assert_eq!(adapter.classify('!'), CharacterClass::SentenceTerminal);

        // Test whitespace
        assert_eq!(adapter.classify(' '), CharacterClass::Whitespace);
        assert_eq!(adapter.classify('\t'), CharacterClass::Whitespace);
        assert_eq!(adapter.classify('\n'), CharacterClass::Whitespace);

        // Test alphabetic
        assert_eq!(adapter.classify('a'), CharacterClass::Alphabetic);
        assert_eq!(adapter.classify('Z'), CharacterClass::Alphabetic);

        // Test numeric
        assert_eq!(adapter.classify('0'), CharacterClass::Numeric);
        assert_eq!(adapter.classify('9'), CharacterClass::Numeric);

        // Test delimiters
        assert_eq!(adapter.classify('('), CharacterClass::DelimiterOpen);
        assert_eq!(adapter.classify(')'), CharacterClass::DelimiterClose);
        assert_eq!(adapter.classify('"'), CharacterClass::DelimiterOpen);

        // Test other punctuation
        assert_eq!(adapter.classify(','), CharacterClass::OtherPunctuation);
        assert_eq!(adapter.classify(';'), CharacterClass::OtherPunctuation);
    }

    #[test]
    fn test_language_specific_rules_adapter() {
        let rules = EnglishLanguageRules::new();
        let adapter = LanguageRulesAdapter::new(rules);

        // Test abbreviation detection
        assert!(adapter.is_abbreviation("Dr"));
        assert!(adapter.is_abbreviation("Mr"));
        assert!(!adapter.is_abbreviation("Hello"));

        // Test language code
        assert_eq!(adapter.language_code(), "en");

        // Test quote behavior
        assert!(matches!(
            adapter.quote_behavior(QuoteType::Double),
            QuoteBehavior::Contextual
        ));
    }

    #[test]
    fn test_boundary_analyzer_adapter() {
        let rules = EnglishLanguageRules::new();
        let adapter = LanguageRulesAdapter::new(rules);

        let context = NewBoundaryContext {
            text_before: "Hello".to_string(),
            text_after: " World".to_string(),
            position: 5,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        // Test candidate analysis
        let candidate = adapter.analyze_candidate(&context);
        assert_eq!(candidate.position, 5);
        assert_eq!(candidate.confidence, 0.7); // Period gets 0.7
        assert!(matches!(candidate.marker_type, BoundaryMarkerType::Period));

        // Test with question mark
        let context_question = NewBoundaryContext {
            boundary_char: '?',
            ..context.clone()
        };
        let candidate_question = adapter.analyze_candidate(&context_question);
        assert_eq!(candidate_question.confidence, 0.9); // Question gets 0.9

        // Test boundary evaluation
        let state = PartialState::default();
        let decision = adapter.evaluate_boundary(&candidate, &state);
        assert!(matches!(decision, NewBoundaryDecision::Confirmed { .. }));
    }

    #[test]
    fn test_japanese_character_classification() {
        use crate::domain::language::japanese::JapaneseLanguageRules;

        let rules = JapaneseLanguageRules::new();
        let adapter = LanguageRulesAdapter::new(rules);

        // Test Japanese sentence terminals
        assert_eq!(adapter.classify('。'), CharacterClass::SentenceTerminal);
        assert_eq!(adapter.classify('？'), CharacterClass::SentenceTerminal);
        assert_eq!(adapter.classify('！'), CharacterClass::SentenceTerminal);

        // Test Japanese brackets
        assert_eq!(adapter.classify('「'), CharacterClass::DelimiterOpen);
        assert_eq!(adapter.classify('」'), CharacterClass::DelimiterClose);
        assert_eq!(adapter.classify('（'), CharacterClass::DelimiterOpen);
        assert_eq!(adapter.classify('）'), CharacterClass::DelimiterClose);
    }
}
