//! Sequential processing strategy for small texts

use super::traits::{
    InputCharacteristics, ProcessingConfig, ProcessingStrategy, StrategyInput, StrategyOutput,
};
use crate::application::{
    config::{ProcessingError, ProcessingResult as Result},
    parser::TextParser,
};
use crate::domain::{
    language::LanguageRules, prefix_sum::PrefixSumComputer, reduce::BoundaryReducer,
    state::PartialState,
};
use std::sync::Arc;

/// Sequential processing strategy optimized for small texts
pub struct SequentialStrategy {
    parser: TextParser,
}

impl SequentialStrategy {
    /// Create a new sequential strategy
    pub fn new() -> Self {
        Self {
            parser: TextParser::new(),
        }
    }

    /// Process text sequentially using the domain layer directly
    fn process_text(
        &self,
        text: &str,
        language_rules: Arc<dyn LanguageRules>,
    ) -> Result<Vec<usize>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        // Phase 1: Scan
        let state = self.parser.scan_chunk(text, language_rules.as_ref());

        // For single chunk, no prefix sum needed
        // Phase 3: Reduce
        let boundaries = BoundaryReducer::reduce_single(&state);

        Ok(boundaries.into_iter().map(|b| b.offset).collect())
    }

    /// Process file by loading it into memory
    fn process_file(
        &self,
        path: std::path::PathBuf,
        language_rules: Arc<dyn LanguageRules>,
    ) -> Result<Vec<usize>> {
        let text =
            std::fs::read_to_string(&path).map_err(|e| ProcessingError::IoError { source: e })?;
        self.process_text(&text, language_rules)
    }

    /// Process pre-chunked text
    fn process_chunks(
        &self,
        chunks: Vec<&str>,
        language_rules: Arc<dyn LanguageRules>,
    ) -> Result<Vec<usize>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        if chunks.len() == 1 {
            // Single chunk, process directly
            return self.process_text(chunks[0], language_rules);
        }

        // Multiple chunks need full three-phase processing
        // Phase 1: Scan all chunks
        let states: Vec<PartialState> = chunks
            .iter()
            .map(|chunk| self.parser.scan_chunk(chunk, language_rules.as_ref()))
            .collect();

        // Phase 2: Prefix sum
        let chunk_starts = PrefixSumComputer::compute_prefix_sum(&states);

        // Phase 3: Reduce
        let boundaries = BoundaryReducer::reduce_all(&states, &chunk_starts);

        Ok(boundaries.into_iter().map(|b| b.offset).collect())
    }
}

impl Default for SequentialStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessingStrategy for SequentialStrategy {
    fn process(
        &self,
        input: StrategyInput,
        language_rules: Arc<dyn LanguageRules>,
        _config: &ProcessingConfig,
    ) -> Result<StrategyOutput> {
        let boundaries = match input {
            StrategyInput::Text(text) => self.process_text(text, language_rules)?,
            StrategyInput::File(path) => self.process_file(path, language_rules)?,
            StrategyInput::Stream(_) => {
                return Err(ProcessingError::Other(
                    "Sequential strategy doesn't support streaming".to_string(),
                ));
            }
            StrategyInput::Chunks(chunks) => {
                let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_ref()).collect();
                self.process_chunks(chunk_refs, language_rules)?
            }
        };

        Ok(StrategyOutput::Boundaries(boundaries))
    }

    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32 {
        if characteristics.is_streaming {
            0.0 // Can't handle streaming
        } else if characteristics.is_small() {
            1.0 // Perfect for small texts
        } else if characteristics.is_medium() {
            0.6 // OK for medium texts
        } else {
            0.2 // Too slow for large texts
        }
    }

    fn optimal_config(&self, _characteristics: &InputCharacteristics) -> ProcessingConfig {
        ProcessingConfig::sequential()
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn supports_parallel(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "sequential"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::EnglishLanguageRules;

    #[test]
    fn test_sequential_strategy() {
        let strategy = SequentialStrategy::new();
        let language_rules = Arc::new(EnglishLanguageRules::new());

        let text = "This is a sentence. And another one!";
        let result = strategy.process(
            StrategyInput::Text(text),
            language_rules,
            &ProcessingConfig::sequential(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            StrategyOutput::Boundaries(boundaries) => {
                assert_eq!(boundaries.len(), 2);
                assert_eq!(boundaries[0], 19); // After "sentence."
                assert_eq!(boundaries[1], 36); // After "one!"
            }
            _ => panic!("Expected Boundaries output"),
        }
    }

    #[test]
    fn test_sequential_chunks() {
        let strategy = SequentialStrategy::new();
        let language_rules = Arc::new(EnglishLanguageRules::new());

        let chunks = vec!["First chunk. ", "Second chunk."];

        let result = strategy.process(
            StrategyInput::Chunks(chunks),
            language_rules,
            &ProcessingConfig::sequential(),
        );

        assert!(result.is_ok());
        match result.unwrap() {
            StrategyOutput::Boundaries(boundaries) => {
                assert_eq!(boundaries.len(), 2);
            }
            _ => panic!("Expected Boundaries output"),
        }
    }

    #[test]
    fn test_sequential_suitability() {
        let strategy = SequentialStrategy::new();

        // Small text
        let small = InputCharacteristics::from_text("small");
        assert_eq!(strategy.suitability_score(&small), 1.0);

        // Large text
        let large = InputCharacteristics {
            size_bytes: 20_000_000, // 20MB, clearly large
            estimated_char_count: 20_000_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
            language_hint: None,
        };
        assert!(strategy.suitability_score(&large) < 0.5);
    }
}
