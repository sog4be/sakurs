//! Application orchestration for sentence boundary detection
//!
//! This crate provides execution strategies, chunking, and coordination
//! logic for the Delta-Stack Monoid algorithm.

#![warn(missing_docs)]

pub mod adaptive_dispatcher;
pub mod assembler;
pub mod chunker;
pub mod config;
pub mod error;
pub mod executor;
pub mod input;
pub mod language;
pub mod processor;
pub mod processor_config;

// Re-export key types
pub use adaptive_dispatcher::AdaptiveDispatcher;
pub use assembler::ResultAssembler;
pub use config::{ChunkPolicy, EngineConfig};
pub use error::{ApiError, ApiResult, EngineError, Result};
pub use executor::{ExecutionMetrics, ExecutionMode, Executor, ProcessingOutput};
pub use input::{Input, PerformanceHints, ProcessingPattern};
pub use language::LanguageRulesImpl;
pub use processor::{Output, ProcessingMetadata, SentenceProcessor, SentenceProcessorBuilder};
pub use processor_config::{ProcessorConfig, ProcessorConfigBuilder};

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
