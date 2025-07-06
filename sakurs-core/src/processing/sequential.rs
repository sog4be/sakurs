//! Sequential processing strategy for small texts

use super::strategy::{InputCharacteristics, ProcessingConfig, ProcessingStrategy};
use crate::application::{config::ProcessingResult as Result, UnifiedProcessor};
use std::sync::Arc;

/// Sequential processing strategy optimized for small texts
pub struct SequentialStrategy {
    language_rules: Arc<dyn crate::domain::language::LanguageRules>,
}

impl SequentialStrategy {
    /// Create a new sequential strategy
    pub fn new(language_rules: Arc<dyn crate::domain::language::LanguageRules>) -> Self {
        Self { language_rules }
    }
}

impl ProcessingStrategy for SequentialStrategy {
    fn process(&self, text: &str, _config: &ProcessingConfig) -> Result<Vec<usize>> {
        // For sequential processing, we process the entire text at once
        let processor = UnifiedProcessor::new(self.language_rules.clone());
        let output = processor.process(text)?;
        Ok(output.boundaries.into_iter().map(|b| b.offset).collect())
    }

    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32 {
        if characteristics.is_small() {
            1.0 // Perfect for small files
        } else if characteristics.is_medium() {
            0.3 // Can handle but not optimal
        } else {
            0.1 // Not suitable for large files
        }
    }

    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig {
        ProcessingConfig {
            chunk_size: characteristics.size_bytes, // Process entire file
            thread_count: 1,
            buffer_size: 0,       // No buffering needed
            prefetch_distance: 0, // No prefetching
        }
    }

    fn name(&self) -> &'static str {
        "Sequential"
    }
}
