use crate::domain::language::traits::{BoundaryContext, BoundaryDecision};
use crate::domain::BoundaryFlags;
use regex::Regex;
use std::collections::HashSet;

/// Context condition for ellipsis boundary determination
#[derive(Debug, Clone, PartialEq)]
pub enum ContextCondition {
    /// Followed by a capital letter
    FollowedByCapital,
    /// Followed by a lowercase letter
    FollowedByLowercase,
    /// Custom condition (future extension)
    Custom(String),
}

/// Rule for context-based ellipsis handling
#[derive(Debug, Clone)]
pub struct ContextRule {
    pub condition: ContextCondition,
    pub is_boundary: bool,
}

/// Exception pattern for ellipsis handling
#[derive(Debug)]
pub struct ExceptionPattern {
    pub regex: Regex,
    pub is_boundary: bool,
}

/// Rules for handling ellipsis patterns
#[derive(Debug)]
pub struct EllipsisRules {
    /// Whether to treat ellipsis as boundary by default
    treat_as_boundary: bool,
    /// Recognized ellipsis patterns
    patterns: HashSet<String>,
    /// Context-based rules
    context_rules: Vec<ContextRule>,
    /// Exception patterns
    exception_patterns: Vec<ExceptionPattern>,
}

impl EllipsisRules {
    /// Create new ellipsis rules
    pub fn new(
        treat_as_boundary: bool,
        patterns: Vec<String>,
        context_rules: Vec<(String, bool)>,
        exceptions: Vec<(String, bool)>,
    ) -> Result<Self, String> {
        // Parse context rules
        let parsed_context_rules: Vec<ContextRule> = context_rules
            .into_iter()
            .map(|(condition_str, is_boundary)| {
                let condition = match condition_str.as_str() {
                    "followed_by_capital" => ContextCondition::FollowedByCapital,
                    "followed_by_lowercase" => ContextCondition::FollowedByLowercase,
                    other => ContextCondition::Custom(other.to_string()),
                };
                ContextRule {
                    condition,
                    is_boundary,
                }
            })
            .collect();

        // Parse exception patterns
        let mut parsed_exceptions = Vec::new();
        for (pattern_str, is_boundary) in exceptions {
            let regex = Regex::new(&pattern_str)
                .map_err(|e| format!("Invalid exception regex '{pattern_str}': {e}"))?;
            parsed_exceptions.push(ExceptionPattern { regex, is_boundary });
        }

        Ok(Self {
            treat_as_boundary,
            patterns: patterns.into_iter().collect(),
            context_rules: parsed_context_rules,
            exception_patterns: parsed_exceptions,
        })
    }

    /// Check if the given position contains an ellipsis pattern
    pub fn is_ellipsis_pattern(&self, context: &BoundaryContext) -> bool {
        // Check each pattern
        for pattern in &self.patterns {
            if self.matches_pattern_at_position(context.text, context.position, pattern) {
                return true;
            }
        }
        false
    }

    /// Check if a specific pattern matches at the position
    fn matches_pattern_at_position(&self, text: &str, position: usize, pattern: &str) -> bool {
        let pattern_len = pattern.len();

        // For patterns like "...", we need to check if the current position
        // is part of the pattern or at the end of it

        // Check various possible positions where this character could be part of the pattern
        for offset in 0..pattern_len {
            // Calculate where the pattern would start if current position is at 'offset' within it
            if position >= offset {
                let start = position - offset;
                let end = start + pattern_len;

                if end <= text.len()
                    && text.is_char_boundary(start)
                    && text.is_char_boundary(end)
                    && &text[start..end] == pattern
                {
                    // We're part of this pattern
                    // Only consider it a complete pattern if we're at the last character
                    return offset == pattern_len - 1;
                }
            }
        }

        false
    }

    /// Evaluate whether an ellipsis should be a boundary
    pub fn evaluate_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        // First, check exception patterns
        if let Some(decision) = self.check_exceptions(context) {
            return decision;
        }

        // Apply context rules
        for rule in &self.context_rules {
            if let Some(decision) = self.apply_context_rule(context, rule) {
                return decision;
            }
        }

        // Default behavior
        if self.treat_as_boundary {
            BoundaryDecision::Boundary(BoundaryFlags::WEAK)
        } else {
            BoundaryDecision::NotBoundary
        }
    }

    /// Check exception patterns
    fn check_exceptions(&self, context: &BoundaryContext) -> Option<BoundaryDecision> {
        // Create a window around the position for regex matching
        let start = context.position.saturating_sub(20);
        let end = (context.position + 20).min(context.text.len());
        let window = &context.text[start..end];

        for exception in &self.exception_patterns {
            if exception.regex.is_match(window) {
                return Some(if exception.is_boundary {
                    BoundaryDecision::Boundary(BoundaryFlags::WEAK)
                } else {
                    BoundaryDecision::NotBoundary
                });
            }
        }

        None
    }

    /// Apply a context rule
    fn apply_context_rule(
        &self,
        context: &BoundaryContext,
        rule: &ContextRule,
    ) -> Option<BoundaryDecision> {
        let matches = match &rule.condition {
            ContextCondition::FollowedByCapital => self.is_followed_by_capital(context),
            ContextCondition::FollowedByLowercase => self.is_followed_by_lowercase(context),
            ContextCondition::Custom(_) => {
                // Custom conditions not implemented yet
                false
            }
        };

        if matches {
            Some(if rule.is_boundary {
                BoundaryDecision::Boundary(BoundaryFlags::WEAK)
            } else {
                BoundaryDecision::NotBoundary
            })
        } else {
            None
        }
    }

    /// Check if ellipsis is followed by a capital letter
    fn is_followed_by_capital(&self, context: &BoundaryContext) -> bool {
        context
            .following_context
            .chars()
            .find(|&ch| ch.is_alphabetic())
            .map(|ch| ch.is_uppercase())
            .unwrap_or(false)
    }

    /// Check if ellipsis is followed by a lowercase letter
    fn is_followed_by_lowercase(&self, context: &BoundaryContext) -> bool {
        context
            .following_context
            .chars()
            .find(|&ch| ch.is_alphabetic())
            .map(|ch| ch.is_lowercase())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsis_pattern_detection() {
        let rules = EllipsisRules::new(
            true,
            vec!["...".to_string(), "…".to_string()],
            vec![],
            vec![],
        )
        .unwrap();

        let context = BoundaryContext {
            text: "Hello... World".to_string(),
            position: 7, // After the last dot
            boundary_char: '.',
            preceding_context: "Hello..".to_string(),
            following_context: " World".to_string(),
        };

        assert!(rules.is_ellipsis_pattern(&context));
    }

    #[test]
    fn test_context_rules() {
        let rules = EllipsisRules::new(
            true,
            vec!["...".to_string()],
            vec![
                ("followed_by_capital".to_string(), true),
                ("followed_by_lowercase".to_string(), false),
            ],
            vec![],
        )
        .unwrap();

        // Test followed by capital
        let context = BoundaryContext {
            text: "Wait... Then he left.".to_string(),
            position: 7,
            boundary_char: '.',
            preceding_context: "Wait..".to_string(),
            following_context: " Then he left.".to_string(),
        };

        match rules.evaluate_boundary(&context) {
            BoundaryDecision::Boundary(_) => {}
            _ => panic!("Expected boundary when followed by capital"),
        }

        // Test followed by lowercase
        let context = BoundaryContext {
            text: "Wait... then he left.".to_string(),
            position: 7,
            boundary_char: '.',
            preceding_context: "Wait..".to_string(),
            following_context: " then he left.".to_string(),
        };

        assert_eq!(
            rules.evaluate_boundary(&context),
            BoundaryDecision::NotBoundary
        );
    }

    #[test]
    fn test_exception_patterns() {
        let rules = EllipsisRules::new(
            true,
            vec!["...".to_string()],
            vec![],
            vec![(r"\b(um|uh|er)\.\.\.".to_string(), false)],
        )
        .unwrap();

        let context = BoundaryContext {
            text: "He said um... I think so.".to_string(),
            position: 12, // After "um..."
            boundary_char: '.',
            preceding_context: "He said um..".to_string(),
            following_context: " I think so.".to_string(),
        };

        assert_eq!(
            rules.evaluate_boundary(&context),
            BoundaryDecision::NotBoundary
        );
    }

    #[test]
    fn test_default_behavior() {
        // Test treat_as_boundary = true
        let rules = EllipsisRules::new(true, vec!["...".to_string()], vec![], vec![]).unwrap();

        let context = BoundaryContext {
            text: "Hello...".to_string(),
            position: 7,
            boundary_char: '.',
            preceding_context: "Hello..".to_string(),
            following_context: "".to_string(),
        };

        match rules.evaluate_boundary(&context) {
            BoundaryDecision::Boundary(flags) => assert_eq!(flags, BoundaryFlags::WEAK),
            _ => panic!("Expected weak boundary"),
        }

        // Test treat_as_boundary = false
        let rules = EllipsisRules::new(false, vec!["...".to_string()], vec![], vec![]).unwrap();

        assert_eq!(
            rules.evaluate_boundary(&context),
            BoundaryDecision::NotBoundary
        );
    }
}
