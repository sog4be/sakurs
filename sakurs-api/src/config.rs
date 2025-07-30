//! High-level configuration API

use crate::dto::{ExecutionMode, Language};
use crate::error::{ApiError, Result};

/// High-level configuration for sentence processing
#[derive(Debug, Clone)]
pub struct Config {
    /// Language for processing
    pub language: Language,
    /// Execution mode
    pub execution_mode: ExecutionMode,
    /// Number of threads (None = use all available)
    pub threads: Option<usize>,
    /// Chunk size in KB for parallel processing
    pub chunk_kb: Option<usize>,
    /// Adaptive threshold in KB per core
    pub adaptive_threshold_kb: Option<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::English,
            execution_mode: ExecutionMode::Adaptive,
            threads: None,
            chunk_kb: None,
            adaptive_threshold_kb: None,
        }
    }
}

impl Config {
    /// Create a streaming configuration
    pub fn streaming() -> ConfigBuilder {
        ConfigBuilder::default().execution_mode(ExecutionMode::Streaming)
    }

    /// Create a fast configuration (larger chunks, all threads)
    pub fn fast() -> ConfigBuilder {
        ConfigBuilder::default()
            .execution_mode(ExecutionMode::Adaptive)
            .chunk_kb(Some(512))
    }

    /// Create a balanced configuration (default)
    pub fn balanced() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Create an accurate configuration (smaller chunks for better accuracy)
    pub fn accurate() -> ConfigBuilder {
        ConfigBuilder::default()
            .execution_mode(ExecutionMode::Adaptive)
            .chunk_kb(Some(128))
    }

    /// Create a builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// Configuration builder with fluent interface
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    language: Option<Language>,
    execution_mode: Option<ExecutionMode>,
    threads: Option<usize>,
    chunk_kb: Option<usize>,
    adaptive_threshold_kb: Option<usize>,
}

impl ConfigBuilder {
    /// Set the language
    pub fn language(mut self, language: impl AsRef<str>) -> Result<Self> {
        let lang = match language.as_ref() {
            "en" | "english" | "English" => Language::English,
            "ja" | "japanese" | "Japanese" => Language::Japanese,
            code => return Err(ApiError::Config(format!("unsupported language: {code}"))),
        };
        self.language = Some(lang);
        Ok(self)
    }

    /// Set the execution mode
    pub fn execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.execution_mode = Some(mode);
        self
    }

    /// Set thread count
    pub fn threads(mut self, threads: Option<usize>) -> Self {
        // Validate thread count if provided
        if let Some(count) = threads {
            if count == 0 {
                self.threads = None; // 0 means use default
            } else {
                self.threads = Some(count);
            }
        } else {
            self.threads = threads;
        }
        self
    }

    /// Set chunk size in KB
    pub fn chunk_kb(mut self, size_kb: Option<usize>) -> Self {
        self.chunk_kb = size_kb;
        self
    }

    /// Set chunk size in bytes (compatibility method)
    pub fn chunk_size(mut self, size_bytes: usize) -> Self {
        self.chunk_kb = Some(size_bytes / 1024);
        self
    }

    /// Set adaptive threshold in KB per core
    pub fn adaptive_threshold(mut self, threshold_kb: usize) -> Self {
        self.adaptive_threshold_kb = Some(threshold_kb);
        self
    }

    /// Use sequential processing
    pub fn sequential(mut self) -> Self {
        self.execution_mode = Some(ExecutionMode::Sequential);
        self
    }

    /// Use parallel processing
    pub fn parallel(mut self) -> Self {
        self.execution_mode = Some(ExecutionMode::Parallel);
        self
    }

    /// Use streaming processing
    pub fn streaming(mut self) -> Self {
        self.execution_mode = Some(ExecutionMode::Streaming);
        self
    }

    /// Use adaptive processing (default)
    pub fn adaptive(mut self) -> Self {
        self.execution_mode = Some(ExecutionMode::Adaptive);
        self
    }

    /// Use fast configuration preset
    pub fn fast(mut self) -> Self {
        self.execution_mode = Some(ExecutionMode::Adaptive);
        self.chunk_kb = Some(512);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Config> {
        Ok(Config {
            language: self.language.unwrap_or(Language::English),
            execution_mode: self.execution_mode.unwrap_or(ExecutionMode::Adaptive),
            threads: self.threads,
            chunk_kb: self.chunk_kb,
            adaptive_threshold_kb: self.adaptive_threshold_kb,
        })
    }

    /// Build a sentence processor directly (compatibility method)
    pub fn build_processor(self) -> Result<crate::SentenceProcessor> {
        let config = self.build()?;
        crate::SentenceProcessor::with_config(config)
    }
}
