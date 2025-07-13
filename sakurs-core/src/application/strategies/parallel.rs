//! Parallel processing strategy for medium to large texts

use super::traits::{
    InputCharacteristics, ProcessingConfig, ProcessingStrategy, StrategyInput, StrategyOutput,
};
use crate::application::{
    chunking::ChunkManager,
    config::{ProcessingError, ProcessingResult as Result},
    parser::TextParser,
};
use crate::domain::{
    language::LanguageRules, prefix_sum::PrefixSumComputer, reduce::BoundaryReducer,
    types::PartialState,
};
use std::sync::Arc;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Parallel processing strategy using Rayon
pub struct ParallelStrategy {
    parser: TextParser,
}

impl ParallelStrategy {
    /// Create a new parallel strategy
    pub fn new() -> Self {
        Self {
            parser: TextParser::new(),
        }
    }

    /// Process text in parallel using three-phase algorithm
    #[cfg(feature = "parallel")]
    fn process_text_parallel(
        &self,
        text: &str,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<Vec<usize>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        // Update chunk manager with config
        let chunk_manager = ChunkManager::new(config.chunk_size, config.overlap_size);

        // Set up thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.thread_count)
            .build()
            .map_err(|e| ProcessingError::ParallelError {
                source: Box::new(e),
            })?;

        pool.install(|| {
            // Create chunks with proper boundaries
            let chunks = chunk_manager.chunk_text(text)?;

            if chunks.is_empty() {
                return Ok(Vec::new());
            }

            if chunks.len() == 1 {
                // Single chunk, no need for parallel processing
                let state = self
                    .parser
                    .scan_chunk(&chunks[0].content, language_rules.as_ref());
                let boundaries = BoundaryReducer::reduce_single(&state);
                return Ok(boundaries.into_iter().map(|b| b.offset).collect());
            }

            // Phase 1: Parallel scan
            let states: Vec<PartialState> = chunks
                .par_iter()
                .map(|chunk| {
                    self.parser
                        .scan_chunk(&chunk.content, language_rules.as_ref())
                })
                .collect();

            // Phase 2: Sequential prefix sum (must be sequential)
            let chunk_starts = PrefixSumComputer::compute_prefix_sum_with_chunks(&states, &chunks);

            // Phase 3: Parallel reduce
            let boundaries = BoundaryReducer::reduce_all(&states, &chunk_starts);

            Ok(boundaries.into_iter().map(|b| b.offset).collect())
        })
    }

    /// Fallback to sequential processing when parallel feature is disabled
    #[cfg(not(feature = "parallel"))]
    fn process_text_parallel(
        &self,
        text: &str,
        language_rules: Arc<dyn LanguageRules>,
        _config: &ProcessingConfig,
    ) -> Result<Vec<usize>> {
        // Fallback to sequential processing
        let sequential = super::SequentialStrategy::new();
        sequential
            .process(
                StrategyInput::Text(text),
                language_rules,
                &ProcessingConfig::sequential(),
            )
            .and_then(|output| match output {
                StrategyOutput::Boundaries(boundaries) => Ok(boundaries),
                _ => Err(ProcessingError::Other("Unexpected output type".to_string())),
            })
    }
}

impl Default for ParallelStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessingStrategy for ParallelStrategy {
    fn process(
        &self,
        input: StrategyInput,
        language_rules: Arc<dyn LanguageRules>,
        config: &ProcessingConfig,
    ) -> Result<StrategyOutput> {
        let boundaries = match input {
            StrategyInput::Text(text) => {
                self.process_text_parallel(text, language_rules, config)?
            }
            StrategyInput::File(path) => {
                // Load file and process
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| ProcessingError::IoError { source: e })?;
                self.process_text_parallel(&text, language_rules, config)?
            }
            StrategyInput::Stream(_) => {
                return Err(ProcessingError::Other(
                    "Parallel strategy doesn't support streaming".to_string(),
                ));
            }
            StrategyInput::Chunks(chunks) => {
                // Process pre-chunked text
                let text = chunks.join("");
                self.process_text_parallel(&text, language_rules, config)?
            }
        };

        Ok(StrategyOutput::Boundaries(boundaries))
    }

    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32 {
        if characteristics.is_streaming {
            0.0 // Can't handle streaming
        } else if characteristics.would_benefit_from_parallel() {
            1.0 // Perfect for parallel processing
        } else if characteristics.is_medium() {
            0.8 // Good for medium files
        } else if characteristics.is_small() {
            0.3 // Overhead not worth it for small files
        } else {
            0.6 // OK for very large files, but streaming might be better
        }
    }

    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig {
        let thread_count = if characteristics.cpu_count > 1 {
            (characteristics.cpu_count / 2).clamp(2, 8) // Use half the cores, 2-8 threads
        } else {
            1
        };

        let chunk_size = if characteristics.size_bytes < 1_000_000 {
            65536 // 64KB for smaller files
        } else {
            131072 // 128KB for larger files
        };

        ProcessingConfig {
            chunk_size,
            thread_count,
            buffer_size: 0,    // No buffering needed
            overlap_size: 256, // Standard overlap
        }
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn supports_parallel(&self) -> bool {
        cfg!(feature = "parallel")
    }

    fn name(&self) -> &'static str {
        "parallel"
    }
}

#[cfg(all(test, feature = "parallel"))]
mod tests {
    use super::*;
    use crate::domain::language::EnglishLanguageRules;

    #[test]
    fn test_parallel_strategy() {
        let strategy = ParallelStrategy::new();
        let language_rules = Arc::new(EnglishLanguageRules::new());

        let text = "This is the first sentence. This is the second sentence. \
                    This is the third sentence. This is the fourth sentence.";

        let config = ProcessingConfig::parallel(2);
        let result = strategy.process(StrategyInput::Text(text), language_rules, &config);

        assert!(result.is_ok());
        match result.unwrap() {
            StrategyOutput::Boundaries(boundaries) => {
                assert_eq!(boundaries.len(), 4);
            }
            _ => panic!("Expected Boundaries output"),
        }
    }

    #[test]
    fn test_parallel_suitability() {
        let strategy = ParallelStrategy::new();

        // Large file with multiple cores
        let large = InputCharacteristics {
            size_bytes: 5_000_000,
            estimated_char_count: 5_000_000,
            is_streaming: false,
            available_memory: 1_073_741_824,
            cpu_count: 8,
            language_hint: None,
        };
        assert_eq!(strategy.suitability_score(&large), 1.0);

        // Small file
        let small = InputCharacteristics::from_text("small");
        assert!(strategy.suitability_score(&small) < 0.5);
    }
}
