//! File reading utilities

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// File reader with UTF-8 validation
pub struct FileReader;

impl FileReader {
    /// Read a file as UTF-8 text
    pub fn read_text(path: &Path) -> Result<String> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        Ok(content)
    }

    /// Get file size in bytes
    pub fn file_size(path: &Path) -> Result<u64> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        Ok(metadata.len())
    }

    /// Check if file should be processed in streaming mode based on size
    pub fn should_stream(path: &Path, threshold_mb: u64) -> Result<bool> {
        let size = Self::file_size(path)?;
        Ok(size > threshold_mb * 1024 * 1024)
    }
}
