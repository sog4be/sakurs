//! Configuration API for sentence processing

use crate::api::{Error, Language};
use std::str::FromStr;

/// Default configuration constants
pub mod defaults {
    /// Default chunk size in bytes (512KB)
    pub const CHUNK_SIZE: usize = 512 * 1024;

    /// Parallel processing threshold in bytes (1MB)
    pub const PARALLEL_THRESHOLD: usize = 1024 * 1024;

    /// Overlap size between chunks in bytes
    pub const OVERLAP_SIZE: usize = 256;
}

/// Processing configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) language: Language,
    pub(crate) chunk_size: usize,      // in bytes
    pub(crate) threads: Option<usize>, // None = all available threads
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::default(),
            chunk_size: defaults::CHUNK_SIZE,
            threads: None,
        }
    }
}

impl Config {
    /// Create a configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Validate the configuration
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.chunk_size == 0 {
            return Err(Error::Configuration(
                "chunk_size must be greater than 0".into(),
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
    threads: Option<usize>,
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

    /// Build the configuration
    pub fn build(self) -> Result<Config, Error> {
        let mut config = Config::default();

        if let Some(lang_code) = self.language {
            config.language = Language::from_str(&lang_code)?;
        }

        if let Some(size) = self.chunk_size {
            config.chunk_size = size;
        }

        if self.threads.is_some() {
            config.threads = self.threads;
        }

        config.validate()?;
        Ok(config)
    }
}
