//! Public API for Sakurs sentence boundary detection
//!
//! This crate provides a clean, stable interface for sentence segmentation
//! that hides internal implementation details.

#![warn(missing_docs)]

pub mod config;
pub mod dto;
pub mod error;

use dto::{BoundaryDTO, ExecutionMode, Language, Metadata};
use error::Result;
use std::sync::Arc;

// Re-export key types
pub use config::{Config, ConfigBuilder};
pub use dto::{Boundary, Input, Output};
pub use error::ApiError;

/// Main entry point for sentence boundary detection
///
/// This struct provides a stable public API that abstracts away internal
/// implementation details and engine-specific types.
pub struct SentenceProcessor {
    inner: Arc<sakurs_engine::SentenceProcessor>,
    config: Config,
}

impl SentenceProcessor {
    /// Create a new processor with default configuration (English, adaptive mode)
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }

    /// Create a new processor with specific language
    pub fn with_language(lang_code: &str) -> Result<Self> {
        let config = Config::builder().language(lang_code)?.build()?;
        Self::with_config(config)
    }

    /// Create a new processor with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        // Create processor config from API config
        let mut proc_config_builder =
            sakurs_engine::SentenceProcessorBuilder::new().language(config.language.code());

        // Set execution mode
        proc_config_builder = proc_config_builder.execution_mode(config.execution_mode.into());

        // Set thread count if specified
        proc_config_builder = proc_config_builder.threads(config.threads);

        // Set adaptive threshold if specified
        if let Some(threshold_kb) = config.adaptive_threshold_kb {
            proc_config_builder = proc_config_builder.adaptive_threshold_kb(threshold_kb);
        }

        // Build the processor
        let inner = proc_config_builder
            .build()
            .map_err(|e| ApiError::Engine(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(inner),
            config,
        })
    }

    /// Process input and return sentence boundaries
    pub fn process(&self, input: Input) -> Result<Output> {
        let start = std::time::Instant::now();

        // Read text from input
        let text = input.read_text()?;
        let text_len = text.len();
        let char_count = text.chars().count();

        // Convert to engine input
        let engine_input = sakurs_engine::Input::from_text(text);

        // Process with engine
        let engine_output = self
            .inner
            .process(engine_input)
            .map_err(|e| ApiError::Engine(e.to_string()))?;

        let elapsed = start.elapsed();

        // Convert engine output to API DTOs
        let boundaries = engine_output
            .boundaries
            .into_iter()
            .map(|b| BoundaryDTO {
                byte_offset: b.byte_offset,
                char_offset: b.char_offset,
                kind: format!("{:?}", b.kind),
            })
            .collect();

        // Create metadata
        let metadata = Metadata {
            total_bytes: text_len,
            total_chars: char_count,
            processing_time_ms: elapsed.as_millis() as u64,
            throughput_mbps: (text_len as f64 / 1_048_576.0) / elapsed.as_secs_f64(),
            mode_used: format!("{:?}", engine_output.metadata.execution_mode),
            thread_count: 1, // TODO: Get actual thread count from engine
            chunk_size: engine_output
                .metadata
                .chunks_processed
                .filter(|&count| count > 1)
                .map(|count| text_len / count),
        };

        Ok(Output {
            boundaries,
            metadata,
        })
    }

    /// Process with automatic mode selection (adaptive)
    ///
    /// This method explicitly uses adaptive mode to select between
    /// sequential and parallel processing based on input size.
    pub fn process_auto(&self, input: Input) -> Result<Output> {
        // For now, just delegate to process() since we default to adaptive
        self.process(input)
    }

    /// Process with explicit execution mode
    pub fn process_with_mode(&self, input: Input, mode: ExecutionMode) -> Result<Output> {
        // Create a temporary config with the specified mode
        let mut temp_config = self.config.clone();
        temp_config.execution_mode = mode;

        // Create a new processor with the temporary config
        let processor = Self::with_config(temp_config)?;
        processor.process(input)
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the configured language
    pub fn language(&self) -> Language {
        self.config.language.clone()
    }

    /// Process text directly (convenience method)
    pub fn process_text(&self, text: &str) -> Result<Output> {
        self.process(Input::from_text(text))
    }
}

impl Default for SentenceProcessor {
    fn default() -> Self {
        Self::new().expect("default processor creation should not fail")
    }
}

// Convenience functions

/// Process text with default configuration
pub fn process_text(text: &str) -> Result<Output> {
    let processor = SentenceProcessor::new()?;
    processor.process(Input::from_text(text))
}

/// Process a file with default configuration
pub fn process_file<P: AsRef<std::path::Path>>(path: P) -> Result<Output> {
    let processor = SentenceProcessor::new()?;
    processor.process(Input::from_file(path.as_ref().to_path_buf()))
}

/// Process text with a specific language
pub fn process_text_with_language(text: &str, lang_code: &str) -> Result<Output> {
    let processor = SentenceProcessor::with_language(lang_code)?;
    processor.process(Input::from_text(text))
}

/// Process text with a given processor (compatibility function)
pub fn process_with_processor(processor: &SentenceProcessor, text: &str) -> Result<Output> {
    processor.process(Input::from_text(text))
}
