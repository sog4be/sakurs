//! Parallel processing strategy for medium to large texts

use super::strategy::{InputCharacteristics, ProcessingConfig, ProcessingStrategy};
use crate::application::{
    config::{ProcessingError, ProcessingResult as Result},
    UnifiedProcessor,
};
use std::sync::Arc;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Parallel processing strategy using Rayon
pub struct ParallelStrategy {
    language_rules: Arc<dyn crate::domain::language::LanguageRules>,
}

impl ParallelStrategy {
    /// Create a new parallel strategy
    pub fn new(language_rules: Arc<dyn crate::domain::language::LanguageRules>) -> Self {
        Self { language_rules }
    }

    #[cfg(feature = "parallel")]
    fn process_parallel(&self, text: &str, config: &ProcessingConfig) -> Result<Vec<usize>> {
        // Set up thread pool with configured thread count
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.thread_count)
            .build()
            .map_err(|e| ProcessingError::ParallelError {
                source: Box::new(std::io::Error::other(e.to_string())),
            })?;

        let result = pool.install(|| {
            // Split text into chunks by character boundaries
            let chunks: Vec<&str> = text
                .char_indices()
                .step_by(config.chunk_size)
                .map(|(i, _)| i)
                .chain(std::iter::once(text.len()))
                .collect::<Vec<_>>()
                .windows(2)
                .map(|w| &text[w[0]..w[1]])
                .collect();

            // Process chunks in parallel
            let language_rules = self.language_rules.clone();
            let chunk_results: Vec<_> = chunks
                .par_iter()
                .map(|chunk| {
                    let processor = UnifiedProcessor::new(language_rules.clone());
                    processor.process(chunk).map(|output| {
                        output
                            .boundaries
                            .into_iter()
                            .map(|b| b.offset)
                            .collect::<Vec<_>>()
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            // Merge results with proper offset adjustments
            let mut all_boundaries = Vec::new();
            let mut offset = 0;

            for (i, boundaries) in chunk_results.into_iter().enumerate() {
                for boundary in boundaries {
                    all_boundaries.push(boundary + offset);
                }
                if i < chunks.len() - 1 {
                    offset += chunks[i].len();
                }
            }

            // Sort and deduplicate
            all_boundaries.sort_unstable();
            all_boundaries.dedup();

            Ok(all_boundaries)
        });

        result
    }

    #[cfg(not(feature = "parallel"))]
    fn process_parallel(&self, text: &str, _config: &ProcessingConfig) -> Result<Vec<usize>> {
        // Fallback to sequential if parallel feature is disabled
        let processor = UnifiedProcessor::new(self.language_rules.clone());
        let output = processor.process(text)?;
        Ok(output.boundaries.into_iter().map(|b| b.offset).collect())
    }
}

impl ProcessingStrategy for ParallelStrategy {
    fn process(&self, text: &str, config: &ProcessingConfig) -> Result<Vec<usize>> {
        self.process_parallel(text, config)
    }

    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32 {
        if characteristics.is_medium() {
            0.9 // Very good for medium files
        } else if characteristics.is_large() {
            1.0 // Perfect for large files
        } else if characteristics.is_small() {
            0.2 // Overhead not worth it
        } else {
            0.7 // Good for very large, but streaming might be better
        }
    }

    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig {
        let thread_count = if characteristics.size_bytes < 75_000 {
            1 // <75KB: single thread (shouldn't use parallel at all)
        } else if characteristics.size_bytes < 200_000 {
            2 // 75-200KB: 2 threads optimal
        } else if characteristics.size_bytes < 1_000_000 {
            4 // 200KB-1MB: 4 threads optimal
        } else if characteristics.size_bytes < 10_000_000 {
            (characteristics.cpu_count - 1).clamp(4, 6) // 1-10MB: 4-6 threads
        } else {
            (characteristics.cpu_count - 1).clamp(6, 8) // >10MB: 6-8 threads
        };

        let chunk_size = if characteristics.size_bytes < 200_000 {
            32_768 // 32KB for small files (better cache locality)
        } else if characteristics.size_bytes < 1_000_000 {
            65_536 // 64KB for medium files
        } else if characteristics.size_bytes < 10_000_000 {
            262_144 // 256KB for large files
        } else {
            524_288 // 512KB for very large files
        };

        ProcessingConfig {
            chunk_size,
            thread_count,
            buffer_size: 0, // Not used in parallel
            prefetch_distance: 32,
        }
    }

    fn name(&self) -> &'static str {
        "Parallel"
    }
}
