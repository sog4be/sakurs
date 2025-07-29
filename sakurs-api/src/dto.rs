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
    /// Create input from text
    pub fn from_text(text: impl Into<String>) -> Self {
        Input::Text(text.into())
    }

    /// Create input from file path
    pub fn from_file(path: impl Into<PathBuf>) -> Self {
        Input::File(path.into())
    }

    /// Create input from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Input::Bytes(bytes)
    }

    /// Create input from a reader
    pub fn from_reader<R: Read + 'static>(reader: R) -> Self {
        Input::Reader(Box::new(reader))
    }

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

/// Boundary information for serialization (FFI-safe DTO)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoundaryDTO {
    /// Byte offset in the text
    pub byte_offset: usize,
    /// Character offset in the text
    pub char_offset: usize,
    /// Type of boundary (string representation)
    pub kind: String,
}

impl BoundaryDTO {
    /// Create a new boundary DTO
    pub fn new(byte_offset: usize, char_offset: usize, kind: String) -> Self {
        Self {
            byte_offset,
            char_offset,
            kind,
        }
    }
}

/// Type alias for backward compatibility
pub type Boundary = BoundaryDTO;

/// Processing metadata with runtime statistics
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metadata {
    /// Total bytes processed
    pub total_bytes: usize,
    /// Total characters processed
    pub total_chars: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Throughput in MB/s
    pub throughput_mbps: f64,
    /// Execution mode used
    pub mode_used: String,
    /// Number of threads used
    pub thread_count: usize,
    /// Chunk size in bytes (if applicable)
    pub chunk_size: Option<usize>,
}

impl Metadata {
    /// Create new metadata
    pub fn new(
        total_bytes: usize,
        total_chars: usize,
        processing_time_ms: u64,
        mode_used: String,
        thread_count: usize,
    ) -> Self {
        let throughput_mbps = if processing_time_ms > 0 {
            (total_bytes as f64 / 1_048_576.0) / (processing_time_ms as f64 / 1000.0)
        } else {
            0.0
        };

        Self {
            total_bytes,
            total_chars,
            processing_time_ms,
            throughput_mbps,
            mode_used,
            thread_count,
            chunk_size: None,
        }
    }
}

/// Type alias for backward compatibility
pub type ProcessingMetadata = Metadata;

/// Complete output with boundaries and metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Output {
    /// Detected boundaries
    pub boundaries: Vec<BoundaryDTO>,
    /// Processing metadata
    pub metadata: Metadata,
}
