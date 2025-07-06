//! Adaptive processor that automatically selects optimal strategy

use super::{
    parallel::ParallelStrategy, sequential::SequentialStrategy, streaming::StreamingStrategy,
    InputCharacteristics, ProcessingStrategy, StrategySelector,
};
use crate::application::config::ProcessingResult as Result;
use std::time::Instant;

/// Processor that adapts its strategy based on input characteristics
pub struct AdaptiveProcessor {
    strategies: Vec<Box<dyn ProcessingStrategy>>,
    selector: StrategySelector,
}

impl AdaptiveProcessor {
    /// Create a new adaptive processor with default strategies
    pub fn new(language_rules: std::sync::Arc<dyn crate::domain::language::LanguageRules>) -> Self {
        let strategies: Vec<Box<dyn ProcessingStrategy>> = vec![
            Box::new(SequentialStrategy::new(language_rules.clone())),
            Box::new(ParallelStrategy::new(language_rules.clone())),
            Box::new(StreamingStrategy::new(language_rules)),
        ];

        Self {
            strategies,
            selector: StrategySelector::new(),
        }
    }

    /// Process text with automatically selected strategy
    pub fn process(&self, text: &str) -> Result<Vec<usize>> {
        let start = Instant::now();

        // Analyze input characteristics
        let characteristics = InputCharacteristics::from_text(text);

        // Select optimal strategy
        let strategy = self
            .selector
            .select_strategy(&characteristics, &self.strategies);
        let config = strategy.optimal_config(&characteristics);

        // Process with selected strategy
        let result = strategy.process(text, &config)?;

        let _elapsed = start.elapsed();

        Ok(result)
    }

    /// Get performance metrics for the last processing
    pub fn get_metrics(&self) -> ProcessingMetrics {
        // TODO: Implement metrics collection
        ProcessingMetrics::default()
    }
}

/// Metrics collected during processing
#[derive(Debug, Default, Clone)]
pub struct ProcessingMetrics {
    pub strategy_used: String,
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
    fn test_adaptive_processing() {
        use crate::domain::language::EnglishLanguageRules;
        use std::sync::Arc;

        let language_rules = Arc::new(EnglishLanguageRules::new());
        let adaptive = AdaptiveProcessor::new(language_rules);

        // Test small text
        let small_text = "Hello world. This is a test.";
        let result = adaptive.process(small_text).unwrap();
        assert!(!result.is_empty());

        // Test medium text
        let medium_text = "Hello world. ".repeat(10_000);
        let result = adaptive.process(&medium_text).unwrap();
        assert!(!result.is_empty());
    }
}
