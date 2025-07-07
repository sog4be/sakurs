//! Parallel processing coordination
//!
//! This module provides parallel processing capabilities using rayon,
//! with proper fallback for environments that don't support parallelism.

use crate::application::parser::TextParser;
use crate::application::{
    chunking::TextChunk,
    config::{ProcessingError, ProcessingResult, ThreadPoolConfig},
};
use crate::domain::{language::LanguageRules, state::PartialState};
use rayon::prelude::*;
use std::sync::Arc;

/// Manages parallel execution of text processing
pub struct ParallelProcessor {
    /// Thread pool for parallel execution
    thread_pool: Arc<rayon::ThreadPool>,

    /// Minimum chunk count to justify parallel processing
    min_chunks_for_parallel: usize,

    /// Parser instance
    parser: TextParser,
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
            parser: TextParser::new(),
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
                .map(|chunk| parser.scan_chunk(&chunk.content, language_rules.as_ref()))
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
                .scan_chunk(&chunk.content, language_rules.as_ref());
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
        // Handle edge case of zero threads
        if thread_count == 0 {
            return 4096; // Return minimum chunk size
        }

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
    use crate::domain::Monoid;

    #[test]
    fn test_parallel_processor_creation() {
        let processor = ParallelProcessor::new();
        assert!(processor.is_ok());

        let processor = processor.unwrap();
        assert!(processor.thread_count() > 0);
    }

    #[test]
    fn test_parallel_processor_with_custom_config() {
        let config = ThreadPoolConfig {
            num_threads: 2,
            thread_name_prefix: "test-worker".to_string(),
            stack_size: Some(2 * 1024 * 1024), // 2MB
        };

        let processor = ParallelProcessor::with_config(config);
        assert!(processor.is_ok());

        let processor = processor.unwrap();
        assert_eq!(processor.thread_count(), 2);
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

    #[test]
    fn test_parallel_reduce_states_empty() {
        // Test with empty vector
        let states = vec![];
        let result = utils::parallel_reduce_states(states);
        assert_eq!(result, PartialState::identity());
    }

    #[test]
    fn test_parallel_reduce_states_single() {
        // Test with single state
        let mut state = PartialState::new(1);
        state.add_boundary_candidate(10, vec![0].into(), crate::domain::BoundaryFlags::WEAK);
        state.chunk_length = 20;

        let states = vec![state.clone()];
        let result = utils::parallel_reduce_states(states);
        assert_eq!(
            result.boundary_candidates.len(),
            state.boundary_candidates.len()
        );
        assert_eq!(result.chunk_length, state.chunk_length);
    }

    #[test]
    fn test_partition_work_edge_cases() {
        // Test with 0 items
        let partitions = utils::partition_work(0, 4);
        assert!(partitions.is_empty());

        // Test with 1 item
        let partitions = utils::partition_work(1, 4);
        assert_eq!(partitions.len(), 1);
        assert_eq!(partitions[0], (0, 1));

        // Test with more threads than items
        let partitions = utils::partition_work(3, 10);
        assert_eq!(partitions.len(), 3);
        assert_eq!(partitions[0], (0, 1));
        assert_eq!(partitions[1], (1, 2));
        assert_eq!(partitions[2], (2, 3));

        // Test with items not evenly divisible
        let partitions = utils::partition_work(7, 3);
        assert_eq!(partitions.len(), 3);
        assert_eq!(partitions[0], (0, 3));
        assert_eq!(partitions[1], (3, 6));
        assert_eq!(partitions[2], (6, 7));
    }

    #[test]
    fn test_sequential_fallback_for_small_chunks() {
        let processor = ParallelProcessor::new().unwrap();
        let rules = Arc::new(MockLanguageRules::english());

        // Single chunk - should use sequential
        let chunks = vec![TextChunk {
            content: "Single chunk test.".to_string(),
            start_offset: 0,
            end_offset: 18,
            has_prefix_overlap: false,
            has_suffix_overlap: false,
            index: 0,
            total_chunks: 1,
        }];

        let states = processor.process_chunks(chunks, rules).unwrap();
        assert_eq!(states.len(), 1);
    }

    #[test]
    fn test_process_adaptive() {
        let processor = ParallelProcessor::new().unwrap();
        let rules = Arc::new(MockLanguageRules::english());

        // Small text - should use sequential
        let small_chunks = vec![TextChunk {
            content: "Small text.".to_string(),
            start_offset: 0,
            end_offset: 11,
            has_prefix_overlap: false,
            has_suffix_overlap: false,
            index: 0,
            total_chunks: 1,
        }];

        let result = processor
            .process_adaptive(small_chunks, rules.clone(), 11)
            .unwrap();
        assert_eq!(result.len(), 1);

        // Large text with many chunks - should use parallel
        let thread_count = processor.thread_count();
        let many_chunks: Vec<TextChunk> = (0..thread_count * 2)
            .map(|i| TextChunk {
                content: format!("Chunk number {}.", i),
                start_offset: i * 100,
                end_offset: (i + 1) * 100,
                has_prefix_overlap: false,
                has_suffix_overlap: false,
                index: i,
                total_chunks: thread_count * 2,
            })
            .collect();

        let total_size = 100 * 1024; // 100KB
        let result = processor
            .process_adaptive(many_chunks, rules, total_size)
            .unwrap();
        assert_eq!(result.len(), thread_count * 2);
    }

    #[test]
    fn test_estimate_optimal_chunk_size_edge_cases() {
        // Very small total size
        let chunk_size = utils::estimate_optimal_chunk_size(100, 4, 256 * 1024);
        assert_eq!(chunk_size, 4096); // Should return minimum

        // Zero threads (edge case)
        let chunk_size = utils::estimate_optimal_chunk_size(10000, 0, 256 * 1024);
        assert_eq!(chunk_size, 4096); // Should handle division by zero gracefully

        // Very large cache
        let chunk_size = utils::estimate_optimal_chunk_size(1024 * 1024, 4, 10 * 1024 * 1024);
        assert_eq!(chunk_size, 256 * 1024); // Should be limited by thread distribution
    }
}
