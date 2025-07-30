//! Application orchestration for sentence boundary detection
//!
//! This crate provides execution strategies, chunking, and coordination
//! logic for the Delta-Stack Monoid algorithm.

#![warn(missing_docs)]

pub mod assembler;
pub mod chunker;
pub mod config;
pub mod error;
pub mod executor;
pub mod language;
pub mod processor;

// Re-export key types
pub use assembler::ResultAssembler;
pub use config::{ChunkPolicy, EngineConfig};
pub use error::{EngineError, Result};
pub use executor::{ExecutionMode, Executor};
pub use language::LanguageRulesImpl;
pub use processor::{SentenceProcessor, SentenceProcessorBuilder};

// Re-export from core for convenience
pub use sakurs_core::{Boundary, BoundaryKind, LanguageRules};

/// Language identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    /// English
    English,
    /// Japanese
    Japanese,
}

impl Language {
    /// Get language code
    pub fn code(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Japanese => "ja",
        }
    }
}
