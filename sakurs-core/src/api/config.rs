//! Configuration API for sentence processing

use crate::api::{Error, Language};
use std::str::FromStr;

/// Default configuration constants
pub mod defaults {
    /// Default chunk size in bytes (256KB)
    pub const CHUNK_SIZE: usize = 256 * 1024;

    /// Parallel processing threshold in bytes (1MB)
    pub const PARALLEL_THRESHOLD: usize = 1024 * 1024;

    /// Overlap size between chunks in bytes
    pub const OVERLAP_SIZE: usize = 256;
}

/// Processing configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) language: Language,
    pub(crate) chunk_size: usize,         // in bytes
    pub(crate) parallel_threshold: usize, // minimum size for parallel processing
    pub(crate) threads: Option<usize>,    // None = all available threads
    pub(crate) overlap_size: usize,       // overlap between chunks in bytes
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
            chunk_size: defaults::CHUNK_SIZE,
            parallel_threshold: defaults::PARALLEL_THRESHOLD,
            threads: None,
            overlap_size: defaults::OVERLAP_SIZE,
        }
    }
}

impl Config {
    /// Create a configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Create a configuration optimized for small texts
    pub fn small_text() -> Self {
        Self {
            language: Language::default(),
            chunk_size: 8 * 1024,           // 8KB chunks
            parallel_threshold: usize::MAX, // Never use parallel
            threads: None,
            overlap_size: 64, // Smaller overlap
        }
    }

    /// Create a configuration optimized for large texts
    pub fn large_text() -> Self {
        Self {
            language: Language::default(),
            chunk_size: 512 * 1024,         // 512KB chunks
            parallel_threshold: 512 * 1024, // 512KB threshold
            threads: None,                  // Use all available cores
            overlap_size: 512,              // Larger overlap
        }
    }

    /// Create a configuration optimized for streaming
    pub fn streaming() -> Self {
        Self {
            language: Language::default(),
            chunk_size: 32 * 1024,          // 32KB chunks
            parallel_threshold: 256 * 1024, // 256KB threshold
            threads: Some(2),               // Limited parallelism
            overlap_size: 128,              // Moderate overlap
        }
    }

    /// Validate the configuration
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.chunk_size == 0 {
            return Err(Error::Configuration(
                "chunk_size must be greater than 0".into(),
            ));
        }

        if self.overlap_size >= self.chunk_size {
            return Err(Error::Configuration(
                "overlap_size must be less than chunk_size".into(),
            ));
        }

        if let Some(threads) = self.threads {
            if threads == 0 {
                return Err(Error::Configuration(
                    "threads must be greater than 0".into(),
                ));
            }
        }

        Ok(())
    }
}

/// Fluent builder for configuration
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    language: Option<String>,
    chunk_size: Option<usize>,
    parallel_threshold: Option<usize>,
    threads: Option<usize>,
    overlap_size: Option<usize>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the language by code
    pub fn language(mut self, code: impl Into<String>) -> Result<Self, Error> {
        self.language = Some(code.into());
        Ok(self)
    }

    /// Set the chunk size in bytes
    pub fn chunk_size(mut self, bytes: usize) -> Self {
        self.chunk_size = Some(bytes);
        self
    }

    /// Set the number of threads (None = all available)
    pub fn threads(mut self, count: Option<usize>) -> Self {
        self.threads = count;
        self
    }

    /// Set the parallel processing threshold in bytes
    pub fn parallel_threshold(mut self, bytes: usize) -> Self {
        self.parallel_threshold = Some(bytes);
        self
    }

    /// Set the overlap size between chunks in bytes
    pub fn overlap_size(mut self, bytes: usize) -> Self {
        self.overlap_size = Some(bytes);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Config, Error> {
        let mut config = Config::default();

        if let Some(lang_code) = self.language {
            config.language = Language::from_str(&lang_code)?;
        }

        if let Some(size) = self.chunk_size {
            config.chunk_size = size;
        }

        if let Some(threshold) = self.parallel_threshold {
            config.parallel_threshold = threshold;
        }

        if self.threads.is_some() {
            config.threads = self.threads;
        }

        if let Some(overlap) = self.overlap_size {
            config.overlap_size = overlap;
        }

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.chunk_size, defaults::CHUNK_SIZE);
        assert_eq!(config.parallel_threshold, defaults::PARALLEL_THRESHOLD);
        assert_eq!(config.overlap_size, defaults::OVERLAP_SIZE);
        assert!(config.threads.is_none());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        // Invalid chunk size
        let config = Config {
            chunk_size: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid overlap size
        let config = Config {
            chunk_size: 100,
            overlap_size: 200,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid thread count
        let config = Config {
            threads: Some(0),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_preset_configs() {
        // Small text preset
        let small = Config::small_text();
        assert_eq!(small.chunk_size, 8 * 1024);
        assert_eq!(small.parallel_threshold, usize::MAX);
        assert_eq!(small.overlap_size, 64);
        assert!(small.validate().is_ok());

        // Large text preset
        let large = Config::large_text();
        assert_eq!(large.chunk_size, 512 * 1024);
        assert_eq!(large.parallel_threshold, 512 * 1024);
        assert_eq!(large.overlap_size, 512);
        assert!(large.validate().is_ok());

        // Streaming preset
        let streaming = Config::streaming();
        assert_eq!(streaming.chunk_size, 32 * 1024);
        assert_eq!(streaming.parallel_threshold, 256 * 1024);
        assert_eq!(streaming.threads, Some(2));
        assert_eq!(streaming.overlap_size, 128);
        assert!(streaming.validate().is_ok());
    }

    #[test]
    fn test_config_builder_with_new_fields() {
        let config = Config::builder()
            .chunk_size(128 * 1024)
            .parallel_threshold(256 * 1024)
            .overlap_size(512)
            .threads(Some(4))
            .build()
            .unwrap();

        assert_eq!(config.chunk_size, 128 * 1024);
        assert_eq!(config.parallel_threshold, 256 * 1024);
        assert_eq!(config.overlap_size, 512);
        assert_eq!(config.threads, Some(4));
    }
}
