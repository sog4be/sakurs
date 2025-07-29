//! High-level configuration API

use crate::error::{ApiError, Result};
use sakurs_engine::{
    ChunkPolicy, EngineConfig, LanguageRulesImpl, SentenceProcessor, SentenceProcessorBuilder,
};

/// High-level configuration for sentence processing
#[derive(Debug, Clone)]
pub struct Config {
    inner: EngineConfig,
    language: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inner: EngineConfig::default(),
            language: "en".to_string(),
        }
    }
}

impl Config {
    /// Create a streaming configuration
    pub fn streaming() -> Self {
        Self {
            inner: EngineConfig::streaming(),
            language: "en".to_string(),
        }
    }

    /// Create a fast configuration
    pub fn fast() -> Self {
        Self {
            inner: EngineConfig::fast(),
            language: "en".to_string(),
        }
    }

    /// Create a balanced configuration
    pub fn balanced() -> Self {
        Self::default()
    }

    /// Create a builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// Configuration builder
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: Config,
    custom_rules: Option<LanguageRulesImpl>,
}

impl ConfigBuilder {
    /// Set the language
    pub fn language(mut self, language: impl Into<String>) -> Result<Self> {
        self.config.language = language.into();
        Ok(self)
    }

    /// Set thread count
    pub fn threads(mut self, threads: Option<usize>) -> Self {
        self.config.inner.threads = threads;
        self
    }

    /// Set chunk size for fixed chunking
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.config.inner.chunk_policy = ChunkPolicy::Fixed { size };
        self
    }

    /// Use streaming configuration
    pub fn streaming(mut self) -> Self {
        self.config.inner = EngineConfig::streaming();
        self
    }

    /// Use fast configuration
    pub fn fast(mut self) -> Self {
        self.config.inner = EngineConfig::fast();
        self
    }

    /// Set custom language rules
    pub fn custom_rules(mut self, rules: LanguageRulesImpl) -> Self {
        self.custom_rules = Some(rules);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Config> {
        // Validate configuration
        if self.config.language.is_empty() && self.custom_rules.is_none() {
            return Err(ApiError::Config(
                "language or custom rules required".to_string(),
            ));
        }

        Ok(self.config)
    }

    /// Build a sentence processor directly
    pub fn build_processor(self) -> Result<SentenceProcessor> {
        let mut builder = SentenceProcessorBuilder::new()
            .chunk_policy(self.config.inner.chunk_policy)
            .threads(self.config.inner.threads)
            .parallel_threshold(self.config.inner.parallel_threshold);

        if let Some(rules) = self.custom_rules {
            builder = builder.rules(rules);
        } else {
            builder = builder.language(self.config.language);
        }

        builder.build().map_err(|e| ApiError::Engine(e.to_string()))
    }
}
