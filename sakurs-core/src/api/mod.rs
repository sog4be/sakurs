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

pub use crate::domain::language::config::LanguageConfig;

/// The language configuration schema (the TOML file structure), for
/// constructing configurations programmatically (used by the bindings).
pub mod language_config {
    pub use crate::domain::language::config::{
        AbbreviationConfig, ContextRule, EllipsisConfig, EnclosureConfig, EnclosurePair,
        ExceptionPattern, FastPattern, LanguageConfig, MetadataConfig, RegexPattern,
        SentenceStarterConfig, SuppressionConfig, TerminatorConfig, TerminatorPattern,
    };
}
pub use config::{Config, ConfigBuilder};
pub use error::{Error, Result};
pub use input::Input;
pub use language::Language;
pub use output::{Boundary, Output, ProcessingMetadata, ProcessingStats};
pub use processor::SentenceProcessor;
