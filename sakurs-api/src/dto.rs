//! Data Transfer Objects for API

use crate::error::{ApiError, Result};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

/// Input source for processing
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Input {
    /// Raw text string
    Text(String),
    /// File path
    File(PathBuf),
    /// Raw bytes (UTF-8)
    Bytes(Vec<u8>),
    /// Reader (not serializable)
    #[cfg_attr(feature = "serde", serde(skip))]
    Reader(Box<dyn Read>),
}

impl std::fmt::Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::Text(text) => f.debug_tuple("Text").field(text).finish(),
            Input::File(path) => f.debug_tuple("File").field(path).finish(),
            Input::Bytes(bytes) => f.debug_tuple("Bytes").field(&bytes.len()).finish(),
            Input::Reader(_) => f.debug_tuple("Reader").field(&"<dyn Read>").finish(),
        }
    }
}

impl Input {
    /// Read the text content from the input
    pub fn read_text(self) -> Result<String> {
        match self {
            Input::Text(text) => Ok(text),
            Input::File(path) => fs::read_to_string(&path).map_err(ApiError::Io),
            Input::Bytes(bytes) => String::from_utf8(bytes).map_err(ApiError::Utf8),
            Input::Reader(mut reader) => {
                let mut buffer = String::new();
                reader.read_to_string(&mut buffer).map_err(ApiError::Io)?;
                Ok(buffer)
            }
        }
    }
}

/// Boundary information for serialization
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Boundary {
    /// Byte offset in the text
    pub byte_offset: usize,
    /// Character offset in the text
    pub char_offset: usize,
    /// Type of boundary (string representation)
    pub kind: String,
}

/// Processing metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessingMetadata {
    /// Total bytes processed
    pub total_bytes: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Execution mode used
    pub mode: String,
    /// Number of threads used
    pub thread_count: usize,
}

/// Complete output with boundaries and metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Output {
    /// Detected boundaries
    pub boundaries: Vec<Boundary>,
    /// Processing metadata
    pub metadata: ProcessingMetadata,
}
