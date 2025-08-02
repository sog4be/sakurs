//! Processor configuration
//!
//! Public configuration interface as specified in DESIGN.md Section 3.3.

use crate::{
    config::{ChunkPolicy, EngineConfig},
    error::{EngineError, Result},
    executor::ExecutionMode,
    input::{PerformanceHints, ProcessingPattern},
    language::{get_language_rules, LanguageRulesImpl},
};

/// Custom rules override (placeholder for future extension)
#[derive(Debug, Clone)]
pub struct CustomRules {
    /// Custom language configuration (future)
    _placeholder: (),
}

/// Public processor configuration
///
/// This provides a clean, stable API that maps to the internal EngineConfig
/// while hiding implementation details.
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Language code (ISO 639-1)
    pub language: String,
    /// Custom rules override (optional)
    pub custom_rules: Option<CustomRules>,
    /// Performance optimization hints
    pub performance_hints: PerformanceHints,
    /// Execution mode preference
    pub execution_mode: ExecutionMode,
    /// Thread count override (None = auto-detect)
    pub thread_count: Option<usize>,
    /// Adaptive threshold in KB (for adaptive mode)
    pub adaptive_threshold_kb: usize,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            custom_rules: None,
            performance_hints: PerformanceHints::default(),
            execution_mode: ExecutionMode::Adaptive,
            thread_count: None,
            adaptive_threshold_kb: 128, // Default from DESIGN.md
        }
    }
}

impl ProcessorConfig {
    /// Create a new configuration with the specified language
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
            ..Default::default()
        }
    }

    /// Create a streaming configuration preset
    pub fn streaming(language: &str) -> Self {
        Self {
            language: language.to_string(),
            performance_hints: PerformanceHints {
                processing_pattern: ProcessingPattern::Streaming,
                ..Default::default()
            },
            execution_mode: ExecutionMode::Streaming,
            thread_count: Some(1), // Streaming is typically single-threaded
            ..Default::default()
        }
    }

    /// Create a fast configuration preset
    pub fn fast(language: &str) -> Self {
        Self {
            language: language.to_string(),
            performance_hints: PerformanceHints {
                prefer_speed: true,
                processing_pattern: ProcessingPattern::Batch,
                ..Default::default()
            },
            execution_mode: ExecutionMode::Adaptive,
            adaptive_threshold_kb: 64, // Lower threshold for faster parallel activation
            ..Default::default()
        }
    }

    /// Create a balanced configuration preset
    pub fn balanced(language: &str) -> Self {
        Self::new(language)
    }

    /// Create an accurate configuration preset
    pub fn accurate(language: &str) -> Self {
        Self {
            language: language.to_string(),
            performance_hints: PerformanceHints {
                prefer_speed: false,
                processing_pattern: ProcessingPattern::Interactive,
                ..Default::default()
            },
            execution_mode: ExecutionMode::Sequential, // Sequential for maximum accuracy
            adaptive_threshold_kb: 256,                // Higher threshold
            ..Default::default()
        }
    }

    /// Convert to internal EngineConfig
    pub(crate) fn to_engine_config(&self) -> EngineConfig {
        let chunk_policy = match self.performance_hints.processing_pattern {
            ProcessingPattern::Streaming => ChunkPolicy::Streaming {
                window_size: 64 * 1024,
                overlap: 1024,
            },
            ProcessingPattern::Interactive => ChunkPolicy::Fixed { size: 128 * 1024 },
            ProcessingPattern::Batch => {
                if self.performance_hints.prefer_speed {
                    ChunkPolicy::Fixed { size: 512 * 1024 }
                } else {
                    ChunkPolicy::Auto {
                        target_bytes: 256 * 1024,
                    }
                }
            }
        };

        EngineConfig {
            execution_mode: self.execution_mode,
            chunk_policy,
            threads: self.thread_count,
            parallel_threshold: self.adaptive_threshold_kb * 1024,
            adaptive_threshold: Some(self.adaptive_threshold_kb * 1024),
        }
    }

    /// Load language rules for this configuration
    pub(crate) fn load_language_rules(&self) -> Result<LanguageRulesImpl> {
        get_language_rules(&self.language)
            .ok_or_else(|| EngineError::ConfigError(format!("Unknown language: {}", self.language)))
    }
}

/// Builder for ProcessorConfig
#[derive(Debug, Default)]
pub struct ProcessorConfigBuilder {
    config: ProcessorConfig,
}

impl ProcessorConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the language
    pub fn language<S: Into<String>>(mut self, language: S) -> Self {
        self.config.language = language.into();
        self
    }

    /// Set the execution mode
    pub fn execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.config.execution_mode = mode;
        self
    }

    /// Set the thread count
    pub fn threads(mut self, count: Option<usize>) -> Self {
        self.config.thread_count = count;
        self
    }

    /// Set the adaptive threshold
    pub fn adaptive_threshold_kb(mut self, threshold: usize) -> Self {
        self.config.adaptive_threshold_kb = threshold;
        self
    }

    /// Use streaming preset
    pub fn streaming(mut self) -> Self {
        self.config.performance_hints.processing_pattern = ProcessingPattern::Streaming;
        self.config.execution_mode = ExecutionMode::Streaming;
        self.config.thread_count = Some(1);
        self
    }

    /// Use fast preset
    pub fn fast(mut self) -> Self {
        self.config.performance_hints.prefer_speed = true;
        self.config.adaptive_threshold_kb = 64;
        self
    }

    /// Use balanced preset
    pub fn balanced(self) -> Self {
        // Already the default
        self
    }

    /// Use accurate preset  
    pub fn accurate(mut self) -> Self {
        self.config.performance_hints.prefer_speed = false;
        self.config.execution_mode = ExecutionMode::Sequential;
        self.config.adaptive_threshold_kb = 256;
        self
    }

    /// Set performance hints
    pub fn performance_hints(mut self, hints: PerformanceHints) -> Self {
        self.config.performance_hints = hints;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<ProcessorConfig> {
        // Validate the configuration
        if self.config.language.is_empty() {
            return Err(EngineError::ConfigError(
                "Language cannot be empty".to_string(),
            ));
        }

        // Check if language is supported
        self.config.load_language_rules()?;

        Ok(self.config)
    }
}
