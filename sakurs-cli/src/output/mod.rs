//! Output formatting module

use anyhow::Result;

/// Trait for output formatters
pub trait OutputFormatter: Send + Sync {
    /// Format and output a single sentence
    fn format_sentence(&mut self, sentence: &str, offset: usize) -> Result<()>;

    /// Finalize output (e.g., close JSON array)
    fn finish(&mut self) -> Result<()>;
}

pub mod json;
pub mod markdown;
pub mod text;

pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
pub use text::TextFormatter;
