//! Parallel processing coordination
//!
//! This module provides parallel processing capabilities using rayon,
//! with proper fallback for environments that don't support parallelism.

use crate::application::{
    chunking::TextChunk,
    config::{ProcessingError, ProcessingResult, ThreadPoolConfig},
};
use crate::domain::{language::LanguageRules, parser::Parser, state::PartialState};
use rayon::prelude::*;
use std::sync::Arc;

/// Manages parallel execution of text processing
pub struct ParallelProcessor {
    /// Thread pool for parallel execution
    thread_pool: Arc<rayon::ThreadPool>,

    /// Minimum chunk count to justify parallel processing
    min_chunks_for_parallel: usize,

    /// Parser instance
    parser: Parser,
}

impl ParallelProcessor {
    /// Creates a new parallel processor with default configuration
    pub fn new() -> ProcessingResult<Self> {
        Self::with_config(ThreadPoolConfig::default())
    }

    /// Creates a new parallel processor with custom configuration
    pub fn with_config(config: ThreadPoolConfig) -> ProcessingResult<Self> {
        let mut pool_builder = rayon::ThreadPoolBuilder::new()
            .num_threads(config.num_threads)
            .thread_name(move |i| format!("{}-{i}", config.thread_name_prefix));

        if let Some(stack_size) = config.stack_size {
            pool_builder = pool_builder.stack_size(stack_size);
        }

        let thread_pool = pool_builder
            .build()
            .map_err(|e| ProcessingError::ParallelError {
                source: Box::new(e),
            })?;

        Ok(Self {
            thread_pool: Arc::new(thread_pool),
            min_chunks_for_parallel: 2,
            parser: Parser::new(),
        })
    }

    /// Processes chunks in parallel
    pub fn process_chunks(
        &self,
        chunks: Vec<TextChunk>,
        language_rules: Arc<dyn LanguageRules>,
    ) -> ProcessingResult<Vec<PartialState>> {
        if chunks.len() < self.min_chunks_for_parallel {
            // Fall back to sequential processing for small chunk counts
            return self.process_chunks_sequential(chunks, language_rules);
        }

        let parser = &self.parser;

        let states = self.thread_pool.install(|| {
            chunks
                .into_par_iter()
                .map(|chunk| {
                    parser.parse_chunk(
                        &chunk.content,
                        language_rules.as_ref(),
                        None, // No carry state in parallel mode
                    )
                })
                .collect::<Vec<_>>()
        });

        Ok(states)
    }

    /// Sequential fallback for small chunk counts
    fn process_chunks_sequential(
        &self,
        chunks: Vec<TextChunk>,
        language_rules: Arc<dyn LanguageRules>,
    ) -> ProcessingResult<Vec<PartialState>> {
        let mut states = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let state = self
                .parser
                .parse_chunk(&chunk.content, language_rules.as_ref(), None);
            states.push(state);
        }

        Ok(states)
    }

    /// Checks if parallel processing would be beneficial
    pub fn is_parallel_beneficial(&self, text_size: usize, chunk_count: usize) -> bool {
        // Heuristics for deciding when parallel processing is worth it
        if chunk_count < self.min_chunks_for_parallel {
            return false;
        }

        // Consider overhead vs benefit
        let avg_chunk_size = text_size / chunk_count;
        let overhead_threshold = 4096; // 4KB minimum chunk size

        avg_chunk_size >= overhead_threshold
            && chunk_count >= self.thread_pool.current_num_threads()
    }

    /// Returns the number of threads in the pool
    pub fn thread_count(&self) -> usize {
        self.thread_pool.current_num_threads()
    }

    /// Processes chunks with adaptive parallelism
    pub fn process_adaptive(
        &self,
        chunks: Vec<TextChunk>,
        language_rules: Arc<dyn LanguageRules>,
        text_size: usize,
    ) -> ProcessingResult<Vec<PartialState>> {
        if self.is_parallel_beneficial(text_size, chunks.len()) {
            self.process_chunks(chunks, language_rules)
        } else {
            self.process_chunks_sequential(chunks, language_rules)
        }
    }
}

impl Default for ParallelProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create default parallel processor")
    }
}

/// Utilities for parallel text processing
pub mod utils {
    use super::*;
    use crate::domain::Monoid;

    /// Parallel reduction of partial states using tree reduction
    pub fn parallel_reduce_states(states: Vec<PartialState>) -> PartialState {
        if states.is_empty() {
            return PartialState::identity();
        }

        if states.len() == 1 {
            return states.into_iter().next().unwrap();
        }

        // Use parallel tree reduction for efficiency
        states
            .into_par_iter()
            .reduce(PartialState::identity, |a, b| a.combine(&b))
    }

    /// Partitions work across available threads
    pub fn partition_work(total_items: usize, thread_count: usize) -> Vec<(usize, usize)> {
        let chunk_size = total_items.div_ceil(thread_count);
        let mut partitions = Vec::with_capacity(thread_count);

        for i in 0..thread_count {
            let start = i * chunk_size;
            if start >= total_items {
                break;
            }
            let end = ((i + 1) * chunk_size).min(total_items);
            partitions.push((start, end));
        }

        partitions
    }

    /// Estimates optimal chunk size based on cache considerations
    pub fn estimate_optimal_chunk_size(
        total_size: usize,
        thread_count: usize,
        l2_cache_size: usize,
    ) -> usize {
        // Aim for chunks that fit in L2 cache
        let cache_optimal = l2_cache_size / 2; // Leave room for other data

        // But also consider even distribution across threads
        let thread_optimal = total_size / thread_count;

        // Choose the smaller to ensure cache efficiency
        cache_optimal.min(thread_optimal).max(4096) // At least 4KB
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::MockLanguageRules;

    #[test]
    fn test_parallel_processor_creation() {
        let processor = ParallelProcessor::new();
        assert!(processor.is_ok());

        let processor = processor.unwrap();
        assert!(processor.thread_count() > 0);
    }

    #[test]
    fn test_parallel_beneficial_heuristics() {
        let processor = ParallelProcessor::new().unwrap();
        let thread_count = processor.thread_count();

        // Small text, few chunks - not beneficial (less than min_chunks_for_parallel)
        assert!(!processor.is_parallel_beneficial(1024, 1));

        // Large text, many chunks - beneficial if we have enough chunks for all threads
        let many_chunks = thread_count * 2;
        assert!(processor.is_parallel_beneficial(1024 * 1024, many_chunks));

        // Small chunks (less than 4KB average) - not beneficial
        let small_chunk_size = 2048; // 2KB average
        assert!(!processor.is_parallel_beneficial(small_chunk_size * 2, 2));

        // Chunks equal to thread count with sufficient size - beneficial
        let sufficient_size = 4096 * thread_count;
        assert!(processor.is_parallel_beneficial(sufficient_size, thread_count));
    }

    #[test]
    fn test_parallel_chunk_processing() {
        let processor = ParallelProcessor::new().unwrap();
        let rules = Arc::new(MockLanguageRules::english());

        let chunks = vec![
            TextChunk {
                content: "First chunk.".to_string(),
                start_offset: 0,
                end_offset: 12,
                has_prefix_overlap: false,
                has_suffix_overlap: false,
                index: 0,
                total_chunks: 2,
            },
            TextChunk {
                content: "Second chunk.".to_string(),
                start_offset: 12,
                end_offset: 25,
                has_prefix_overlap: false,
                has_suffix_overlap: false,
                index: 1,
                total_chunks: 2,
            },
        ];

        let states = processor.process_chunks(chunks, rules).unwrap();
        assert_eq!(states.len(), 2);
    }

    #[test]
    fn test_work_partitioning() {
        let partitions = utils::partition_work(100, 4);
        assert_eq!(partitions.len(), 4);
        assert_eq!(partitions[0], (0, 25));
        assert_eq!(partitions[1], (25, 50));
        assert_eq!(partitions[2], (50, 75));
        assert_eq!(partitions[3], (75, 100));

        // Uneven distribution
        let partitions = utils::partition_work(10, 3);
        assert_eq!(partitions.len(), 3);
        assert_eq!(partitions[0], (0, 4));
        assert_eq!(partitions[1], (4, 8));
        assert_eq!(partitions[2], (8, 10));
    }

    #[test]
    fn test_optimal_chunk_size() {
        let l2_cache = 256 * 1024; // 256KB L2 cache

        // Small text - should return minimum
        let chunk_size = utils::estimate_optimal_chunk_size(8 * 1024, 4, l2_cache);
        assert_eq!(chunk_size, 4096);

        // Large text - should consider cache
        let chunk_size = utils::estimate_optimal_chunk_size(10 * 1024 * 1024, 4, l2_cache);
        assert_eq!(chunk_size, l2_cache / 2);
    }
}
