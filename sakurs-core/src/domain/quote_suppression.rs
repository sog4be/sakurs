//! Quote suppression logic for sentence boundary detection
//!
//! This module implements language-aware quote suppression that determines
//! when sentence boundaries inside quoted text should be suppressed.

use crate::domain::{
    enclosure::EnclosureType, language::LanguageRules, state::BoundaryCandidate, BoundaryFlags,
};

/// Configuration for quote suppression behavior
#[derive(Debug, Clone)]
pub struct QuoteSuppressionConfig {
    /// Whether to suppress boundaries in double quotes
    pub suppress_in_double_quotes: bool,
    /// Whether to suppress boundaries in single quotes
    pub suppress_in_single_quotes: bool,
    /// Whether to validate quote pairing
    pub validate_pairing: bool,
    /// Maximum nesting level for quotes (0 = no nesting allowed)
    pub max_nesting_level: usize,
}

impl Default for QuoteSuppressionConfig {
    fn default() -> Self {
        Self {
            suppress_in_double_quotes: true,
            suppress_in_single_quotes: false, // Single quotes often used for contractions
            validate_pairing: true,
            max_nesting_level: 2,
        }
    }
}

/// Context for quote suppression decisions
pub struct QuoteSuppressionContext<'a> {
    /// The boundary candidate to evaluate
    pub candidate: &'a BoundaryCandidate,
    /// Language rules for context
    pub language_rules: &'a dyn LanguageRules,
    /// Current enclosure depths by type
    pub enclosure_depths: &'a [i32],
    /// Configuration for suppression behavior
    pub config: &'a QuoteSuppressionConfig,
}

/// Result of quote suppression evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum SuppressionDecision {
    /// Boundary should be suppressed
    Suppress { reason: String },
    /// Boundary should be kept
    Keep,
    /// Boundary should be weakened but not removed
    Weaken { new_flags: BoundaryFlags },
}

/// Quote suppression evaluator
pub struct QuoteSuppressor;

impl QuoteSuppressor {
    /// Evaluate whether a boundary should be suppressed based on quote context
    pub fn evaluate(context: QuoteSuppressionContext) -> SuppressionDecision {
        let candidate = context.candidate;
        let depths = context.enclosure_depths;
        let config = context.config;

        // Check each enclosure type
        for (type_id, &depth) in depths.iter().enumerate() {
            if depth > 0 {
                // We're inside this enclosure type
                if let Some(decision) = Self::evaluate_enclosure_type(
                    type_id,
                    depth,
                    candidate,
                    context.language_rules,
                    config,
                ) {
                    return decision;
                }
            }
        }

        SuppressionDecision::Keep
    }

    /// Evaluate suppression for a specific enclosure type
    fn evaluate_enclosure_type(
        type_id: usize,
        depth: i32,
        candidate: &BoundaryCandidate,
        language_rules: &dyn LanguageRules,
        config: &QuoteSuppressionConfig,
    ) -> Option<SuppressionDecision> {
        // Map type_id to enclosure type based on language rules
        let enclosure_type = Self::infer_enclosure_type(type_id, language_rules)?;

        match enclosure_type {
            EnclosureType::DoubleQuote => {
                if config.suppress_in_double_quotes {
                    Some(SuppressionDecision::Suppress {
                        reason: format!("Inside double quotes (depth: {depth})"),
                    })
                } else if depth > 1 && config.max_nesting_level > 0 {
                    // Nested quotes - weaken boundary
                    Some(SuppressionDecision::Weaken {
                        new_flags: BoundaryFlags::WEAK,
                    })
                } else {
                    None
                }
            }
            EnclosureType::SingleQuote => {
                if config.suppress_in_single_quotes {
                    Some(SuppressionDecision::Suppress {
                        reason: format!("Inside single quotes (depth: {depth})"),
                    })
                } else {
                    None
                }
            }
            EnclosureType::Parenthesis
            | EnclosureType::SquareBracket
            | EnclosureType::CurlyBrace => {
                // These enclosures typically allow sentence boundaries
                if !candidate.flags.is_strong {
                    // But suppress weak boundaries
                    Some(SuppressionDecision::Suppress {
                        reason: format!("Weak boundary inside {enclosure_type:?}"),
                    })
                } else {
                    // Strong boundaries are kept
                    None
                }
            }
            EnclosureType::JapaneseQuote | EnclosureType::JapaneseDoubleQuote => {
                // Japanese quotes typically suppress boundaries
                Some(SuppressionDecision::Suppress {
                    reason: format!("Inside Japanese quotes (depth: {depth})"),
                })
            }
            _ => {
                // Other enclosure types - check if they typically contain sentences
                if depth > 0 && candidate.flags.contains(BoundaryFlags::WEAK) {
                    Some(SuppressionDecision::Suppress {
                        reason: "Weak boundary inside enclosure".to_string(),
                    })
                } else {
                    None
                }
            }
        }
    }

    /// Infer enclosure type from type ID based on language rules
    fn infer_enclosure_type(
        type_id: usize,
        language_rules: &dyn LanguageRules,
    ) -> Option<EnclosureType> {
        // This is a simplified mapping - in practice, language rules
        // would provide this mapping
        match (language_rules.language_code(), type_id) {
            ("en", 0) => Some(EnclosureType::DoubleQuote),
            ("en", 1) => Some(EnclosureType::SingleQuote),
            ("en", 2) => Some(EnclosureType::Parenthesis),
            ("en", 3) => Some(EnclosureType::SquareBracket),
            ("en", 4) => Some(EnclosureType::CurlyBrace),
            ("ja", 0) => Some(EnclosureType::JapaneseQuote),
            ("ja", 1) => Some(EnclosureType::JapaneseDoubleQuote),
            ("ja", 2) => Some(EnclosureType::Parenthesis),
            ("ja", 3) => Some(EnclosureType::SquareBracket),
            _ => None,
        }
    }

    /// Check if quote pairing is valid
    pub fn validate_pairing(
        text: &str,
        _boundaries: &[BoundaryCandidate],
        language_rules: &dyn LanguageRules,
    ) -> Vec<QuotePairingIssue> {
        let mut issues = Vec::new();
        let mut quote_stack: Vec<(char, usize)> = Vec::new();

        for (i, ch) in text.chars().enumerate() {
            if let Some(enclosure) = language_rules.get_enclosure_char(ch) {
                match enclosure.enclosure_type {
                    EnclosureType::DoubleQuote | EnclosureType::SingleQuote => {
                        // For quotes that use the same character for open/close,
                        // we need to track state differently
                        if ch == '"' || ch == '\'' {
                            // Check if we have an open quote of the same type
                            if let Some(pos) = quote_stack.iter().rposition(|(c, _)| *c == ch) {
                                // Found matching open quote, this is a close
                                let _ = quote_stack.remove(pos);
                                // Already matching pair, no need to check
                            } else {
                                // No matching open quote, this is an open
                                quote_stack.push((ch, i));
                            }
                        } else {
                            // For different open/close characters
                            if enclosure.is_opening {
                                quote_stack.push((ch, i));
                            } else if let Some((open_char, open_pos)) = quote_stack.pop() {
                                if !Self::is_matching_quote_pair(open_char, ch) {
                                    issues.push(QuotePairingIssue::Mismatch {
                                        open_char,
                                        open_pos,
                                        close_char: ch,
                                        close_pos: i,
                                    });
                                }
                            } else {
                                issues.push(QuotePairingIssue::UnmatchedClose { char: ch, pos: i });
                            }
                        }
                    }
                    _ => {} // Other enclosure types handled separately
                }
            }
        }

        // Check for unclosed quotes
        for (ch, pos) in quote_stack {
            issues.push(QuotePairingIssue::UnmatchedOpen { char: ch, pos });
        }

        issues
    }

    /// Check if two characters form a matching quote pair
    fn is_matching_quote_pair(open: char, close: char) -> bool {
        matches!(
            (open, close),
            ('"', '"') | ('\'', '\'') | ('「', '」') | ('『', '』')
        )
    }
}

/// Issues found during quote pairing validation
#[derive(Debug, Clone, PartialEq)]
pub enum QuotePairingIssue {
    /// Opening quote without matching close
    UnmatchedOpen { char: char, pos: usize },
    /// Closing quote without matching open
    UnmatchedClose { char: char, pos: usize },
    /// Mismatched quote types
    Mismatch {
        open_char: char,
        open_pos: usize,
        close_char: char,
        close_pos: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{language::MockLanguageRules, types::DepthVec};

    #[test]
    fn test_basic_quote_suppression() {
        let config = QuoteSuppressionConfig::default();
        let rules = MockLanguageRules::english();
        let candidate = BoundaryCandidate {
            local_offset: 10,
            local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]), // Inside double quotes
            flags: BoundaryFlags::STRONG,
        };

        let context = QuoteSuppressionContext {
            candidate: &candidate,
            language_rules: &rules,
            enclosure_depths: &[1, 0, 0, 0, 0],
            config: &config,
        };

        let decision = QuoteSuppressor::evaluate(context);
        assert!(matches!(decision, SuppressionDecision::Suppress { .. }));
    }

    #[test]
    fn test_nested_quote_weakening() {
        let mut config = QuoteSuppressionConfig::default();
        config.suppress_in_double_quotes = false;
        config.max_nesting_level = 2;

        let rules = MockLanguageRules::english();
        let candidate = BoundaryCandidate {
            local_offset: 10,
            local_depths: DepthVec::from_vec(vec![2, 0, 0, 0, 0]), // Nested double quotes
            flags: BoundaryFlags::STRONG,
        };

        let context = QuoteSuppressionContext {
            candidate: &candidate,
            language_rules: &rules,
            enclosure_depths: &[2, 0, 0, 0, 0],
            config: &config,
        };

        let decision = QuoteSuppressor::evaluate(context);
        assert!(matches!(
            decision,
            SuppressionDecision::Weaken {
                new_flags: BoundaryFlags::WEAK
            }
        ));
    }

    #[test]
    fn test_parenthesis_allows_strong_boundaries() {
        let config = QuoteSuppressionConfig::default();
        let rules = MockLanguageRules::english();
        let candidate = BoundaryCandidate {
            local_offset: 10,
            local_depths: DepthVec::from_vec(vec![0, 0, 1, 0, 0]), // Inside parentheses
            flags: BoundaryFlags::STRONG,
        };

        let context = QuoteSuppressionContext {
            candidate: &candidate,
            language_rules: &rules,
            enclosure_depths: &[0, 0, 1, 0, 0],
            config: &config,
        };

        let decision = QuoteSuppressor::evaluate(context);
        assert_eq!(decision, SuppressionDecision::Keep);
    }

    #[test]
    fn test_quote_pairing_validation() {
        let text = "He said \"hello\" and then 'goodbye'.";
        let rules = MockLanguageRules::english();
        let boundaries = vec![];

        let issues = QuoteSuppressor::validate_pairing(text, &boundaries, &rules);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_unmatched_quotes() {
        let text = "He said \"hello and forgot to close.";
        let rules = MockLanguageRules::english();
        let boundaries = vec![];

        let issues = QuoteSuppressor::validate_pairing(text, &boundaries, &rules);
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0], QuotePairingIssue::UnmatchedOpen { .. }));
    }
}
