use std::sync::Arc;

use rayon::prelude::*;

use crate::{
    application::{
        chunking::{ChunkManager, TextChunk},
        config::{ProcessingError, ProcessingResult, ProcessorConfig},
        parser::TextParser,
    },
    domain::{
        prefix_sum::{ChunkStartState, PrefixSumComputer},
        reduce::BoundaryReducer,
        types::{Boundary, DeltaEntry, DeltaVec, PartialState},
    },
    LanguageRules,
};

use super::execution_mode::ExecutionMode;

/// Result of delta-stack processing with metadata
pub struct DeltaStackResult {
    pub boundaries: Vec<usize>,
    pub chunk_count: usize,
    pub thread_count: usize,
}

/// Core implementation of the Δ-Stack Monoid algorithm
///
/// This struct encapsulates the three-phase sentence boundary detection algorithm:
/// 1. Scan phase: Process chunks in parallel to compute partial states
/// 2. Prefix phase: Compute prefix sums to determine chunk start states
/// 3. Reduce phase: Combine partial results with start states to find boundaries
pub struct DeltaStackProcessor {
    language_rules: Arc<dyn LanguageRules>,
    chunk_manager: ChunkManager,
    parser: TextParser,
}

impl DeltaStackProcessor {
    /// Creates a new DeltaStackProcessor with the given configuration
    pub fn new(config: ProcessorConfig, language_rules: Arc<dyn LanguageRules>) -> Self {
        let chunk_manager = ChunkManager::new(config.chunk_size, config.overlap_size);

        Self {
            language_rules,
            chunk_manager,
            parser: TextParser::new(),
        }
    }

    /// Main processing method that executes the Δ-Stack Monoid algorithm
    pub fn process(&self, text: &str, mode: ExecutionMode) -> ProcessingResult<DeltaStackResult> {
        // Early return for empty text
        if text.is_empty() {
            return Ok(DeltaStackResult {
                boundaries: Vec::new(),
                chunk_count: 0,
                thread_count: 1,
            });
        }

        // Phase 0: Chunk the text
        let chunks = self.chunk_manager.chunk_text(text)?;
        if chunks.is_empty() {
            return Ok(DeltaStackResult {
                boundaries: Vec::new(),
                chunk_count: 0,
                thread_count: 1,
            });
        }

        let chunk_count = chunks.len();

        // Determine execution strategy
        let thread_count = mode.determine_thread_count(text.len());

        // Execute the three phases
        let partial_states = self.scan_phase(&chunks, thread_count)?;
        let chunk_starts = self.prefix_phase(&partial_states, &chunks)?;
        let boundaries =
            self.reduce_phase(&partial_states, &chunk_starts, &chunks, thread_count)?;

        Ok(DeltaStackResult {
            boundaries,
            chunk_count,
            thread_count,
        })
    }

    /// Phase 1: Scan - Process chunks to compute partial states
    fn scan_phase(
        &self,
        chunks: &[TextChunk],
        thread_count: usize,
    ) -> ProcessingResult<Vec<PartialState>> {
        if thread_count > 1 {
            // Parallel processing with custom thread pool
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .map_err(|e| ProcessingError::InvalidConfig {
                    reason: format!("Failed to create thread pool: {}", e),
                })?;

            pool.install(|| {
                Ok(chunks
                    .par_iter()
                    .map(|chunk| {
                        self.parser
                            .scan_chunk(&chunk.content, self.language_rules.as_ref())
                    })
                    .collect::<Vec<_>>())
            })
        } else {
            // Sequential processing
            Ok(chunks
                .iter()
                .map(|chunk| {
                    self.parser
                        .scan_chunk(&chunk.content, self.language_rules.as_ref())
                })
                .collect())
        }
    }

    /// Phase 2: Prefix - Compute chunk start states using prefix sums
    fn prefix_phase(
        &self,
        partial_states: &[PartialState],
        chunks: &[TextChunk],
    ) -> ProcessingResult<Vec<ChunkStartState>> {
        if partial_states.len() > 1 {
            Ok(PrefixSumComputer::compute_prefix_sum_with_overlap(
                partial_states,
                chunks,
            ))
        } else {
            // Single chunk - create default start state
            Ok(vec![ChunkStartState {
                cumulative_deltas: DeltaVec::from_vec(vec![
                    DeltaEntry { net: 0, min: 0 };
                    partial_states[0].deltas.len()
                ]),
                global_offset: 0,
            }])
        }
    }

    /// Phase 3: Reduce - Combine partial results with start states to find boundaries
    fn reduce_phase(
        &self,
        partial_states: &[PartialState],
        chunk_starts: &[ChunkStartState],
        chunks: &[TextChunk],
        thread_count: usize,
    ) -> ProcessingResult<Vec<usize>> {
        // Create pairs of (state, start) for processing
        let state_start_pairs: Vec<_> = partial_states
            .iter()
            .zip(chunk_starts.iter())
            .zip(chunks.iter())
            .collect();

        // Process chunks to find boundaries
        let chunk_boundaries: Vec<Vec<Boundary>> = if thread_count > 1 {
            // Parallel reduction with custom thread pool
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .map_err(|e| ProcessingError::InvalidConfig {
                    reason: format!("Failed to create thread pool: {}", e),
                })?;

            pool.install(|| {
                state_start_pairs
                    .par_iter()
                    .map(|((state, start), chunk)| self.reduce_chunk(state, start, chunk))
                    .collect()
            })
        } else {
            // Sequential reduction
            state_start_pairs
                .iter()
                .map(|((state, start), chunk)| self.reduce_chunk(state, start, chunk))
                .collect()
        };

        // Merge boundaries from all chunks
        Ok(self.merge_boundaries(chunk_boundaries))
    }

    /// Reduces a single chunk to find its boundaries
    fn reduce_chunk(
        &self,
        state: &PartialState,
        start: &ChunkStartState,
        chunk: &TextChunk,
    ) -> Vec<Boundary> {
        // For single chunk, use reduce_single
        // For multiple chunks, we need proper indexing
        if chunk.start_offset == 0 && !chunk.has_suffix_overlap {
            // This is a single chunk or the only chunk
            BoundaryReducer::reduce_single(state)
        } else {
            // Multi-chunk - need to handle properly with correct offsets
            let states = vec![state.clone()];
            let starts = vec![start.clone()];
            BoundaryReducer::reduce_all(&states, &starts)
                .into_iter()
                .filter(|b| b.offset >= chunk.start_offset && b.offset < chunk.end_offset)
                .collect()
        }
    }

    /// Merges boundaries from multiple chunks into a single sorted list
    fn merge_boundaries(&self, chunk_boundaries: Vec<Vec<Boundary>>) -> Vec<usize> {
        let mut all_boundaries = Vec::new();

        for boundaries in chunk_boundaries {
            for boundary in boundaries {
                all_boundaries.push(boundary.offset);
            }
        }

        // Sort and deduplicate
        all_boundaries.sort_unstable();
        all_boundaries.dedup();

        all_boundaries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::EnglishLanguageRules;

    fn create_test_processor() -> DeltaStackProcessor {
        let config = ProcessorConfig::default();
        let rules = Arc::new(EnglishLanguageRules::new());
        DeltaStackProcessor::new(config, rules)
    }

    #[test]
    fn test_empty_text() {
        let processor = create_test_processor();
        let result = processor.process("", ExecutionMode::Sequential).unwrap();
        assert!(result.boundaries.is_empty());
        assert_eq!(result.chunk_count, 0);
        assert_eq!(result.thread_count, 1);
    }

    #[test]
    fn test_single_sentence() {
        let processor = create_test_processor();
        let text = "This is a sentence.";
        let result = processor.process(text, ExecutionMode::Sequential).unwrap();
        assert_eq!(result.boundaries.len(), 1);
        assert_eq!(result.boundaries[0], 19); // Position after the period
        assert_eq!(result.chunk_count, 1);
        assert_eq!(result.thread_count, 1);
    }

    #[test]
    fn test_parallel_vs_sequential() {
        let processor = create_test_processor();
        let text = "First sentence. Second sentence. Third sentence.";

        let seq_result = processor.process(text, ExecutionMode::Sequential).unwrap();
        let par_result = processor
            .process(text, ExecutionMode::Parallel { threads: Some(2) })
            .unwrap();

        assert_eq!(seq_result.boundaries, par_result.boundaries);
        assert_eq!(seq_result.chunk_count, par_result.chunk_count);
        assert_eq!(seq_result.thread_count, 1);
        assert_eq!(par_result.thread_count, 2);
    }
}
