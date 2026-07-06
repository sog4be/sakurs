//! Main sentence processor implementation

use std::io::Read;
use std::time::Instant;

use crate::api::{Config, Error, Input, Language, Output};
use crate::application::{DeltaStackProcessor, ExecutionMode, ProcessorConfig};
use crate::domain::language::config::LanguageConfig;

/// Unified sentence processor with clean API
pub struct SentenceProcessor {
    processor: DeltaStackProcessor,
    config: Config,
}

impl SentenceProcessor {
    /// Create a new processor with default configuration
    pub fn new() -> Self {
        Self::with_config(Config::default()).expect("Default config should always be valid")
    }

    /// Create a processor with custom configuration
    pub fn with_config(config: Config) -> Result<Self, Error> {
        let processor_config = Self::build_processor_config(&config)?;
        let code = match config.language {
            Language::English => "en",
            Language::Japanese => "ja",
        };
        let processor = DeltaStackProcessor::from_language_code(processor_config, code)?;

        Ok(Self { processor, config })
    }

    /// Create a processor with a custom language configuration (e.g. loaded
    /// from an external TOML file via [`LanguageConfig::from_file`])
    pub fn with_language_config(config: Config, language: &LanguageConfig) -> Result<Self, Error> {
        let processor_config = Self::build_processor_config(&config)?;
        let processor = DeltaStackProcessor::from_language_config(processor_config, language)?;

        Ok(Self { processor, config })
    }

    /// Create a processor for a specific language
    pub fn with_language(lang_code: impl Into<String>) -> Result<Self, Error> {
        let config = Config::builder().language(lang_code)?.build()?;
        Self::with_config(config)
    }

    /// Process input and return sentence boundaries
    pub fn process(&self, input: Input) -> Result<Output, Error> {
        let start = Instant::now();

        // Convert input to text
        let text = input.into_text()?;

        // Determine execution mode based on configuration
        let mode = if let Some(threads) = self.config.threads {
            if threads == 1 {
                ExecutionMode::Sequential
            } else {
                ExecutionMode::Parallel {
                    threads: Some(threads),
                }
            }
        } else {
            ExecutionMode::Adaptive
        };

        // Process using the processor
        let result = self.processor.process(&text, mode)?;

        // Convert to public output format
        let duration = start.elapsed();
        Ok(Output::from_delta_stack_result(result, &text, duration))
    }

    /// Process input from a reader stream
    pub fn process_stream<R: Read + Send + Sync + 'static>(
        &self,
        reader: R,
    ) -> Result<Output, Error> {
        self.process(Input::from_reader(reader))
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Convert public config to internal processor config
    fn build_processor_config(config: &Config) -> Result<ProcessorConfig, Error> {
        Ok(ProcessorConfig {
            chunk_size: config.chunk_size,
            parallel_threshold: config.parallel_threshold,
            max_threads: config.threads,
        })
    }
}

impl Default for SentenceProcessor {
    fn default() -> Self {
        Self::new()
    }
}
