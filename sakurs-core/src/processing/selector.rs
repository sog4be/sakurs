//! Strategy selection logic

use super::strategy::{InputCharacteristics, ProcessingStrategy};

/// Selects the optimal processing strategy based on input characteristics
pub struct StrategySelector {
    /// Minimum score threshold for strategy selection
    min_score_threshold: f32,
}

impl StrategySelector {
    /// Create a new strategy selector
    pub fn new() -> Self {
        Self {
            min_score_threshold: 0.3,
        }
    }

    /// Select the best strategy for given input characteristics
    pub fn select_strategy<'a>(
        &self,
        characteristics: &InputCharacteristics,
        strategies: &'a [Box<dyn ProcessingStrategy>],
    ) -> &'a dyn ProcessingStrategy {
        let mut best_strategy = &strategies[0];
        let mut best_score = 0.0;

        for strategy in strategies {
            let score = strategy.suitability_score(characteristics);

            if score > best_score && score >= self.min_score_threshold {
                best_score = score;
                best_strategy = strategy;
            }
        }

        best_strategy.as_ref()
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
    use crate::processing::{parallel::ParallelStrategy, sequential::SequentialStrategy};

    #[test]
    fn test_strategy_selection() {
        use crate::domain::language::EnglishLanguageRules;
        use std::sync::Arc;

        let language_rules = Arc::new(EnglishLanguageRules::new());
        let strategies: Vec<Box<dyn ProcessingStrategy>> = vec![
            Box::new(SequentialStrategy::new(language_rules.clone())),
            Box::new(ParallelStrategy::new(language_rules)),
        ];

        let selector = StrategySelector::new();

        // Small file should select sequential
        let small_chars = InputCharacteristics {
            size_bytes: 50_000,
            estimated_char_count: 50_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
        };
        let selected = selector.select_strategy(&small_chars, &strategies);
        assert_eq!(selected.name(), "Sequential");

        // Large file should select parallel
        let large_chars = InputCharacteristics {
            size_bytes: 50_000_000,
            estimated_char_count: 50_000_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
        };
        let selected = selector.select_strategy(&large_chars, &strategies);
        assert_eq!(selected.name(), "Parallel");
    }
}
