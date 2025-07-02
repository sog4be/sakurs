//! Main text processor orchestrator
//!
//! This module provides the core TextProcessor that coordinates all aspects
//! of text processing including chunking, parallel execution, and result merging.

use crate::application::{
    chunking::{ChunkManager, TextChunk},
    config::{ProcessingError, ProcessingMetrics, ProcessingResult, ProcessorConfig},
};
use crate::domain::{
    language::LanguageRules,
    parser::Parser,
    state::{Boundary, PartialState},
    Monoid,
};
use std::sync::Arc;
use std::time::Instant;

/// Result of text processing containing boundaries and metrics
#[derive(Debug, Clone)]
pub struct ProcessingOutput {
    /// Detected sentence boundaries
    pub boundaries: Vec<Boundary>,

    /// Total text length processed
    pub text_length: usize,

    /// Performance metrics
    pub metrics: ProcessingMetrics,
}

impl ProcessingOutput {
    /// Converts boundaries to sentence ranges
    pub fn sentence_ranges(&self) -> Vec<std::ops::Range<usize>> {
        let mut ranges = Vec::new();
        let mut start = 0;

        for boundary in &self.boundaries {
            if boundary.offset > start {
                ranges.push(start..boundary.offset);
                start = boundary.offset;
            }
        }

        // Add final range if text doesn't end with boundary
        if start < self.text_length {
            ranges.push(start..self.text_length);
        }

        ranges
    }

    /// Extracts sentences from the original text
    pub fn extract_sentences<'a>(&self, text: &'a str) -> Vec<&'a str> {
        self.sentence_ranges()
            .into_iter()
            .filter_map(|range| text.get(range))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Main text processor that orchestrates the processing pipeline
pub struct TextProcessor {
    /// Configuration parameters
    config: ProcessorConfig,

    /// Language rules for sentence detection
    language_rules: Arc<dyn LanguageRules>,

    /// Parser instance
    parser: Parser,

    /// Chunk manager
    chunk_manager: ChunkManager,
}

impl TextProcessor {
    /// Creates a new text processor with default configuration
    pub fn new(language_rules: Arc<dyn LanguageRules>) -> Self {
        let config = ProcessorConfig::default();
        Self::with_config(config, language_rules)
    }

    /// Creates a new text processor with custom configuration
    pub fn with_config(config: ProcessorConfig, language_rules: Arc<dyn LanguageRules>) -> Self {
        // Validate configuration
        config.validate().expect("Invalid processor configuration");

        let chunk_manager = ChunkManager::new(config.chunk_size, config.overlap_size);

        Self {
            config,
            language_rules,
            parser: Parser::new(),
            chunk_manager,
        }
    }

    /// Processes text and returns detected sentence boundaries
    pub fn process_text(&self, text: &str) -> ProcessingResult<ProcessingOutput> {
        let start_time = Instant::now();
        let mut metrics = ProcessingMetrics::default();

        // Check text size limits
        if text.len() > self.config.max_text_size {
            return Err(ProcessingError::TextTooLarge {
                size: text.len(),
                max: self.config.max_text_size,
            });
        }

        // Empty text handling
        if text.is_empty() {
            return Ok(ProcessingOutput {
                boundaries: Vec::new(),
                text_length: 0,
                metrics,
            });
        }

        metrics.bytes_processed = text.len();

        // Decide between sequential and parallel processing
        let result = if text.len() < self.config.parallel_threshold {
            metrics.thread_count = 1;
            self.process_sequential(text, &mut metrics)?
        } else {
            #[cfg(feature = "parallel")]
            {
                self.process_parallel(text, &mut metrics)?
            }
            #[cfg(not(feature = "parallel"))]
            {
                metrics.thread_count = 1;
                self.process_sequential(text, &mut metrics)?
            }
        };

        metrics.total_time_us = start_time.elapsed().as_micros() as u64;

        Ok(ProcessingOutput {
            boundaries: result.boundaries.into_iter().collect(),
            text_length: text.len(),
            metrics,
        })
    }

    /// Sequential processing for small texts
    fn process_sequential(
        &self,
        text: &str,
        metrics: &mut ProcessingMetrics,
    ) -> ProcessingResult<PartialState> {
        let chunk_start = Instant::now();

        // For sequential processing, we might still chunk for memory efficiency
        let chunks = self.chunk_manager.chunk_text(text)?;
        metrics.chunk_count = chunks.len();
        metrics.chunking_time_us = chunk_start.elapsed().as_micros() as u64;

        let _process_start = Instant::now();

        if chunks.len() == 1 {
            // Single chunk - process directly
            let state =
                self.parser
                    .parse_chunk(&chunks[0].content, self.language_rules.as_ref(), None);
            metrics.boundaries_found = state.boundaries.len();
            Ok(state)
        } else {
            // Multiple chunks - process sequentially and merge
            let states = self.process_chunks_sequential(&chunks)?;

            let merge_start = Instant::now();
            let merged = self.merge_states(states);
            metrics.merge_time_us = merge_start.elapsed().as_micros() as u64;

            metrics.boundaries_found = merged.boundaries.len();
            Ok(merged)
        }
    }

    /// Process chunks sequentially
    fn process_chunks_sequential(
        &self,
        chunks: &[TextChunk],
    ) -> ProcessingResult<Vec<PartialState>> {
        let mut states = Vec::with_capacity(chunks.len());
        let mut carry_state: Option<PartialState> = None;

        for chunk in chunks {
            let state = self.parser.parse_chunk(
                &chunk.content,
                self.language_rules.as_ref(),
                carry_state.as_ref(),
            );

            // Update carry state for next chunk
            carry_state = Some(state.clone());
            states.push(state);
        }

        Ok(states)
    }

    /// Parallel processing for large texts
    #[cfg(feature = "parallel")]
    fn process_parallel(
        &self,
        text: &str,
        metrics: &mut ProcessingMetrics,
    ) -> ProcessingResult<PartialState> {
        use rayon::prelude::*;

        let chunk_start = Instant::now();
        let chunks = self.chunk_manager.chunk_text(text)?;
        metrics.chunk_count = chunks.len();
        metrics.chunking_time_us = chunk_start.elapsed().as_micros() as u64;

        let parallel_start = Instant::now();

        // Configure thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.config.max_threads.unwrap_or_else(num_cpus::get))
            .thread_name(|i| format!("sakurs-worker-{i}"))
            .build()
            .map_err(|e| ProcessingError::ParallelError {
                source: Box::new(e),
            })?;

        metrics.thread_count = pool.current_num_threads();

        // Process chunks in parallel
        let states = pool.install(|| {
            chunks
                .par_iter()
                .map(|chunk| {
                    self.parser.parse_chunk(
                        &chunk.content,
                        self.language_rules.as_ref(),
                        None, // No carry state in parallel mode
                    )
                })
                .collect::<Vec<_>>()
        });

        metrics.parallel_time_us = parallel_start.elapsed().as_micros() as u64;

        // Merge results
        let merge_start = Instant::now();
        let merged = self.merge_states_with_overlap_resolution(states, &chunks);
        metrics.merge_time_us = merge_start.elapsed().as_micros() as u64;

        metrics.boundaries_found = merged.boundaries.len();
        Ok(merged)
    }

    /// Merges multiple partial states using monoid operations
    fn merge_states(&self, states: Vec<PartialState>) -> PartialState {
        if states.is_empty() {
            return PartialState::identity();
        }

        states.into_iter().reduce(|a, b| a.combine(&b)).unwrap()
    }

    /// Merges states with overlap resolution for parallel processing
    #[cfg(feature = "parallel")]
    fn merge_states_with_overlap_resolution(
        &self,
        states: Vec<PartialState>,
        chunks: &[TextChunk],
    ) -> PartialState {
        if states.is_empty() {
            return PartialState::identity();
        }

        // First, adjust boundary offsets based on chunk positions
        let adjusted_states: Vec<PartialState> = states
            .into_iter()
            .zip(chunks)
            .map(|(state, chunk)| {
                // Adjust boundary offsets to global positions
                let adjusted_boundaries = state
                    .boundaries
                    .into_iter()
                    .filter_map(|mut boundary| {
                        // Only keep boundaries within effective range
                        let effective_range = chunk.effective_range();
                        let global_offset = chunk.start_offset + boundary.offset;

                        if global_offset >= effective_range.start
                            && global_offset < effective_range.end
                        {
                            boundary.offset = global_offset;
                            Some(boundary)
                        } else {
                            None
                        }
                    })
                    .collect();

                PartialState {
                    boundaries: adjusted_boundaries,
                    deltas: state.deltas,
                    abbreviation: state.abbreviation,
                    chunk_length: chunk.effective_range().len(),
                }
            })
            .collect();

        // Merge using monoid operations
        self.merge_states(adjusted_states)
    }

    /// Processes a stream of text chunks
    pub fn process_streaming<I>(&self, chunks: I) -> ProcessingResult<ProcessingOutput>
    where
        I: Iterator<Item = String>,
    {
        let start_time = Instant::now();
        let mut metrics = ProcessingMetrics::default();
        let mut accumulated_state = PartialState::identity();
        let mut total_length = 0;

        for chunk_text in chunks {
            if chunk_text.is_empty() {
                continue;
            }

            metrics.chunk_count += 1;
            metrics.bytes_processed += chunk_text.len();
            total_length += chunk_text.len();

            let state = self.parser.parse_chunk(
                &chunk_text,
                self.language_rules.as_ref(),
                Some(&accumulated_state),
            );

            accumulated_state = accumulated_state.combine(&state);
        }

        metrics.boundaries_found = accumulated_state.boundaries.len();
        metrics.total_time_us = start_time.elapsed().as_micros() as u64;
        metrics.thread_count = 1; // Streaming is sequential

        Ok(ProcessingOutput {
            boundaries: accumulated_state.boundaries.into_iter().collect(),
            text_length: total_length,
            metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::MockLanguageRules;

    #[test]
    fn test_empty_text_processing() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = TextProcessor::new(rules);

        let result = processor.process_text("").unwrap();
        assert!(result.boundaries.is_empty());
        assert_eq!(result.text_length, 0);
    }

    #[test]
    fn test_simple_text_processing() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = TextProcessor::new(rules);

        let text = "Hello world. This is a test.";
        let result = processor.process_text(text).unwrap();

        assert!(!result.boundaries.is_empty());
        assert_eq!(result.text_length, text.len());

        // Extract sentences
        let sentences = result.extract_sentences(text);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "This is a test.");
    }

    #[test]
    fn test_large_text_threshold() {
        let mut config = ProcessorConfig::default();
        config.parallel_threshold = 100; // Very low threshold

        let rules = Arc::new(MockLanguageRules::english());
        let processor = TextProcessor::with_config(config, rules);

        let text = "This is a longer text that should trigger parallel processing if available. \
                    It contains multiple sentences. Each sentence ends with a period. \
                    The processor should handle this correctly.";

        let result = processor.process_text(text).unwrap();
        assert!(!result.boundaries.is_empty());

        #[cfg(feature = "parallel")]
        assert!(result.metrics.thread_count > 1);
    }

    #[test]
    fn test_text_size_limit() {
        let mut config = ProcessorConfig::default();
        config.max_text_size = 100;

        let rules = Arc::new(MockLanguageRules::english());
        let processor = TextProcessor::with_config(config, rules);

        let text = "a".repeat(200);
        let result = processor.process_text(&text);

        assert!(matches!(result, Err(ProcessingError::TextTooLarge { .. })));
    }

    #[test]
    fn test_streaming_processing() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = TextProcessor::new(rules);

        let chunks = vec![
            "First chunk with a sentence.".to_string(),
            " Second chunk continues.".to_string(),
            " Final chunk ends here.".to_string(),
        ];

        let result = processor.process_streaming(chunks.into_iter()).unwrap();
        assert!(!result.boundaries.is_empty());
        assert_eq!(result.metrics.chunk_count, 3);
    }

    #[test]
    fn test_metrics_calculation() {
        let rules = Arc::new(MockLanguageRules::english());
        let processor = TextProcessor::new(rules);

        let text = "Test sentence. Another one.";
        let result = processor.process_text(text).unwrap();

        assert_eq!(result.metrics.bytes_processed, text.len());
        assert!(result.metrics.total_time_us > 0);
        assert!(result.metrics.throughput_mbps() > 0.0);
    }
}
