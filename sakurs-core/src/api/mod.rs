//! New unified API for sakurs-core
//!
//! This module provides a clean, intuitive interface for sentence segmentation
//! that hides internal implementation details and provides a consistent API
//! for both CLI and Python bindings.

mod config;
mod error;
mod input;
mod language;
mod output;
mod processor;

#[cfg(test)]
mod tests;

pub use config::{Config, ConfigBuilder};
pub use error::{Error, Result};
pub use input::Input;
pub use language::Language;
pub use output::{Boundary, BoundaryContext, Output, ProcessingMetadata, ProcessingStats};
pub use processor::SentenceProcessor;
