//! Strategy selection logic

use super::traits::{InputCharacteristics, ProcessingStrategy, StrategySelection, StrategyType};

/// Selects the optimal processing strategy based on input characteristics
pub struct StrategySelector {
    /// Minimum score threshold for strategy selection
    min_score_threshold: f32,
    /// Whether to enable debug logging
    debug_mode: bool,
}

impl StrategySelector {
    /// Create a new strategy selector
    pub fn new() -> Self {
        Self {
            min_score_threshold: 0.3,
            debug_mode: false,
        }
    }

    /// Create a selector with debug mode enabled
    pub fn with_debug(mut self) -> Self {
        self.debug_mode = true;
        self
    }

    /// Select the best strategy for given input characteristics
    pub fn select_strategy<'a>(
        &self,
        characteristics: &InputCharacteristics,
        strategies: &'a [Box<dyn ProcessingStrategy>],
    ) -> &'a dyn ProcessingStrategy {
        let mut best_strategy = &strategies[0];
        let mut best_score = 0.0;
        let mut best_reason = String::new();

        for strategy in strategies {
            let score = strategy.suitability_score(characteristics);
            let reason = self.get_selection_reason(strategy.name(), characteristics, score);

            if self.debug_mode {
                eprintln!(
                    "Strategy '{}' scored {:.2} for input size {}",
                    strategy.name(),
                    score,
                    characteristics.size_bytes
                );
            }

            if score > best_score && score >= self.min_score_threshold {
                best_score = score;
                best_strategy = strategy;
                best_reason = reason;
            }
        }

        if self.debug_mode {
            eprintln!(
                "Selected strategy '{}' with score {:.2}: {}",
                best_strategy.name(),
                best_score,
                best_reason
            );
        }

        best_strategy.as_ref()
    }

    /// Get detailed strategy selection information
    pub fn analyze_selection(
        &self,
        characteristics: &InputCharacteristics,
        strategies: &[Box<dyn ProcessingStrategy>],
    ) -> StrategySelection {
        let selected = self.select_strategy(characteristics, strategies);
        let score = selected.suitability_score(characteristics);
        let reason = self.get_selection_reason(selected.name(), characteristics, score);

        let strategy_type = match selected.name() {
            "sequential" => StrategyType::Sequential,
            "parallel" => StrategyType::Parallel,
            "streaming" => StrategyType::Streaming,
            "adaptive" => StrategyType::Adaptive,
            _ => StrategyType::Sequential,
        };

        StrategySelection {
            strategy_type,
            score,
            reason,
        }
    }

    /// Get human-readable reason for strategy selection
    fn get_selection_reason(
        &self,
        strategy_name: &str,
        characteristics: &InputCharacteristics,
        score: f32,
    ) -> String {
        match strategy_name {
            "sequential" => {
                if characteristics.is_small() {
                    format!(
                        "Sequential strategy selected for small input ({} bytes)",
                        characteristics.size_bytes
                    )
                } else {
                    format!("Sequential strategy selected (score: {score:.2})")
                }
            }
            "parallel" => {
                if characteristics.would_benefit_from_parallel() {
                    format!(
                        "Parallel strategy selected for {} bytes with {} CPU cores",
                        characteristics.size_bytes, characteristics.cpu_count
                    )
                } else {
                    format!("Parallel strategy selected (score: {score:.2})")
                }
            }
            "streaming" => {
                if characteristics.requires_streaming() {
                    if characteristics.is_streaming {
                        "Streaming strategy selected for stream input".to_string()
                    } else {
                        format!(
                            "Streaming strategy selected for large input ({} bytes > {} memory/4)",
                            characteristics.size_bytes, characteristics.available_memory
                        )
                    }
                } else {
                    format!("Streaming strategy selected (score: {score:.2})")
                }
            }
            "adaptive" => "Adaptive strategy selected".to_string(),
            _ => format!("{strategy_name} strategy selected (score: {score:.2})"),
        }
    }
}

impl Default for StrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::strategies::{ParallelStrategy, SequentialStrategy};

    #[test]
    fn test_strategy_selection() {
        let strategies: Vec<Box<dyn ProcessingStrategy>> = vec![
            Box::new(SequentialStrategy::new()),
            Box::new(ParallelStrategy::new()),
        ];

        let selector = StrategySelector::new();

        // Small file should select sequential
        let small_chars = InputCharacteristics {
            size_bytes: 50_000,
            estimated_char_count: 50_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
            language_hint: None,
        };
        let selected = selector.select_strategy(&small_chars, &strategies);
        assert_eq!(selected.name(), "sequential");

        // Large file should select parallel
        let large_chars = InputCharacteristics {
            size_bytes: 50_000_000,
            estimated_char_count: 50_000_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
            language_hint: None,
        };
        let selected = selector.select_strategy(&large_chars, &strategies);
        assert_eq!(selected.name(), "parallel");
    }

    #[test]
    fn test_analyze_selection() {
        let strategies: Vec<Box<dyn ProcessingStrategy>> = vec![
            Box::new(SequentialStrategy::new()),
            Box::new(ParallelStrategy::new()),
        ];

        let selector = StrategySelector::new();
        let characteristics = InputCharacteristics::from_text("small text");

        let selection = selector.analyze_selection(&characteristics, &strategies);
        assert!(matches!(selection.strategy_type, StrategyType::Sequential));
        assert!(selection.score > 0.0);
        assert!(!selection.reason.is_empty());
    }
}
