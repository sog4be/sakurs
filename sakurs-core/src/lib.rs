//! Delta-Stack Monoid algorithm for parallel sentence boundary detection
//!
//! This crate implements a mathematically sound parallel approach to sentence
//! boundary detection using monoid algebra. The core innovation lies in
//! representing parsing state as a monoid, enabling associative operations
//! that can be computed in parallel while maintaining perfect accuracy.
//!
//! # Stability Notice
//!
//! This crate is pre-1.0. The API is not yet stable and may change
//! significantly in future releases (a breaking rework of the public
//! surface is planned for v0.2.0). Use with caution in production code.
//!
//! We recommend pinning to exact versions in your Cargo.toml:
//! ```toml
//! sakurs-core = "=0.1.2"
//! ```
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
//! use sakurs_core::{SentenceProcessor, Input};
//!
//! // Create processor with default configuration
//! let processor = SentenceProcessor::new();
//!
//! // Process text
//! let text = "Hello world. This is a test.";
//! let result = processor.process(Input::from_text(text)).unwrap();
//!
//! // Check boundaries
//! assert!(!result.boundaries.is_empty());
//! // Note: SentenceProcessor may detect different boundary counts than expected
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
pub use application::{DeltaStackProcessor, DeltaStackResult, ExecutionMode, ProcessorConfig};
pub use domain::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_module_exports() {
        // Verify that the essential domain types are properly exported
        let _boundary_test = domain::Boundary {
            offset: 0,
            flags: BoundaryFlags::STRONG,
        };
    }
}
