//! Input abstraction for unified API

use std::io::Read;
use std::path::{Path, PathBuf};

/// Unified input abstraction for various data sources
pub enum Input {
    /// Direct text input
    Text(String),
    /// File path input
    File(PathBuf),
    /// Raw bytes input
    Bytes(Vec<u8>),
    /// Reader input (boxed for object safety)
    Reader(Box<dyn Read + Send + Sync>),
}

impl std::fmt::Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::Text(text) => f
                .debug_struct("Input::Text")
                .field("length", &text.len())
                .finish(),
            Input::File(path) => f.debug_struct("Input::File").field("path", path).finish(),
            Input::Bytes(bytes) => f
                .debug_struct("Input::Bytes")
                .field("length", &bytes.len())
                .finish(),
            Input::Reader(_) => f.debug_struct("Input::Reader").finish(),
        }
    }
}

impl Input {
    /// Create input from text
    pub fn from_text(text: impl Into<String>) -> Self {
        Input::Text(text.into())
    }

    /// Create input from file path
    pub fn from_file(path: impl AsRef<Path>) -> Self {
        Input::File(path.as_ref().to_path_buf())
    }

    /// Create input from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Input::Bytes(bytes)
    }

    /// Create input from reader
    pub fn from_reader(reader: impl Read + Send + Sync + 'static) -> Self {
        Input::Reader(Box::new(reader))
    }

    /// Convert input to bytes
    pub(crate) fn into_bytes(self) -> Result<Vec<u8>, crate::api::Error> {
        match self {
            Input::Text(text) => Ok(text.into_bytes()),
            Input::Bytes(bytes) => Ok(bytes),
            Input::File(path) => std::fs::read(&path).map_err(|e| {
                crate::api::Error::Infrastructure(format!(
                    "Failed to read file {}: {}",
                    path.display(),
                    e
                ))
            }),
            Input::Reader(mut reader) => {
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer).map_err(|e| {
                    crate::api::Error::Infrastructure(format!("Failed to read from reader: {}", e))
                })?;
                Ok(buffer)
            }
        }
    }

    /// Get text content from input
    pub(crate) fn into_text(self) -> Result<String, crate::api::Error> {
        let bytes = self.into_bytes()?;
        String::from_utf8(bytes).map_err(|e| {
            crate::api::Error::Infrastructure(format!("Invalid UTF-8 encoding: {}", e))
        })
    }
}
