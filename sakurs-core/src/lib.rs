//! Delta-Stack Monoid algorithm for parallel sentence boundary detection
//!
//! This crate implements a mathematically sound parallel approach to sentence
//! boundary detection using monoid algebra. The core innovation lies in
//! representing parsing state as a monoid, enabling associative operations
//! that can be computed in parallel while maintaining perfect accuracy.
//!
//! # Stability Notice
//!
//! This crate is pre-1.0. The 0.2 series is the first pass at a stable
//! public surface: the API is intentionally small (the [`api`] module,
//! re-exported at the crate root) so that internal improvements no longer
//! require breaking changes. Pin a minor version in your Cargo.toml:
//! ```toml
//! sakurs-core = "0.2"
//! ```
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
//! ```

pub mod api;
pub(crate) mod application;
pub(crate) mod domain;

pub use api::{
    Boundary, Config, ConfigBuilder, Error as ApiError, Input, Language, LanguageConfig, Output,
    ProcessingMetadata, ProcessingStats, SentenceProcessor,
};
