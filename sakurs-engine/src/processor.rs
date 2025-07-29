//! Main sentence processor and builder

use crate::{
    config::{ChunkPolicy, EngineConfig},
    error::{EngineError, Result},
    executor::{auto_select, ExecutionMode, Executor, SequentialExecutor, StreamingExecutor},
    language::{get_language_rules, LanguageRulesImpl},
};
use sakurs_core::Boundary;

#[cfg(feature = "parallel")]
use crate::executor::ParallelExecutor;

/// Main sentence processor
pub struct SentenceProcessor {
    config: EngineConfig,
    rules: LanguageRulesImpl,
}

impl SentenceProcessor {
    /// Create a new processor with default English rules
    pub fn new() -> Result<Self> {
        Self::with_language("en")
    }

    /// Create a processor for a specific language
    pub fn with_language(language: &str) -> Result<Self> {
        let rules = get_language_rules(language)
            .ok_or_else(|| EngineError::ConfigError(format!("unknown language: {language}")))?;

        Ok(Self {
            config: EngineConfig::default(),
            rules,
        })
    }

    /// Create a processor with custom configuration
    pub fn with_config(config: EngineConfig, rules: LanguageRulesImpl) -> Self {
        Self { config, rules }
    }

    /// Process text with automatic mode selection
    pub fn process(&self, text: &str) -> Result<Vec<Boundary>> {
        let mode = auto_select(text.len(), &self.config);
        self.process_with_mode(text, mode)
    }

    /// Process text with specific execution mode
    pub fn process_with_mode(&self, text: &str, mode: ExecutionMode) -> Result<Vec<Boundary>> {
        match mode {
            ExecutionMode::Sequential => {
                let executor = SequentialExecutor;
                executor.process(text, &self.rules)
            }

            #[cfg(feature = "parallel")]
            ExecutionMode::Parallel => {
                let executor = ParallelExecutor::new(self.config.chunk_policy);
                executor.process(text, &self.rules)
            }

            #[cfg(not(feature = "parallel"))]
            ExecutionMode::Parallel => Err(EngineError::ConfigError(
                "parallel execution not available (feature disabled)".to_string(),
            )),

            ExecutionMode::Streaming => {
                let executor = match self.config.chunk_policy {
                    ChunkPolicy::Streaming {
                        window_size,
                        overlap,
                    } => StreamingExecutor::new(window_size, overlap),
                    _ => StreamingExecutor::new(64 * 1024, 1024),
                };
                executor.process(text, &self.rules)
            }
        }
    }
}

/// Builder for SentenceProcessor
pub struct SentenceProcessorBuilder {
    config: EngineConfig,
    language: Option<String>,
    rules: Option<LanguageRulesImpl>,
}

impl Default for SentenceProcessorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SentenceProcessorBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: EngineConfig::default(),
            language: None,
            rules: None,
        }
    }

    /// Set the language
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set custom language rules
    pub fn rules(mut self, rules: LanguageRulesImpl) -> Self {
        self.rules = Some(rules);
        self
    }

    /// Set chunk policy
    pub fn chunk_policy(mut self, policy: ChunkPolicy) -> Self {
        self.config.chunk_policy = policy;
        self
    }

    /// Set thread count
    pub fn threads(mut self, threads: Option<usize>) -> Self {
        self.config.threads = threads;
        self
    }

    /// Set parallel threshold
    pub fn parallel_threshold(mut self, threshold: usize) -> Self {
        self.config.parallel_threshold = threshold;
        self
    }

    /// Use streaming configuration
    pub fn streaming(mut self) -> Self {
        self.config = EngineConfig::streaming();
        self
    }

    /// Use fast configuration
    pub fn fast(mut self) -> Self {
        self.config = EngineConfig::fast();
        self
    }

    /// Build the processor
    pub fn build(self) -> Result<SentenceProcessor> {
        let rules = if let Some(rules) = self.rules {
            rules
        } else if let Some(language) = self.language {
            get_language_rules(&language)
                .ok_or_else(|| EngineError::ConfigError(format!("unknown language: {language}")))?
        } else {
            get_language_rules("en").unwrap() // Default to English
        };

        Ok(SentenceProcessor::with_config(self.config, rules))
    }
}
