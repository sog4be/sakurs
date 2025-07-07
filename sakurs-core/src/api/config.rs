//! Configuration API for sentence processing

use crate::api::Language;

/// Processing configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) language: Language,
    pub(crate) performance: PerformanceConfig,
    pub(crate) accuracy: AccuracyConfig,
}

/// Performance-related configuration
#[derive(Debug, Clone)]
pub(crate) struct PerformanceConfig {
    pub threads: Option<usize>,
    pub chunk_size_kb: usize,
    pub memory_limit_mb: Option<usize>,
}

/// Accuracy-related configuration
#[derive(Debug, Clone)]
pub(crate) struct AccuracyConfig {
    pub enable_abbreviations: bool,
    pub enable_numbers: bool,
    pub enable_quotes: bool,
}

impl Config {
    /// Create a configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Speed-optimized preset
    pub fn fast() -> Self {
        Self {
            language: Language::default(),
            performance: PerformanceConfig {
                threads: None,       // Use all available
                chunk_size_kb: 1024, // 1MB chunks
                memory_limit_mb: None,
            },
            accuracy: AccuracyConfig {
                enable_abbreviations: false,
                enable_numbers: false,
                enable_quotes: false,
            },
        }
    }

    /// Balanced preset (default)
    pub fn balanced() -> Self {
        Self {
            language: Language::default(),
            performance: PerformanceConfig {
                threads: None,
                chunk_size_kb: 512,
                memory_limit_mb: None,
            },
            accuracy: AccuracyConfig {
                enable_abbreviations: true,
                enable_numbers: true,
                enable_quotes: true,
            },
        }
    }

    /// Accuracy-optimized preset
    pub fn accurate() -> Self {
        Self {
            language: Language::default(),
            performance: PerformanceConfig {
                threads: Some(1), // Single-threaded for consistency
                chunk_size_kb: 256,
                memory_limit_mb: None,
            },
            accuracy: AccuracyConfig {
                enable_abbreviations: true,
                enable_numbers: true,
                enable_quotes: true,
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::balanced()
    }
}

/// Fluent builder for configuration
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    language: Option<Language>,
    threads: Option<usize>,
    chunk_size_kb: Option<usize>,
    memory_limit_mb: Option<usize>,
    enable_abbreviations: Option<bool>,
    enable_numbers: Option<bool>,
    enable_quotes: Option<bool>,
}

impl ConfigBuilder {
    /// Set the language by code
    pub fn language(mut self, code: &str) -> Self {
        self.language = Some(Language::from_code(code));
        self
    }

    /// Set the number of threads
    pub fn threads(mut self, count: usize) -> Self {
        self.threads = Some(count);
        self
    }

    /// Set the chunk size in KB
    pub fn chunk_size(mut self, kb: usize) -> Self {
        self.chunk_size_kb = Some(kb);
        self
    }

    /// Set the memory limit in MB
    pub fn memory_limit(mut self, mb: usize) -> Self {
        self.memory_limit_mb = Some(mb);
        self
    }

    /// Enable or disable abbreviation handling
    pub fn abbreviations(mut self, enable: bool) -> Self {
        self.enable_abbreviations = Some(enable);
        self
    }

    /// Enable or disable number handling
    pub fn numbers(mut self, enable: bool) -> Self {
        self.enable_numbers = Some(enable);
        self
    }

    /// Enable or disable quote handling
    pub fn quotes(mut self, enable: bool) -> Self {
        self.enable_quotes = Some(enable);
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Config, crate::api::Error> {
        let base = Config::balanced();

        Ok(Config {
            language: self.language.unwrap_or(base.language),
            performance: PerformanceConfig {
                threads: self.threads.or(base.performance.threads),
                chunk_size_kb: self.chunk_size_kb.unwrap_or(base.performance.chunk_size_kb),
                memory_limit_mb: self.memory_limit_mb.or(base.performance.memory_limit_mb),
            },
            accuracy: AccuracyConfig {
                enable_abbreviations: self
                    .enable_abbreviations
                    .unwrap_or(base.accuracy.enable_abbreviations),
                enable_numbers: self.enable_numbers.unwrap_or(base.accuracy.enable_numbers),
                enable_quotes: self.enable_quotes.unwrap_or(base.accuracy.enable_quotes),
            },
        })
    }
}
