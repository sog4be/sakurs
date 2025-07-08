//! Pure domain traits for sentence boundary detection
//!
//! This module contains trait definitions that represent pure business logic
//! without any execution or infrastructure concerns.

mod adapters;
mod boundary_analyzer;
mod character_classifier;
mod language_specific_rules;

pub use adapters::LanguageRulesAdapter;
pub use boundary_analyzer::{
    BoundaryAnalyzer, BoundaryCandidateInfo, BoundaryContext as TraitBoundaryContext,
    BoundaryDecision as TraitBoundaryDecision, BoundaryMarkerType, RejectionReason,
};
pub use character_classifier::{CharacterClass, CharacterClassifier};
pub use language_specific_rules::{LanguageSpecificRules, QuoteBehavior, QuoteType};

#[cfg(test)]
mod tests;
