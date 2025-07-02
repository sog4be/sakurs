//! Unified text processor that implements the complete Δ-Stack Monoid algorithm.
//!
//! This module provides a unified interface that handles both sequential and
//! parallel processing using the three-phase algorithm: Scan, Prefix-sum, Reduce.

use crate::application::{
    chunking::{ChunkManager, TextChunk},
    config::{ProcessingError, ProcessingMetrics, ProcessingResult, ProcessorConfig},
};
use crate::domain::{
    language::LanguageRules,
    parser::Parser,
    prefix_sum::{ChunkStartState, PrefixSumComputer},
    reduce::BoundaryReducer,
    state::{Boundary, PartialState},
};
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

/// Processing output with boundaries and metrics.
#[derive(Debug, Clone)]
pub struct UnifiedProcessingOutput {
    pub boundaries: Vec<Boundary>,
    pub text_length: usize,
    pub metrics: ProcessingMetrics,
}

/// Unified processor implementing the complete Δ-Stack Monoid algorithm.
pub struct UnifiedProcessor {
    parser: Parser,
    chunk_manager: ChunkManager,
    language_rules: Arc<dyn LanguageRules>,
    config: ProcessorConfig,
}

impl UnifiedProcessor {
    /// Creates a new unified processor.
    pub fn new(language_rules: Arc<dyn LanguageRules>) -> Self {
        Self::with_config(language_rules, ProcessorConfig::default())
    }

    /// Creates a new processor with custom configuration.
    pub fn with_config(language_rules: Arc<dyn LanguageRules>, config: ProcessorConfig) -> Self {
        Self {
            parser: Parser::new(),
            chunk_manager: ChunkManager::new(config.chunk_size, config.overlap_size),
            language_rules,
            config,
        }
    }

    /// Processes text with specified number of threads.
    ///
    /// # Arguments
    /// * `text` - Input text to process
    /// * `thread_count` - Number of threads (1 for sequential, >1 for parallel)
    ///
    /// # Returns
    /// Processing output with detected boundaries and metrics
    pub fn process_with_threads(
        &self,
        text: &str,
        thread_count: usize,
    ) -> ProcessingResult<UnifiedProcessingOutput> {
        let start_time = Instant::now();
        let mut metrics = ProcessingMetrics::default();

        // Phase 0: Chunking
        let chunk_start = Instant::now();
        let chunks = self.chunk_manager.chunk_text(text)?;
        metrics.chunk_count = chunks.len();
        metrics.chunking_time_us = chunk_start.elapsed().as_micros() as u64;

        if chunks.is_empty() {
            return Ok(UnifiedProcessingOutput {
                boundaries: Vec::new(),
                text_length: 0,
                metrics,
            });
        }

        // Phase 1: Scan (parallel or sequential based on thread count)
        let scan_start = Instant::now();
        let partial_states = if thread_count > 1 && chunks.len() > 1 {
            self.scan_parallel(&chunks, thread_count)?
        } else {
            self.scan_sequential(&chunks)?
        };
        metrics.parallel_time_us = scan_start.elapsed().as_micros() as u64;

        // Phase 2: Prefix-sum (only needed for multiple chunks)
        let prefix_start = Instant::now();
        let chunk_starts = if partial_states.len() > 1 {
            PrefixSumComputer::compute_prefix_sum(&partial_states)
        } else {
            vec![ChunkStartState {
                cumulative_deltas: vec![
                    crate::domain::state::DeltaEntry { net: 0, min: 0 };
                    partial_states[0].deltas.len()
                ],
                global_offset: 0,
            }]
        };
        let prefix_time = prefix_start.elapsed().as_micros() as u64;

        // Phase 3: Reduce (parallel or sequential based on thread count)
        let reduce_start = Instant::now();
        let boundaries = if thread_count > 1 && partial_states.len() > 1 {
            BoundaryReducer::reduce_all(&partial_states, &chunk_starts)
        } else if partial_states.len() == 1 {
            BoundaryReducer::reduce_single(&partial_states[0])
        } else {
            BoundaryReducer::reduce_all(&partial_states, &chunk_starts)
        };
        metrics.merge_time_us = prefix_time + reduce_start.elapsed().as_micros() as u64;

        metrics.boundaries_found = boundaries.len();
        metrics.thread_count = thread_count;
        metrics.total_time_us = start_time.elapsed().as_micros() as u64;

        Ok(UnifiedProcessingOutput {
            boundaries,
            text_length: text.len(),
            metrics,
        })
    }

    /// Processes text using automatic thread count determination.
    pub fn process(&self, text: &str) -> ProcessingResult<UnifiedProcessingOutput> {
        let thread_count = self.determine_thread_count(text.len());
        self.process_with_threads(text, thread_count)
    }

    /// Scans chunks in parallel using rayon.
    fn scan_parallel(
        &self,
        chunks: &[TextChunk],
        thread_count: usize,
    ) -> ProcessingResult<Vec<PartialState>> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build()
            .map_err(|e| ProcessingError::ParallelError {
                source: Box::new(e),
            })?;

        let parser = &self.parser;
        let language_rules = &self.language_rules;

        let states = pool.install(|| {
            chunks
                .par_iter()
                .map(|chunk| parser.scan_chunk(&chunk.content, language_rules.as_ref()))
                .collect::<Vec<_>>()
        });

        Ok(states)
    }

    /// Scans chunks sequentially.
    fn scan_sequential(&self, chunks: &[TextChunk]) -> ProcessingResult<Vec<PartialState>> {
        let states = chunks
            .iter()
            .map(|chunk| {
                self.parser
                    .scan_chunk(&chunk.content, self.language_rules.as_ref())
            })
            .collect();

        Ok(states)
    }

    /// Determines optimal thread count based on text size and configuration.
    pub fn determine_thread_count(&self, text_size: usize) -> usize {
        // Use configured max threads if set
        let max_threads = self.config.max_threads.unwrap_or_else(num_cpus::get);

        // Heuristics for thread count
        if text_size < self.config.chunk_size / 4 {
            // Very small text - use sequential
            1
        } else if text_size < self.config.chunk_size * 4 {
            // Small text - use at most 2 threads
            2.min(max_threads)
        } else if text_size < self.config.chunk_size * 16 {
            // Medium text - use up to 4 threads
            4.min(max_threads)
        } else {
            // Large text - use all available threads
            max_threads
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::MockLanguageRules;

    #[test]
    fn test_empty_text_processing() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = UnifiedProcessor::new(rules);

        let result = processor.process("").unwrap();
        assert!(result.boundaries.is_empty());
        assert_eq!(result.text_length, 0);
    }

    #[test]
    fn test_sequential_processing() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = UnifiedProcessor::new(rules);

        let text = "Hello world. This is a test.";
        let result = processor.process_with_threads(text, 1).unwrap();

        assert!(!result.boundaries.is_empty());
        assert_eq!(result.text_length, text.len());
        assert_eq!(result.metrics.thread_count, 1);
    }

    #[test]
    fn test_parallel_processing() {
        let rules = Arc::new(MockLanguageRules::english());
        let mut config = ProcessorConfig::default();
        config.chunk_size = 10; // Small chunks to force multiple chunks
        let processor = UnifiedProcessor::with_config(rules, config);

        let text = "Hello world. This is a test. Another sentence here.";
        let result = processor.process_with_threads(text, 4).unwrap();

        assert!(!result.boundaries.is_empty());
        assert_eq!(result.text_length, text.len());
        assert_eq!(result.metrics.thread_count, 4);
    }

    #[test]
    fn test_thread_count_determination() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = UnifiedProcessor::new(rules);

        // Small text (less than chunk_size / 4)
        assert_eq!(processor.determine_thread_count(100), 1);

        // Medium text (between chunk_size and chunk_size * 4)
        // Default chunk_size is 8192, so 10_000 should give 2 threads
        let medium_count = processor.determine_thread_count(10_000);
        assert!(medium_count >= 1 && medium_count <= 2);

        // Large text
        let large_count = processor.determine_thread_count(1_000_000);
        assert!(large_count > 1);
    }

    #[test]
    fn test_consistent_results() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = UnifiedProcessor::new(rules);

        let text = "First sentence. Second sentence! Third sentence?";

        // Sequential processing
        let seq_result = processor.process_with_threads(text, 1).unwrap();

        // Parallel processing
        let par_result = processor.process_with_threads(text, 4).unwrap();

        // Results should be identical
        assert_eq!(seq_result.boundaries.len(), par_result.boundaries.len());
        for (s, p) in seq_result
            .boundaries
            .iter()
            .zip(par_result.boundaries.iter())
        {
            assert_eq!(s.offset, p.offset);
            assert_eq!(s.flags, p.flags);
        }
    }
}
