//! Application layer for orchestrating text processing
//!
//! This module provides the coordination layer between the pure domain logic
//! and external adapters, handling concerns like parallel processing, chunking,
//! and performance optimization.
//!
//! # Architecture
//!
//! The application layer implements the Use Case layer in hexagonal architecture,
//! orchestrating domain operations without containing business logic itself.
//! It handles:
//!
//! - Text chunking for efficient parallel processing
//! - Thread pool management and parallel execution
//! - Cross-chunk boundary resolution
//! - Performance optimization and caching
//!
//! # Example
//!
//! ```rust
//! use sakurs_core::application::{TextProcessor, ProcessorConfig};
//! use sakurs_core::domain::language::EnglishLanguageRules;
//! use std::sync::Arc;
//!
//! let rules = Arc::new(EnglishLanguageRules::new());
//! let processor = TextProcessor::new(rules);
//!
//! let text = "This is a long text. It will be processed in parallel.";
//! let result = processor.process_text(text).unwrap();
//! ```

pub mod chunking;
pub mod config;
pub mod processor;
pub mod unified_processor;

#[cfg(feature = "parallel")]
pub mod parallel;

pub use chunking::{ChunkManager, TextChunk};
pub use config::{ProcessingError, ProcessingMetrics, ProcessorConfig};
pub use processor::{ProcessingOutput, TextProcessor};
pub use unified_processor::{UnifiedProcessingOutput, UnifiedProcessor};

#[cfg(feature = "parallel")]
pub use parallel::ParallelProcessor;
