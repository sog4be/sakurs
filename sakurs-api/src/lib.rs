//! Public API for Sakurs sentence boundary detection
//!
//! This crate provides a clean, stable interface for sentence segmentation
//! that hides internal implementation details.

#![warn(missing_docs)]

pub mod config;
pub mod dto;
pub mod error;

use dto::ProcessingMetadata;
use error::Result;
use std::path::Path;

// Re-export key types
pub use config::{Config, ConfigBuilder};
pub use dto::{Boundary, Input, Output};
pub use error::ApiError;

// Re-export from engine
pub use sakurs_engine::{Language, SentenceProcessor};

/// Process text with a processor and return Output
pub fn process_with_processor(processor: &SentenceProcessor, text: &str) -> Result<Output> {
    let start = std::time::Instant::now();
    let text_len = text.len();

    let output = processor
        .process(sakurs_engine::Input::from_text(text.to_string()))
        .map_err(|e| crate::error::ApiError::Engine(e.to_string()))?;

    let elapsed = start.elapsed();

    // Convert to DTOs
    let boundary_dtos = output
        .boundaries
        .into_iter()
        .map(|b| Boundary {
            byte_offset: b.byte_offset,
            char_offset: b.char_offset,
            kind: format!("{:?}", b.kind),
        })
        .collect();

    Ok(Output {
        boundaries: boundary_dtos,
        metadata: ProcessingMetadata::new(
            text_len,
            text.chars().count(),
            elapsed.as_millis() as u64,
            "auto".to_string(),
            1, // TODO: Get from execution
        ),
    })
}

// Convenience functions

/// Process text with default configuration
pub fn process_text(text: &str) -> Result<Output> {
    let processor =
        SentenceProcessor::new().map_err(|e| crate::error::ApiError::Engine(e.to_string()))?;

    let input = crate::dto::Input::Text(text.to_string());
    process_input(processor, input)
}

/// Process a file with default configuration
pub fn process_file<P: AsRef<Path>>(path: P) -> Result<Output> {
    let processor =
        SentenceProcessor::new().map_err(|e| crate::error::ApiError::Engine(e.to_string()))?;

    let input = crate::dto::Input::File(path.as_ref().to_path_buf());
    process_input(processor, input)
}

/// Process input with a specific processor
fn process_input(processor: SentenceProcessor, input: crate::dto::Input) -> Result<Output> {
    let start = std::time::Instant::now();

    let text = input.read_text()?;
    let text_len = text.len();
    let char_count = text.chars().count();

    let output = processor
        .process(sakurs_engine::Input::from_text(text))
        .map_err(|e| crate::error::ApiError::Engine(e.to_string()))?;

    let elapsed = start.elapsed();

    // Convert to DTOs
    let boundary_dtos = output
        .boundaries
        .into_iter()
        .map(|b| Boundary {
            byte_offset: b.byte_offset,
            char_offset: b.char_offset,
            kind: format!("{:?}", b.kind),
        })
        .collect();

    Ok(Output {
        boundaries: boundary_dtos,
        metadata: ProcessingMetadata::new(
            text_len,
            char_count,
            elapsed.as_millis() as u64,
            "auto".to_string(),
            1, // TODO: Get from execution
        ),
    })
}
