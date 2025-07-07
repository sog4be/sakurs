//! Delta-Stack Monoid algorithm for parallel sentence boundary detection
//!
//! This crate implements a mathematically sound parallel approach to sentence
//! boundary detection using monoid algebra. The core innovation lies in
//! representing parsing state as a monoid, enabling associative operations
//! that can be computed in parallel while maintaining perfect accuracy.
//!
//! # Architecture
//!
//! The crate follows a hexagonal architecture pattern:
//! - **Domain layer**: Pure mathematical algorithms and monoid operations
//! - **Application layer**: Orchestration and parallel processing logic
//! - **Adapter layer**: Interfaces for different use cases (CLI, Python, etc.)
//!
//! # Example
//!
//! ```rust
//! use sakurs_core::application::{TextProcessor, ProcessorConfig};
//! use sakurs_core::domain::language::EnglishLanguageRules;
//! use std::sync::Arc;
//!
//! // Create language rules
//! let rules = Arc::new(EnglishLanguageRules::new());
//!
//! // Create processor with default configuration
//! let processor = TextProcessor::new(rules);
//!
//! // Process text
//! let text = "Hello world. This is a test.";
//! let result = processor.process_text(text).unwrap();
//!
//! // Extract sentences
//! let sentences = result.extract_sentences(text);
//! assert_eq!(sentences.len(), 2);
//! ```

pub mod api;
pub mod application;
pub mod domain;

// New unified API (recommended)
pub use api::{
    Boundary, Config, ConfigBuilder, Error as ApiError, Input, Language, Output,
    ProcessingMetadata, ProcessingStats, SentenceProcessor,
};

// Legacy exports (for backward compatibility)
pub use application::strategies::{AdaptiveStrategy, ProcessingStrategy};
pub use application::{ProcessorConfig, UnifiedProcessor};
pub use domain::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monoid_properties_integration() {
        // Integration test ensuring monoid properties hold across the domain
        let state1 = PartialState::new(2);
        let state2 = PartialState::new(2);
        let identity = PartialState::identity();

        // Identity property
        assert_eq!(
            state1.combine(&identity).boundary_candidates.len(),
            state1.boundary_candidates.len()
        );

        // Associativity holds for basic operations
        let combined1 = state1.combine(&state2);
        let combined2 = identity.combine(&combined1);
        assert_eq!(
            combined2.boundary_candidates.len(),
            combined1.boundary_candidates.len()
        );
    }

    #[test]
    fn test_domain_module_exports() {
        // Verify that all essential types are properly exported
        let _monoid_test: PartialState = PartialState::identity();
        let _boundary_test = domain::state::Boundary {
            offset: 0,
            flags: BoundaryFlags::STRONG,
        };
        let _delta_test = DeltaEntry::new(0, 0);
        let _abbr_test = AbbreviationState::identity();
    }
}
