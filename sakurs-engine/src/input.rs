//! Input abstraction for sentence processing
//!
//! Provides a unified interface for processing text from various sources
//! as specified in DESIGN.md Section 3.3.

use crate::error::{EngineError, Result};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Unified input abstraction
///
/// Supports various input sources while providing a consistent interface
/// for text processing.
pub enum Input {
    /// Direct text string
    Text(String),
    /// Static text reference (zero-copy for string literals)
    TextRef(&'static str),
    /// File path to read from
    File(PathBuf),
    /// Bytes to process as UTF-8 text
    Bytes(Vec<u8>),
    /// Reader stream (for stdin, network, etc.)
    Reader(Box<dyn Read + Send>),
}

impl std::fmt::Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::Text(text) => f.debug_tuple("Text").field(text).finish(),
            Input::TextRef(text) => f.debug_tuple("TextRef").field(text).finish(),
            Input::File(path) => f.debug_tuple("File").field(path).finish(),
            Input::Bytes(bytes) => f
                .debug_tuple("Bytes")
                .field(&format!("<{} bytes>", bytes.len()))
                .finish(),
            Input::Reader(_) => f.debug_tuple("Reader").field(&"<Reader>").finish(),
        }
    }
}

impl Input {
    /// Create input from a text string
    pub fn from_text<S: Into<String>>(text: S) -> Self {
        Input::Text(text.into())
    }

    /// Create input from a static string reference (zero-copy)
    pub fn from_text_ref(text: &'static str) -> Self {
        Input::TextRef(text)
    }

    /// Create input from a file path
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Self {
        Input::File(path.into())
    }

    /// Create input from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Input::Bytes(bytes)
    }

    /// Create input from a reader
    pub fn from_reader<R: Read + Send + 'static>(reader: R) -> Self {
        Input::Reader(Box::new(reader))
    }

    /// Convert the input to a text string
    ///
    /// This method handles reading from files, converting bytes to UTF-8,
    /// and reading from streams as needed.
    pub fn to_text(self) -> Result<String> {
        match self {
            Input::Text(text) => Ok(text),
            Input::TextRef(text) => Ok(text.to_string()),
            Input::File(path) => fs::read_to_string(&path)
                .map_err(|e| EngineError::IoError(format!("Failed to read file {path:?}: {e}"))),
            Input::Bytes(bytes) => String::from_utf8(bytes)
                .map_err(|e| EngineError::EncodingError(format!("Invalid UTF-8: {e}"))),
            Input::Reader(mut reader) => {
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).map_err(|e| {
                    EngineError::IoError(format!("Failed to read from stream: {e}"))
                })?;
                String::from_utf8(buffer).map_err(|e| {
                    EngineError::EncodingError(format!("Invalid UTF-8 from stream: {e}"))
                })
            }
        }
    }

    /// Get the estimated size of the input (if available)
    ///
    /// This is used for adaptive mode selection. Returns None if the size
    /// cannot be determined without reading the entire input.
    pub fn estimated_size(&self) -> Option<usize> {
        match self {
            Input::Text(text) => Some(text.len()),
            Input::TextRef(text) => Some(text.len()),
            Input::Bytes(bytes) => Some(bytes.len()),
            Input::File(path) => {
                // Try to get file metadata for size estimation
                fs::metadata(path).ok().map(|m| m.len() as usize)
            }
            Input::Reader(_) => None, // Cannot determine size without reading
        }
    }
}

impl From<String> for Input {
    fn from(text: String) -> Self {
        Input::Text(text)
    }
}

impl From<&'static str> for Input {
    fn from(text: &'static str) -> Self {
        Input::TextRef(text)
    }
}

impl From<PathBuf> for Input {
    fn from(path: PathBuf) -> Self {
        Input::File(path)
    }
}

impl From<Vec<u8>> for Input {
    fn from(bytes: Vec<u8>) -> Self {
        Input::Bytes(bytes)
    }
}

impl Clone for Input {
    fn clone(&self) -> Self {
        match self {
            Input::Text(text) => Input::Text(text.clone()),
            Input::TextRef(text) => Input::TextRef(text),
            Input::File(path) => Input::File(path.clone()),
            Input::Bytes(bytes) => Input::Bytes(bytes.clone()),
            Input::Reader(_) => panic!("Cannot clone Reader input"),
        }
    }
}

/// Performance hints for processing optimization
#[derive(Debug, Clone, Default)]
pub struct PerformanceHints {
    /// Prefer speed over accuracy
    pub prefer_speed: bool,
    /// Expected input size (for adaptive mode tuning)
    pub expected_size: Option<usize>,
    /// Expected processing pattern (batch vs streaming)
    pub processing_pattern: ProcessingPattern,
}

/// Expected processing pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessingPattern {
    /// Single large document
    #[default]
    Batch,
    /// Many small documents
    Interactive,
    /// Continuous stream of text
    Streaming,
}
