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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_read_text_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = "Hello, world!\nThis is a test.";
        fs::write(&file_path, content).unwrap();

        let result = FileReader::read_text(&file_path).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_read_text_nonexistent_file() {
        let path = Path::new("/nonexistent/file.txt");
        let result = FileReader::read_text(path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to read file"));
    }

    #[test]
    fn test_read_text_utf8_content() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("utf8.txt");

        let content = "Hello 世界! 🌍 Emoji and UTF-8";
        fs::write(&file_path, content).unwrap();

        let result = FileReader::read_text(&file_path).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("sized.txt");

        let content = "a".repeat(1024);
        fs::write(&file_path, &content).unwrap();

        let size = FileReader::file_size(&file_path).unwrap();
        assert_eq!(size, 1024);
    }

    #[test]
    fn test_file_size_nonexistent() {
        let path = Path::new("/nonexistent/file.txt");
        let result = FileReader::file_size(path);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to get metadata"));
    }

    #[test]
    fn test_should_stream_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("small.txt");

        fs::write(&file_path, "small content").unwrap();

        let should_stream = FileReader::should_stream(&file_path, 1).unwrap();
        assert!(!should_stream);
    }

    #[test]
    fn test_should_stream_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");

        // Create a file larger than 1KB
        let content = "a".repeat(2048);
        fs::write(&file_path, content).unwrap();

        // Threshold is 0.001 MB (1KB)
        let should_stream = FileReader::should_stream(&file_path, 0).unwrap();
        assert!(should_stream);
    }

    #[test]
    fn test_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");

        File::create(&file_path).unwrap();

        let content = FileReader::read_text(&file_path).unwrap();
        assert_eq!(content, "");

        let size = FileReader::file_size(&file_path).unwrap();
        assert_eq!(size, 0);
    }

    #[cfg(unix)]
    #[test]
    fn test_read_text_permission_denied() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("no_read.txt");

        fs::write(&file_path, "content").unwrap();

        // Remove read permissions
        let metadata = fs::metadata(&file_path).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o000);
        fs::set_permissions(&file_path, permissions).unwrap();

        let result = FileReader::read_text(&file_path);
        assert!(result.is_err());

        // Restore permissions for cleanup
        let mut permissions = fs::metadata(&file_path).unwrap().permissions();
        permissions.set_mode(0o644);
        fs::set_permissions(&file_path, permissions).unwrap();
    }
}
