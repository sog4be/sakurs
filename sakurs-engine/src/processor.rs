//! Main sentence processor and builder
//!
//! Implements the public API interface as specified in DESIGN.md Section 3.3.

use crate::{
    adaptive_dispatcher::AdaptiveDispatcher,
    error::{ApiError, ApiResult, EngineError},
    executor::{ExecutionMetrics, ExecutionMode},
    input::Input,
    processor_config::{ProcessorConfig, ProcessorConfigBuilder},
};
use sakurs_core::Boundary;
use std::sync::Arc;

/// Main sentence processor
///
/// Entry point for sentence boundary detection as specified in DESIGN.md.
/// Provides a clean, stable API with backwards compatibility guarantees.
pub struct SentenceProcessor {
    dispatcher: Arc<AdaptiveDispatcher>,
    config: ProcessorConfig,
}

/// Rich output with metadata
#[derive(Debug, Clone)]
pub struct Output {
    /// Detected sentence boundaries  
    pub boundaries: Vec<Boundary>,
    /// Processing metadata and performance metrics
    pub metadata: ProcessingMetadata,
}

/// Processing metadata
#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    /// Execution mode that was actually used
    pub execution_mode: ExecutionMode,
    /// Processing time in milliseconds
    pub processing_time_ms: f64,
    /// Total bytes processed
    pub bytes_processed: usize,
    /// Number of chunks processed (if applicable)
    pub chunks_processed: Option<usize>,
    /// Thread efficiency (0.0 to 1.0)
    pub thread_efficiency: f64,
    /// Throughput in bytes per second
    pub bytes_per_second: f64,
}

impl From<ExecutionMetrics> for ProcessingMetadata {
    fn from(metrics: ExecutionMetrics) -> Self {
        Self {
            execution_mode: metrics.mode_used,
            processing_time_ms: metrics.processing_time.as_secs_f64() * 1000.0,
            bytes_processed: metrics.bytes_processed,
            chunks_processed: if metrics.chunks_processed > 1 {
                Some(metrics.chunks_processed)
            } else {
                None
            },
            thread_efficiency: metrics.thread_efficiency,
            bytes_per_second: metrics.bytes_per_second,
        }
    }
}

impl SentenceProcessor {
    /// Create a new processor with default configuration
    pub fn new() -> ApiResult<Self> {
        Self::with_config(ProcessorConfig::default())
    }

    /// Create a processor for a specific language
    pub fn with_language(language: &str) -> ApiResult<Self> {
        let config = ProcessorConfig::new(language);
        Self::with_config(config)
    }

    /// Create a processor with custom configuration
    pub fn with_config(config: ProcessorConfig) -> ApiResult<Self> {
        let engine_config = config.to_engine_config();
        let _rules = config.load_language_rules().map_err(|e| match e {
            EngineError::ConfigError(msg) if msg.contains("Unknown language") => {
                ApiError::UnsupportedLanguage {
                    code: config.language.clone(),
                }
            }
            _ => ApiError::Engine(e),
        })?;

        let dispatcher = Arc::new(AdaptiveDispatcher::new(engine_config));

        Ok(Self { dispatcher, config })
    }

    /// Process input with automatic mode selection and return rich output
    pub fn process(&self, input: Input) -> ApiResult<Output> {
        let text = input.to_text().map_err(ApiError::Engine)?;
        let rules = self
            .config
            .load_language_rules()
            .map_err(ApiError::Engine)?;

        let processing_output = self
            .dispatcher
            .process_adaptive(&text, &rules)
            .map_err(ApiError::Engine)?;

        Ok(Output {
            boundaries: processing_output.boundaries,
            metadata: processing_output.metadata.into(),
        })
    }

    /// Process input with specific execution mode
    pub fn process_with_mode(&self, input: Input, mode: ExecutionMode) -> ApiResult<Output> {
        let text = input.to_text().map_err(ApiError::Engine)?;
        let rules = self
            .config
            .load_language_rules()
            .map_err(ApiError::Engine)?;

        let processing_output = self
            .dispatcher
            .process_with_mode(&text, &rules, mode)
            .map_err(ApiError::Engine)?;

        Ok(Output {
            boundaries: processing_output.boundaries,
            metadata: processing_output.metadata.into(),
        })
    }

    /// Process input and return only boundaries (legacy API)
    pub fn process_boundaries(&self, input: Input) -> ApiResult<Vec<Boundary>> {
        Ok(self.process(input)?.boundaries)
    }

    /// Process text string directly (convenience method)
    pub fn process_text(&self, text: &str) -> ApiResult<Output> {
        self.process(Input::from_text(text.to_string()))
    }
}

/// Builder for SentenceProcessor
///
/// Provides a fluent interface for configuring the processor.
pub struct SentenceProcessorBuilder {
    config_builder: ProcessorConfigBuilder,
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
            config_builder: ProcessorConfigBuilder::new(),
        }
    }

    /// Set the language
    pub fn language<S: Into<String>>(mut self, language: S) -> Self {
        self.config_builder = self.config_builder.language(language);
        self
    }

    /// Set the execution mode
    pub fn execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.config_builder = self.config_builder.execution_mode(mode);
        self
    }

    /// Set the thread count
    pub fn threads(mut self, count: Option<usize>) -> Self {
        self.config_builder = self.config_builder.threads(count);
        self
    }

    /// Set the adaptive threshold
    pub fn adaptive_threshold_kb(mut self, threshold: usize) -> Self {
        self.config_builder = self.config_builder.adaptive_threshold_kb(threshold);
        self
    }

    /// Use streaming configuration preset
    pub fn streaming(mut self) -> Self {
        self.config_builder = self.config_builder.streaming();
        self
    }

    /// Use fast configuration preset
    pub fn fast(mut self) -> Self {
        self.config_builder = self.config_builder.fast();
        self
    }

    /// Use balanced configuration preset
    pub fn balanced(mut self) -> Self {
        self.config_builder = self.config_builder.balanced();
        self
    }

    /// Use accurate configuration preset
    pub fn accurate(mut self) -> Self {
        self.config_builder = self.config_builder.accurate();
        self
    }

    /// Build the processor
    pub fn build(self) -> ApiResult<SentenceProcessor> {
        let config = self.config_builder.build().map_err(ApiError::Engine)?;
        SentenceProcessor::with_config(config)
    }
}
