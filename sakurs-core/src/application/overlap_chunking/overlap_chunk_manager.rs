//! Overlap-based chunk manager with cross-chunk pattern detection

use super::{
    constants::{DEFAULT_CHUNK_SIZE, DEFAULT_OVERLAP_SIZE},
    overlap_processor::OverlapProcessor,
    pattern_detector::PatternDetector,
    state_tracker::{CrossChunkStateTracker, StateTrackerConfig},
    types::{ProcessedChunk, SuppressionMarker},
};
use crate::{
    application::{
        chunking::{ChunkManager, TextChunk},
        config::ProcessingResult,
    },
    domain::enclosure_suppressor::EnclosureSuppressor,
};
use std::sync::Arc;

/// Configuration for overlap-based chunking
#[derive(Debug, Clone)]
pub struct OverlapChunkConfig {
    /// Base chunk size in bytes
    pub chunk_size: usize,

    /// Overlap size in bytes (default: 32)
    pub overlap_size: usize,

    /// Whether to enable cross-chunk detection
    pub enable_cross_chunk: bool,

    /// State tracker configuration
    pub state_tracker_config: StateTrackerConfig,
}

impl Default for OverlapChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            overlap_size: DEFAULT_OVERLAP_SIZE,
            enable_cross_chunk: true,
            state_tracker_config: StateTrackerConfig::default(),
        }
    }
}

impl OverlapChunkConfig {
    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.chunk_size == 0 {
            return Err("Chunk size must be greater than 0".to_string());
        }

        if self.overlap_size > self.chunk_size / 2 {
            return Err(format!(
                "Overlap size ({}) must not exceed half of chunk size ({})",
                self.overlap_size,
                self.chunk_size / 2
            ));
        }

        if self.overlap_size > 1024 {
            return Err(format!(
                "Overlap size ({}) is too large; maximum recommended is 1024 bytes",
                self.overlap_size
            ));
        }

        Ok(())
    }
}

/// Overlap-based chunk manager that handles cross-chunk patterns
pub struct OverlapChunkManager {
    /// Base chunk manager
    base_manager: ChunkManager,

    /// Overlap processor for pattern detection
    overlap_processor: OverlapProcessor,

    /// State tracker for cross-chunk handling
    state_tracker: CrossChunkStateTracker,

    /// Configuration
    config: OverlapChunkConfig,
}

impl OverlapChunkManager {
    /// Creates a new overlap-based chunk manager
    pub fn new(
        config: OverlapChunkConfig,
        enclosure_suppressor: Arc<dyn EnclosureSuppressor>,
    ) -> Self {
        // Validate configuration
        config
            .validate()
            .expect("Invalid overlap chunk configuration");

        let base_manager = ChunkManager::new(config.chunk_size, config.overlap_size);
        let overlap_processor = OverlapProcessor::new(config.overlap_size, enclosure_suppressor);
        let state_tracker = CrossChunkStateTracker::new(config.state_tracker_config.clone());

        Self {
            base_manager,
            overlap_processor,
            state_tracker,
            config,
        }
    }

    /// Creates an overlap chunk manager with default configuration
    pub fn with_defaults(enclosure_suppressor: Arc<dyn EnclosureSuppressor>) -> Self {
        Self::new(OverlapChunkConfig::default(), enclosure_suppressor)
    }

    /// Chunks text with cross-chunk pattern processing
    pub fn chunk_with_overlap_processing(
        &mut self,
        text: &str,
    ) -> ProcessingResult<Vec<ProcessedChunk>> {
        // If cross-chunk processing is disabled, just wrap base chunks
        if !self.config.enable_cross_chunk {
            return self.chunk_without_overlap_processing(text);
        }

        // Create base chunks
        let base_chunks = self.base_manager.chunk_text(text)?;

        // If we have only one chunk, no overlap processing needed
        if base_chunks.len() <= 1 {
            let mut processed_chunks: Vec<ProcessedChunk> = base_chunks
                .into_iter()
                .map(ProcessedChunk::from_base)
                .collect();

            // Still detect suppressions within the single chunk
            if let Some(chunk) = processed_chunks.get_mut(0) {
                if let Ok(suppressions) = self.detect_suppressions_in_chunk(&chunk.base_chunk) {
                    for suppression in suppressions {
                        chunk.add_suppression(suppression);
                    }
                }
            }

            return Ok(processed_chunks);
        }

        // Process chunks with overlap detection
        let mut processed_chunks = Vec::with_capacity(base_chunks.len());

        for (idx, chunk) in base_chunks.iter().enumerate() {
            let mut processed = ProcessedChunk::from_base(chunk.clone());

            // First, process the chunk itself for suppressions
            let chunk_suppressions = self.detect_suppressions_in_chunk(chunk)?;
            for suppression in chunk_suppressions {
                processed.add_suppression(suppression);
            }

            // Process overlap with next chunk
            if idx < base_chunks.len() - 1 {
                let next_chunk = &base_chunks[idx + 1];
                let overlap_result = self.overlap_processor.process_overlap(chunk, next_chunk)?;

                // Add suppressions from overlap
                for suppression in overlap_result.suppressions {
                    // Determine which chunk owns this suppression based on position
                    if suppression.position >= chunk.start_offset
                        && suppression.position < chunk.end_offset
                    {
                        // Adjust position to be relative to chunk start
                        let mut adjusted_suppression = suppression.clone();
                        adjusted_suppression.position = suppression.position - chunk.start_offset;
                        processed.add_suppression(adjusted_suppression);
                    } else if suppression.position >= next_chunk.start_offset {
                        // This belongs to the next chunk, will be added in next iteration
                        continue;
                    }
                }

                // Track overlap boundaries
                for adjustment in &overlap_result.boundary_adjustments {
                    if let Some(pos) = adjustment.adjusted_position {
                        processed.overlap_boundaries.push(pos);
                    }
                }
            }

            processed_chunks.push(processed);
        }

        // Apply state tracking
        self.state_tracker.track_transitions(&processed_chunks)?;

        // Update transition states in chunks
        for (idx, state) in self.state_tracker.get_states().iter().enumerate() {
            if idx < processed_chunks.len() {
                processed_chunks[idx].transition_state = Some(state.clone());
            }
        }

        Ok(processed_chunks)
    }

    /// Chunks without overlap processing (fallback mode)
    fn chunk_without_overlap_processing(
        &self,
        text: &str,
    ) -> ProcessingResult<Vec<ProcessedChunk>> {
        let base_chunks = self.base_manager.chunk_text(text)?;
        Ok(base_chunks
            .into_iter()
            .map(ProcessedChunk::from_base)
            .collect())
    }

    /// Gets deduplicated boundaries from all processed chunks
    pub fn get_deduplicated_boundaries(&self) -> Vec<usize> {
        self.state_tracker.get_deduplicated_boundaries()
    }

    /// Resets the state tracker
    pub fn reset_state(&mut self) {
        self.state_tracker = CrossChunkStateTracker::new(self.config.state_tracker_config.clone());
    }

    /// Updates configuration
    pub fn update_config(&mut self, config: OverlapChunkConfig) {
        self.base_manager = ChunkManager::new(config.chunk_size, config.overlap_size);
        self.state_tracker = CrossChunkStateTracker::new(config.state_tracker_config.clone());
        self.config = config;
    }

    /// Gets the current configuration
    pub fn config(&self) -> &OverlapChunkConfig {
        &self.config
    }

    /// Checks if a position is suppressed based on overlap processing
    pub fn is_position_suppressed(&self, position: usize) -> bool {
        self.state_tracker.is_position_suppressed(position)
    }

    /// Detects suppressions within a single chunk
    fn detect_suppressions_in_chunk(
        &self,
        chunk: &TextChunk,
    ) -> ProcessingResult<Vec<SuppressionMarker>> {
        let mut suppressions = Vec::new();
        let content = &chunk.content;

        for (idx, ch) in content.char_indices() {
            // Skip non-enclosure characters
            if !PatternDetector::is_potential_enclosure(ch) {
                continue;
            }

            // Create context
            let context = PatternDetector::create_enclosure_context(content, idx, ch);

            // Check if should be suppressed
            if self
                .overlap_processor
                .enclosure_suppressor
                .should_suppress_enclosure(ch, &context)
            {
                let reason = PatternDetector::determine_suppression_reason(ch, &context);
                let confidence = PatternDetector::calculate_confidence(&context, &reason);

                suppressions.push(SuppressionMarker {
                    position: idx,
                    character: ch,
                    reason,
                    confidence,
                    from_overlap: false,
                });
            }
        }

        Ok(suppressions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enclosure_suppressor::EnglishEnclosureSuppressor;

    #[test]
    fn test_basic_chunking() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let mut manager = OverlapChunkManager::with_defaults(suppressor);

        let text = "This is a simple test.";
        let chunks = manager.chunk_with_overlap_processing(text).unwrap();

        // Should have one chunk for small text
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].base_chunk.content, text);
    }

    #[test]
    fn test_cross_chunk_contraction() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 20, // Small chunks to force splitting
            overlap_size: 10,
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        // This should split around "isn't"
        let text = "The problem here isn't the solution.";
        let chunks = manager.chunk_with_overlap_processing(text).unwrap();

        // Should have multiple chunks
        assert!(chunks.len() > 1);

        // Check for suppressions
        let has_suppressions = chunks.iter().any(|c| !c.suppression_markers.is_empty());
        assert!(
            has_suppressions,
            "Should detect suppression for contraction"
        );
    }

    #[test]
    fn test_config_update() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let mut manager = OverlapChunkManager::with_defaults(suppressor);

        let new_config = OverlapChunkConfig {
            chunk_size: 1024,
            overlap_size: 64,
            ..Default::default()
        };

        manager.update_config(new_config.clone());
        assert_eq!(manager.config().chunk_size, 1024);
        assert_eq!(manager.config().overlap_size, 64);
    }
}
