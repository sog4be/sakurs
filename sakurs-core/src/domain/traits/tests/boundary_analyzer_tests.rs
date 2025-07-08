//! Tests for boundary analyzer traits and types

use crate::domain::state::PartialState;
use crate::domain::traits::boundary_analyzer::*;

/// Mock implementation of BoundaryAnalyzer for testing
struct MockBoundaryAnalyzer {
    /// Whether to always confirm boundaries
    always_confirm: bool,
    /// Confidence to use for confirmed boundaries
    default_confidence: f32,
}

impl MockBoundaryAnalyzer {
    fn new() -> Self {
        Self {
            always_confirm: false,
            default_confidence: 0.8,
        }
    }

    fn always_confirm() -> Self {
        Self {
            always_confirm: true,
            default_confidence: 1.0,
        }
    }
}

impl BoundaryAnalyzer for MockBoundaryAnalyzer {
    fn analyze_candidate(&self, context: &BoundaryContext) -> BoundaryCandidateInfo {
        let marker_type = match context.boundary_char {
            '.' => BoundaryMarkerType::Period,
            '?' => BoundaryMarkerType::Question,
            '!' => BoundaryMarkerType::Exclamation,
            ch => BoundaryMarkerType::Other(ch),
        };

        BoundaryCandidateInfo {
            position: context.position,
            confidence: self.default_confidence,
            context: context.clone(),
            marker_type,
        }
    }

    fn evaluate_boundary(
        &self,
        candidate: &BoundaryCandidateInfo,
        _state: &PartialState,
    ) -> BoundaryDecision {
        if self.always_confirm {
            BoundaryDecision::Confirmed {
                confidence: candidate.confidence,
            }
        } else if candidate.context.enclosure_depth > 0 {
            BoundaryDecision::Rejected {
                reason: RejectionReason::InsideEnclosure,
            }
        } else {
            BoundaryDecision::Pending
        }
    }
}

#[cfg(test)]
mod boundary_context_tests {
    use super::*;

    #[test]
    fn test_boundary_context_creation() {
        let context = BoundaryContext {
            text_before: "Hello world".to_string(),
            text_after: " This is next.".to_string(),
            position: 11,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        assert_eq!(context.text_before, "Hello world");
        assert_eq!(context.text_after, " This is next.");
        assert_eq!(context.position, 11);
        assert_eq!(context.boundary_char, '.');
        assert_eq!(context.enclosure_depth, 0);
    }

    #[test]
    fn test_boundary_context_with_empty_strings() {
        let context = BoundaryContext {
            text_before: "".to_string(),
            text_after: "".to_string(),
            position: 0,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        assert!(context.text_before.is_empty());
        assert!(context.text_after.is_empty());
        assert_eq!(context.position, 0);
    }

    #[test]
    fn test_boundary_context_with_unicode() {
        let context = BoundaryContext {
            text_before: "こんにちは".to_string(),
            text_after: "世界。".to_string(),
            position: 15, // UTF-8 byte position
            boundary_char: '。',
            enclosure_depth: 0,
        };

        assert_eq!(context.text_before, "こんにちは");
        assert_eq!(context.text_after, "世界。");
        assert_eq!(context.boundary_char, '。');
    }

    #[test]
    fn test_boundary_context_with_enclosure() {
        let context = BoundaryContext {
            text_before: "He said \"Hello".to_string(),
            text_after: " world.\"".to_string(),
            position: 14,
            boundary_char: '.',
            enclosure_depth: 1,
        };

        assert_eq!(context.enclosure_depth, 1);
        assert!(context.enclosure_depth > 0);
    }

    #[test]
    fn test_boundary_context_clone() {
        let original = BoundaryContext {
            text_before: "Test".to_string(),
            text_after: " clone".to_string(),
            position: 4,
            boundary_char: '.',
            enclosure_depth: 2,
        };

        let cloned = original.clone();
        assert_eq!(cloned.text_before, original.text_before);
        assert_eq!(cloned.text_after, original.text_after);
        assert_eq!(cloned.position, original.position);
        assert_eq!(cloned.boundary_char, original.boundary_char);
        assert_eq!(cloned.enclosure_depth, original.enclosure_depth);
    }
}

#[cfg(test)]
mod boundary_candidate_info_tests {
    use super::*;

    #[test]
    fn test_boundary_candidate_creation() {
        let context = BoundaryContext {
            text_before: "Test".to_string(),
            text_after: " Next".to_string(),
            position: 4,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        let candidate = BoundaryCandidateInfo {
            position: 4,
            confidence: 0.9,
            context: context.clone(),
            marker_type: BoundaryMarkerType::Period,
        };

        assert_eq!(candidate.position, 4);
        assert_eq!(candidate.confidence, 0.9);
        assert_eq!(candidate.marker_type, BoundaryMarkerType::Period);
    }

    #[test]
    fn test_confidence_values() {
        let context = BoundaryContext {
            text_before: "".to_string(),
            text_after: "".to_string(),
            position: 0,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        // Test various confidence values
        let confidence_values = vec![0.0, 0.5, 1.0, 0.25, 0.75, 0.999];

        for confidence in confidence_values {
            let candidate = BoundaryCandidateInfo {
                position: 0,
                confidence,
                context: context.clone(),
                marker_type: BoundaryMarkerType::Period,
            };

            assert_eq!(candidate.confidence, confidence);
            assert!(candidate.confidence >= 0.0 && candidate.confidence <= 1.0);
        }
    }

    #[test]
    fn test_marker_type_variants() {
        let context = BoundaryContext {
            text_before: "".to_string(),
            text_after: "".to_string(),
            position: 0,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        let test_cases = vec![
            ('.', BoundaryMarkerType::Period),
            ('?', BoundaryMarkerType::Question),
            ('!', BoundaryMarkerType::Exclamation),
            ('。', BoundaryMarkerType::Other('。')),
            ('？', BoundaryMarkerType::Other('？')),
            ('！', BoundaryMarkerType::Other('！')),
            (';', BoundaryMarkerType::Other(';')),
        ];

        for (char, expected_type) in test_cases {
            let candidate = BoundaryCandidateInfo {
                position: 0,
                confidence: 0.5,
                context: context.clone(),
                marker_type: match char {
                    '.' => BoundaryMarkerType::Period,
                    '?' => BoundaryMarkerType::Question,
                    '!' => BoundaryMarkerType::Exclamation,
                    ch => BoundaryMarkerType::Other(ch),
                },
            };

            assert_eq!(candidate.marker_type, expected_type);
        }
    }
}

#[cfg(test)]
mod boundary_decision_tests {
    use super::*;

    #[test]
    fn test_confirmed_decision() {
        let decision = BoundaryDecision::Confirmed { confidence: 0.95 };

        match decision {
            BoundaryDecision::Confirmed { confidence } => {
                assert_eq!(confidence, 0.95);
            }
            _ => panic!("Expected Confirmed decision"),
        }
    }

    #[test]
    fn test_rejected_decision_abbreviation() {
        let decision = BoundaryDecision::Rejected {
            reason: RejectionReason::Abbreviation,
        };

        match decision {
            BoundaryDecision::Rejected { reason } => {
                assert_eq!(reason, RejectionReason::Abbreviation);
            }
            _ => panic!("Expected Rejected decision"),
        }
    }

    #[test]
    fn test_rejected_decision_inside_enclosure() {
        let decision = BoundaryDecision::Rejected {
            reason: RejectionReason::InsideEnclosure,
        };

        match decision {
            BoundaryDecision::Rejected { reason } => {
                assert_eq!(reason, RejectionReason::InsideEnclosure);
            }
            _ => panic!("Expected Rejected decision"),
        }
    }

    #[test]
    fn test_rejected_decision_invalid_following() {
        let decision = BoundaryDecision::Rejected {
            reason: RejectionReason::InvalidFollowing,
        };

        match decision {
            BoundaryDecision::Rejected { reason } => {
                assert_eq!(reason, RejectionReason::InvalidFollowing);
            }
            _ => panic!("Expected Rejected decision"),
        }
    }

    #[test]
    fn test_rejected_decision_language_specific() {
        let decision = BoundaryDecision::Rejected {
            reason: RejectionReason::LanguageSpecific("Custom reason".to_string()),
        };

        match decision {
            BoundaryDecision::Rejected { reason } => match reason {
                RejectionReason::LanguageSpecific(msg) => {
                    assert_eq!(msg, "Custom reason");
                }
                _ => panic!("Expected LanguageSpecific reason"),
            },
            _ => panic!("Expected Rejected decision"),
        }
    }

    #[test]
    fn test_pending_decision() {
        let decision = BoundaryDecision::Pending;

        match decision {
            BoundaryDecision::Pending => {
                // Success
            }
            _ => panic!("Expected Pending decision"),
        }
    }

    #[test]
    fn test_decision_equality() {
        assert_eq!(
            BoundaryDecision::Confirmed { confidence: 0.8 },
            BoundaryDecision::Confirmed { confidence: 0.8 }
        );

        assert_ne!(
            BoundaryDecision::Confirmed { confidence: 0.8 },
            BoundaryDecision::Confirmed { confidence: 0.9 }
        );

        assert_eq!(
            BoundaryDecision::Rejected {
                reason: RejectionReason::Abbreviation,
            },
            BoundaryDecision::Rejected {
                reason: RejectionReason::Abbreviation,
            }
        );
    }
}

#[cfg(test)]
mod boundary_analyzer_trait_tests {
    use super::*;

    #[test]
    fn test_is_potential_boundary_default_impl() {
        let analyzer = MockBoundaryAnalyzer::new();

        // English punctuation
        assert!(analyzer.is_potential_boundary('.'));
        assert!(analyzer.is_potential_boundary('?'));
        assert!(analyzer.is_potential_boundary('!'));

        // Japanese punctuation
        assert!(analyzer.is_potential_boundary('。'));
        assert!(analyzer.is_potential_boundary('？'));
        assert!(analyzer.is_potential_boundary('！'));

        // Ellipsis
        assert!(analyzer.is_potential_boundary('…'));
        assert!(analyzer.is_potential_boundary('‥'));

        // Non-boundary characters
        assert!(!analyzer.is_potential_boundary('a'));
        assert!(!analyzer.is_potential_boundary(','));
        assert!(!analyzer.is_potential_boundary(' '));
        assert!(!analyzer.is_potential_boundary('、'));
    }

    #[test]
    fn test_analyze_candidate() {
        let analyzer = MockBoundaryAnalyzer::new();

        let context = BoundaryContext {
            text_before: "Hello".to_string(),
            text_after: " World".to_string(),
            position: 5,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        let candidate = analyzer.analyze_candidate(&context);

        assert_eq!(candidate.position, 5);
        assert_eq!(candidate.confidence, 0.8);
        assert_eq!(candidate.marker_type, BoundaryMarkerType::Period);
    }

    #[test]
    fn test_evaluate_boundary_always_confirm() {
        let analyzer = MockBoundaryAnalyzer::always_confirm();
        let state = PartialState::default();

        let context = BoundaryContext {
            text_before: "Test".to_string(),
            text_after: " Next".to_string(),
            position: 4,
            boundary_char: '.',
            enclosure_depth: 1, // Even inside enclosure
        };

        let candidate = BoundaryCandidateInfo {
            position: 4,
            confidence: 0.9,
            context,
            marker_type: BoundaryMarkerType::Period,
        };

        let decision = analyzer.evaluate_boundary(&candidate, &state);

        match decision {
            BoundaryDecision::Confirmed { confidence } => {
                assert_eq!(confidence, 0.9);
            }
            _ => panic!("Expected Confirmed decision"),
        }
    }

    #[test]
    fn test_evaluate_boundary_inside_enclosure() {
        let analyzer = MockBoundaryAnalyzer::new();
        let state = PartialState::default();

        let context = BoundaryContext {
            text_before: "Inside (here".to_string(),
            text_after: " there)".to_string(),
            position: 12,
            boundary_char: '.',
            enclosure_depth: 1,
        };

        let candidate = BoundaryCandidateInfo {
            position: 12,
            confidence: 0.8,
            context,
            marker_type: BoundaryMarkerType::Period,
        };

        let decision = analyzer.evaluate_boundary(&candidate, &state);

        match decision {
            BoundaryDecision::Rejected { reason } => {
                assert_eq!(reason, RejectionReason::InsideEnclosure);
            }
            _ => panic!("Expected Rejected decision"),
        }
    }

    #[test]
    fn test_evaluate_boundary_pending() {
        let analyzer = MockBoundaryAnalyzer::new();
        let state = PartialState::default();

        let context = BoundaryContext {
            text_before: "Normal text".to_string(),
            text_after: " continues".to_string(),
            position: 11,
            boundary_char: '.',
            enclosure_depth: 0,
        };

        let candidate = BoundaryCandidateInfo {
            position: 11,
            confidence: 0.8,
            context,
            marker_type: BoundaryMarkerType::Period,
        };

        let decision = analyzer.evaluate_boundary(&candidate, &state);

        assert_eq!(decision, BoundaryDecision::Pending);
    }
}
