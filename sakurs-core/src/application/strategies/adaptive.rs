//! Adaptive strategy that automatically selects optimal processing approach

use super::{
    traits::{
        InputCharacteristics, ProcessingConfig, ProcessingStrategy, StrategyInput, StrategyOutput,
    },
    ParallelStrategy, SequentialStrategy, StrategySelector, StreamingStrategy,
};
use crate::application::config::{ProcessingError, ProcessingResult as Result};
use crate::domain::language::LanguageRules;
use std::sync::Arc;
use std::time::Instant;

/// Strategy that adapts based on input characteristics
pub struct AdaptiveStrategy {
    strategies: Vec<Box<dyn ProcessingStrategy>>,
    selector: StrategySelector,
}

impl AdaptiveStrategy {
    /// Create a new adaptive strategy with default sub-strategies
    pub fn new() -> Self {
        let strategies: Vec<Box<dyn ProcessingStrategy>> = vec![
            Box::new(SequentialStrategy::new()),
            Box::new(ParallelStrategy::new()),
            Box::new(StreamingStrategy::new()),
        ];

        Self {
            strategies,
            selector: StrategySelector::new(),
        }
    }

    /// Create with custom strategies
    pub fn with_strategies(strategies: Vec<Box<dyn ProcessingStrategy>>) -> Self {
        Self {
            strategies,
            selector: StrategySelector::new(),
        }
    }

    /// Select and execute the optimal strategy
    fn select_and_execute(
        &self,
        input: StrategyInput,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
        characteristics: &InputCharacteristics,
    ) -> Result<StrategyOutput> {
        // Select optimal strategy
        let strategy = self
            .selector
            .select_strategy(characteristics, &self.strategies);

        // Get optimal config for selected strategy
        let optimal_config = strategy.optimal_config(characteristics);

        // Merge with provided config (user config takes precedence)
        let merged_config = ProcessingConfig {
            chunk_size: if config.chunk_size != ProcessingConfig::default().chunk_size {
                config.chunk_size
            } else {
                optimal_config.chunk_size
            },
            thread_count: if config.thread_count != ProcessingConfig::default().thread_count {
                config.thread_count
            } else {
                optimal_config.thread_count
            },
            buffer_size: if config.buffer_size != ProcessingConfig::default().buffer_size {
                config.buffer_size
            } else {
                optimal_config.buffer_size
            },
            ..optimal_config
        };

        // Process with selected strategy
        strategy.process(input, language_rules, &merged_config)
    }
}

impl Default for AdaptiveStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessingStrategy for AdaptiveStrategy {
    fn process(
        &self,
        input: StrategyInput,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<StrategyOutput> {
        let start = Instant::now();

        // Analyze input characteristics
        let characteristics = match &input {
            StrategyInput::Text(text) => InputCharacteristics::from_text(text),
            StrategyInput::File(path) => {
                let metadata = std::fs::metadata(path).map_err(|e| {
                    ProcessingError::Other(format!("Failed to read file metadata: {e}"))
                })?;
                InputCharacteristics::from_file_metadata(&metadata)
            }
            StrategyInput::Stream(_) => InputCharacteristics::streaming(),
            StrategyInput::Chunks(chunks) => {
                let total_size: usize = chunks.iter().map(|c| c.len()).sum();
                InputCharacteristics {
                    size_bytes: total_size,
                    estimated_char_count: total_size / 3,
                    is_streaming: false,
                    available_memory: InputCharacteristics::from_text("").available_memory,
                    cpu_count: num_cpus::get(),
                    language_hint: None,
                }
            }
        };

        // Select and execute
        let result = self.select_and_execute(input, language_rules, config, &characteristics)?;

        let _elapsed = start.elapsed();
        // TODO: Collect metrics

        Ok(result)
    }

    fn suitability_score(&self, _characteristics: &InputCharacteristics) -> f32 {
        // Adaptive strategy is always suitable
        1.0
    }

    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig {
        // Delegate to selector
        let strategy = self
            .selector
            .select_strategy(characteristics, &self.strategies);
        strategy.optimal_config(characteristics)
    }

    fn supports_streaming(&self) -> bool {
        // Support if any sub-strategy supports it
        self.strategies.iter().any(|s| s.supports_streaming())
    }

    fn supports_parallel(&self) -> bool {
        // Support if any sub-strategy supports it
        self.strategies.iter().any(|s| s.supports_parallel())
    }

    fn name(&self) -> &'static str {
        "adaptive"
    }
}

/// Metrics collected during adaptive processing
#[derive(Debug, Default, Clone)]
pub struct AdaptiveProcessingMetrics {
    pub strategy_selected: String,
    pub selection_reason: String,
    pub bytes_processed: usize,
    pub processing_time_ms: u64,
    pub throughput_mb_per_sec: f64,
    pub thread_count: usize,
    pub chunk_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_strategy_creation() {
        let strategy = AdaptiveStrategy::new();
        assert_eq!(strategy.name(), "adaptive");
        assert!(strategy.supports_streaming());
        assert!(strategy.supports_parallel());
    }

    #[test]
    fn test_suitability_score() {
        let strategy = AdaptiveStrategy::new();
        let characteristics = InputCharacteristics::from_text("test");
        assert_eq!(strategy.suitability_score(&characteristics), 1.0);
    }
}
