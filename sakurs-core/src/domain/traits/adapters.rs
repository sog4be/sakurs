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
