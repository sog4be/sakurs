//! Domain layer for the Delta-Stack Monoid algorithm
//!
//! This module contains the mathematical foundations and language-specific
//! logic for parallel sentence boundary detection using monoid structures.

pub mod adapters;
pub mod cross_chunk;
pub mod enclosure;
pub mod enclosure_suppressor;
pub mod language;
pub mod monoid;
pub mod prefix_sum;
pub mod quote_suppression;
pub mod reduce;
pub mod traits;
pub mod types;

// Re-export adapter
pub use adapters::LanguageRulesAdapter;

// Re-export from other modules
pub use enclosure::*;
pub use monoid::*;
pub use prefix_sum::*;
pub use reduce::*;
pub use types::*;

// Re-export language module (contains original BoundaryContext, BoundaryDecision)
pub use language::*;

// Re-export new traits with aliases to avoid conflicts
pub use traits::{
    BoundaryAnalyzer,
    BoundaryCandidateInfo,
    // Use aliases for conflicting types
    BoundaryContext as TraitBoundaryContext,
    BoundaryDecision as TraitBoundaryDecision,
    BoundaryMarkerType,
    CharacterClass,
    CharacterClassifier,
    LanguageSpecificRules,
    QuoteBehavior,
    QuoteType,
    RejectionReason,
};
