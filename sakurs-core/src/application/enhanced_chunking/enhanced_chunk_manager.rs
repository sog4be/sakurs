//! Enhanced chunk manager with cross-chunk pattern detection

use super::{
    overlap_processor::OverlapProcessor,
    state_tracker::{CrossChunkStateTracker, StateTrackerConfig},
    types::{ProcessedChunk, SuppressionMarker, SuppressionReason},
};
use crate::{
    application::{
        chunking::{ChunkManager, TextChunk},
        config::ProcessingResult,
    },
    domain::enclosure_suppressor::EnclosureSuppressor,
};
use std::sync::Arc;

/// Configuration for enhanced chunking
#[derive(Debug, Clone)]
pub struct EnhancedChunkConfig {
    /// Base chunk size in bytes
    pub chunk_size: usize,

    /// Overlap size in bytes (default: 32)
    pub overlap_size: usize,

    /// Whether to enable cross-chunk detection
    pub enable_cross_chunk: bool,

    /// Size of suppression result cache
    pub suppression_cache_size: usize,

    /// State tracker configuration
    pub state_tracker_config: StateTrackerConfig,
}

impl Default for EnhancedChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 65536, // 64KB
            overlap_size: 32,  // 32 bytes
            enable_cross_chunk: true,
            suppression_cache_size: 1000,
            state_tracker_config: StateTrackerConfig::default(),
        }
    }
}

/// Enhanced chunk manager that handles cross-chunk patterns
pub struct EnhancedChunkManager {
    /// Base chunk manager
    base_manager: ChunkManager,

    /// Overlap processor for pattern detection
    overlap_processor: OverlapProcessor,

    /// State tracker for cross-chunk handling
    state_tracker: CrossChunkStateTracker,

    /// Configuration
    config: EnhancedChunkConfig,
}

impl EnhancedChunkManager {
    /// Creates a new enhanced chunk manager
    pub fn new(
        config: EnhancedChunkConfig,
        enclosure_suppressor: Arc<dyn EnclosureSuppressor>,
    ) -> Self {
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

    /// Creates an enhanced chunk manager with default configuration
    pub fn with_defaults(enclosure_suppressor: Arc<dyn EnclosureSuppressor>) -> Self {
        Self::new(EnhancedChunkConfig::default(), enclosure_suppressor)
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
            return Ok(base_chunks
                .into_iter()
                .map(ProcessedChunk::from_base)
                .collect());
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
    pub fn update_config(&mut self, config: EnhancedChunkConfig) {
        self.base_manager = ChunkManager::new(config.chunk_size, config.overlap_size);
        self.state_tracker = CrossChunkStateTracker::new(config.state_tracker_config.clone());
        self.config = config;
    }

    /// Gets the current configuration
    pub fn config(&self) -> &EnhancedChunkConfig {
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
        use crate::domain::enclosure_suppressor::EnclosureContext;
        use smallvec::SmallVec;

        let mut suppressions = Vec::new();
        let content = &chunk.content;

        for (idx, ch) in content.char_indices() {
            // Skip non-enclosure characters
            if !matches!(
                ch,
                '\'' | '"'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '\u{2018}'
                    | '\u{2019}'
                    | '\u{201C}'
                    | '\u{201D}'
            ) {
                continue;
            }

            // Create context
            let preceding_chars: SmallVec<[char; 3]> = content[..idx]
                .chars()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            let following_chars: SmallVec<[char; 3]> =
                content[idx + ch.len_utf8()..].chars().take(3).collect();

            let line_offset = content[..idx]
                .rfind('\n')
                .map(|pos| idx - pos - 1)
                .unwrap_or(idx);

            let context = EnclosureContext {
                position: idx,
                preceding_chars,
                following_chars,
                line_offset,
                chunk_text: content,
            };

            // Check if should be suppressed
            if self
                .overlap_processor
                .enclosure_suppressor
                .should_suppress_enclosure(ch, &context)
            {
                let reason = match ch {
                    '\'' | '\u{2019}'
                        if context
                            .preceding_chars
                            .last()
                            .map(|c| c.is_numeric())
                            .unwrap_or(false) =>
                    {
                        SuppressionReason::Measurement
                    }
                    '\'' | '\u{2019}'
                        if context
                            .preceding_chars
                            .last()
                            .map(|c| c.is_alphabetic())
                            .unwrap_or(false)
                            && context
                                .following_chars
                                .first()
                                .map(|c| c.is_alphabetic())
                                .unwrap_or(false) =>
                    {
                        SuppressionReason::Contraction
                    }
                    '\'' | '\u{2019}'
                        if context
                            .preceding_chars
                            .last()
                            .map(|c| c.is_alphanumeric())
                            .unwrap_or(false)
                            && context
                                .following_chars
                                .first()
                                .map(|c| c.is_whitespace() || c.is_ascii_punctuation())
                                .unwrap_or(true) =>
                    {
                        SuppressionReason::Possessive
                    }
                    ')' if context.line_offset < 10 => SuppressionReason::ListItem,
                    '"' if context
                        .preceding_chars
                        .last()
                        .map(|c| c.is_numeric())
                        .unwrap_or(false) =>
                    {
                        SuppressionReason::Measurement
                    }
                    _ => SuppressionReason::CrossChunkPattern {
                        pattern: format!("{:?}", context.preceding_chars),
                    },
                };

                suppressions.push(SuppressionMarker {
                    position: idx,
                    character: ch,
                    reason,
                    confidence: 0.9,
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
        let mut manager = EnhancedChunkManager::with_defaults(suppressor);

        let text = "This is a simple test.";
        let chunks = manager.chunk_with_overlap_processing(text).unwrap();

        // Should have one chunk for small text
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].base_chunk.content, text);
    }

    #[test]
    fn test_cross_chunk_contraction() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = EnhancedChunkConfig {
            chunk_size: 20, // Small chunks to force splitting
            overlap_size: 10,
            ..Default::default()
        };
        let mut manager = EnhancedChunkManager::new(config, suppressor);

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
        let mut manager = EnhancedChunkManager::with_defaults(suppressor);

        let new_config = EnhancedChunkConfig {
            chunk_size: 1024,
            overlap_size: 64,
            ..Default::default()
        };

        manager.update_config(new_config.clone());
        assert_eq!(manager.config().chunk_size, 1024);
        assert_eq!(manager.config().overlap_size, 64);
    }
}
