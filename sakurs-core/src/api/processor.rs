//! Main sentence processor implementation

use std::io::Read;
use std::sync::Arc;
use std::time::Instant;

use crate::api::{Config, Error, Input, Language, Output};
use crate::application::{ProcessorConfig, UnifiedProcessor};
use crate::domain::language::{EnglishLanguageRules, JapaneseLanguageRules, LanguageRules};

/// Unified sentence processor with clean API
pub struct SentenceProcessor {
    processor: UnifiedProcessor,
    config: Config,
}

impl SentenceProcessor {
    /// Create a new processor with default configuration
    pub fn new() -> Self {
        Self::with_config(Config::default()).expect("Default config should always be valid")
    }

    /// Create a processor with custom configuration
    pub fn with_config(config: Config) -> Result<Self, Error> {
        let language_rules = Self::create_language_rules(&config.language);
        let processor_config = Self::build_processor_config(&config)?;
        let processor = UnifiedProcessor::with_config(language_rules, processor_config);

        Ok(Self { processor, config })
    }

    /// Create a processor for a specific language
    pub fn with_language(lang_code: impl Into<String>) -> Result<Self, Error> {
        let config = Config::builder().language(lang_code)?.build()?;
        Self::with_config(config)
    }

    /// Create language rules based on the language
    fn create_language_rules(language: &Language) -> Arc<dyn LanguageRules> {
        match language {
            Language::English => Arc::new(EnglishLanguageRules::new()),
            Language::Japanese => Arc::new(JapaneseLanguageRules::new()),
        }
    }

    /// Process input and return sentence boundaries
    pub fn process(&self, input: Input) -> Result<Output, Error> {
        let start = Instant::now();

        // Convert input to text
        let text = input.into_text()?;

        // Process using the processor
        let result = self.processor.process(&text)?;

        // Convert to public output format
        let duration = start.elapsed();
        Ok(Output::from_internal(result, &text, duration))
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
        use crate::api::config::defaults;

        Ok(ProcessorConfig {
            chunk_size: config.chunk_size,
            parallel_threshold: defaults::PARALLEL_THRESHOLD,
            max_threads: config.threads,
            overlap_size: defaults::OVERLAP_SIZE,
            enable_simd: false,                // SIMD not yet implemented
            max_text_size: 1024 * 1024 * 1024, // 1GB fixed limit
            use_mmap: false,                   // Default to false
        })
    }
}

impl Default for SentenceProcessor {
    fn default() -> Self {
        Self::new()
    }
}
