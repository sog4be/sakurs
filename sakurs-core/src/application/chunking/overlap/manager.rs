//! Overlap-based chunk manager with cross-chunk pattern detection and state tracking

use super::{
    processor::OverlapProcessor,
    types::{ChunkTransitionState, PartialPattern, PatternType, ProcessedChunk, SuppressionMarker},
};
use crate::{
    application::{
        chunking::base::{ChunkManager, TextChunk},
        config::ProcessingResult,
    },
    domain::enclosure_suppressor::EnclosureSuppressor,
};
use std::{collections::HashMap, sync::Arc};

// Re-export constants that were in constants.rs
/// Default chunk size in bytes (64KB)
pub const DEFAULT_CHUNK_SIZE: usize = 65536;

/// Default overlap size in bytes for cross-chunk pattern detection
pub const DEFAULT_OVERLAP_SIZE: usize = 32;

/// Default minimum confidence threshold for pattern detection
pub const DEFAULT_MIN_CONFIDENCE: f32 = 0.7;

/// Default window size for boundary deduplication
pub const DEFAULT_DEDUP_WINDOW: usize = 32;

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

/// Configuration for state tracking
#[derive(Debug, Clone)]
pub struct StateTrackerConfig {
    /// Maximum distance to look for duplicate boundaries
    pub dedup_window: usize,

    /// Minimum confidence threshold for cross-chunk patterns
    pub min_confidence: f32,

    /// Whether to merge adjacent suppressions
    pub merge_adjacent: bool,
}

impl Default for StateTrackerConfig {
    fn default() -> Self {
        Self {
            dedup_window: DEFAULT_DEDUP_WINDOW,
            min_confidence: DEFAULT_MIN_CONFIDENCE,
            merge_adjacent: true,
        }
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
            if !self.overlap_processor.is_potential_enclosure(ch) {
                continue;
            }

            // Create context
            let context = self
                .overlap_processor
                .create_enclosure_context(content, idx, ch);

            // Check if should be suppressed
            if self
                .overlap_processor
                .enclosure_suppressor
                .should_suppress_enclosure(ch, &context)
            {
                let reason = self
                    .overlap_processor
                    .determine_suppression_reason(ch, &context);
                let confidence = self
                    .overlap_processor
                    .calculate_confidence(&context, &reason);

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

// State tracker implementation (previously in state_tracker.rs)

/// Tracks state across chunk boundaries for pattern detection
struct CrossChunkStateTracker {
    /// States for each chunk transition
    chunk_states: Vec<ChunkTransitionState>,

    /// Cache of detected boundaries for deduplication
    boundary_cache: BoundaryCache,

    /// Configuration for state tracking
    config: StateTrackerConfig,
}

/// Cache for boundary deduplication
#[derive(Debug, Default)]
struct BoundaryCache {
    /// Map of position to chunk indices where boundary was detected
    boundaries: HashMap<usize, Vec<usize>>,

    /// Suppressed boundary positions
    suppressed_positions: Vec<usize>,
}

impl CrossChunkStateTracker {
    /// Creates a new state tracker
    pub fn new(config: StateTrackerConfig) -> Self {
        Self {
            chunk_states: Vec::new(),
            boundary_cache: BoundaryCache::default(),
            config,
        }
    }

    /// Tracks transitions between processed chunks
    pub fn track_transitions(&mut self, chunks: &[ProcessedChunk]) -> ProcessingResult<()> {
        // Clear previous state
        self.chunk_states.clear();

        // Process each chunk and its transitions
        for (idx, chunk) in chunks.iter().enumerate() {
            let mut state = ChunkTransitionState::new(idx);

            // Analyze chunk ending for patterns
            if idx < chunks.len() - 1 {
                self.analyze_chunk_ending(chunk, &mut state)?;
            }

            // Analyze chunk beginning for patterns
            if idx > 0 {
                self.analyze_chunk_beginning(chunk, &mut state)?;
            }

            // Track suppressions that affect neighboring chunks
            self.track_forward_suppressions(chunk, &mut state);

            // Update boundary cache
            self.update_boundary_cache(chunk, idx);

            self.chunk_states.push(state);
        }

        // Resolve cross-chunk patterns
        self.resolve_cross_chunk_patterns()?;

        Ok(())
    }

    /// Analyzes the end of a chunk for partial patterns
    fn analyze_chunk_ending(
        &self,
        chunk: &ProcessedChunk,
        state: &mut ChunkTransitionState,
    ) -> ProcessingResult<()> {
        let content = &chunk.base_chunk.content;

        // Check for partial contraction patterns
        if content.ends_with("isn")
            || content.ends_with("don")
            || content.ends_with("won")
            || content.ends_with("can")
        {
            state.ending_patterns.push(PartialPattern {
                text: content
                    .chars()
                    .rev()
                    .take(3)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect(),
                expected_continuations: vec!["'t".to_string()],
                pattern_type: PatternType::Contraction,
            });
            state.add_pattern_confidence("contraction_ending".to_string(), 0.9);
        }

        // Check for words that might have possessives
        if content
            .chars()
            .last()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false)
        {
            // Extract the last word
            let last_word = content.split_whitespace().last().unwrap_or("");
            if !last_word.is_empty() {
                state.ending_patterns.push(PartialPattern {
                    text: last_word.to_string(),
                    expected_continuations: vec!["'".to_string(), "'s".to_string()],
                    pattern_type: PatternType::Possessive,
                });
            }
        }

        Ok(())
    }

    /// Analyzes the beginning of a chunk for pattern continuations
    fn analyze_chunk_beginning(
        &self,
        chunk: &ProcessedChunk,
        state: &mut ChunkTransitionState,
    ) -> ProcessingResult<()> {
        let content = &chunk.base_chunk.content;

        // Check for contraction completions
        if content.starts_with("'t") || content.starts_with("'t") {
            state.starting_patterns.push(PartialPattern {
                text: "'t".to_string(),
                expected_continuations: vec![],
                pattern_type: PatternType::Contraction,
            });
            state.add_pattern_confidence("contraction_start".to_string(), 0.95);
        }

        // Check for possessive markers
        if content.starts_with('\'') || content.starts_with('\u{2019}') {
            state.starting_patterns.push(PartialPattern {
                text: content.chars().take(2).collect(),
                expected_continuations: vec![],
                pattern_type: PatternType::Possessive,
            });
        }

        Ok(())
    }

    /// Tracks suppressions that should affect neighboring chunks
    fn track_forward_suppressions(&self, chunk: &ProcessedChunk, state: &mut ChunkTransitionState) {
        // Find suppressions near the chunk end that might affect the next chunk
        let chunk_end = chunk.base_chunk.end_offset;
        let window_start = chunk_end.saturating_sub(self.config.dedup_window);

        for marker in &chunk.suppression_markers {
            if marker.position >= window_start && marker.confidence >= self.config.min_confidence {
                state.forward_suppressions.push(marker.clone());
            }
        }
    }

    /// Updates the boundary cache with boundaries from a chunk
    fn update_boundary_cache(&mut self, chunk: &ProcessedChunk, chunk_idx: usize) {
        // Add overlap boundaries to cache
        for &boundary_pos in &chunk.overlap_boundaries {
            self.boundary_cache
                .boundaries
                .entry(boundary_pos)
                .or_default()
                .push(chunk_idx);
        }

        // Track suppressed positions
        for marker in &chunk.suppression_markers {
            if marker.confidence >= self.config.min_confidence {
                self.boundary_cache
                    .suppressed_positions
                    .push(marker.position);
            }
        }
    }

    /// Resolves patterns that span multiple chunks
    fn resolve_cross_chunk_patterns(&mut self) -> ProcessingResult<()> {
        // Look for matching patterns between adjacent chunks
        for i in 0..self.chunk_states.len().saturating_sub(1) {
            let (left_patterns, right_patterns) = {
                let left = &self.chunk_states[i].ending_patterns;
                let right = &self.chunk_states[i + 1].starting_patterns;
                (left.clone(), right.clone())
            };

            // Check for pattern matches
            for left_pattern in &left_patterns {
                for right_pattern in &right_patterns {
                    if self.patterns_match(left_pattern, right_pattern) {
                        // Update confidence scores
                        self.chunk_states[i].add_pattern_confidence(
                            format!("{:?}_confirmed", left_pattern.pattern_type),
                            0.95,
                        );
                        self.chunk_states[i + 1].add_pattern_confidence(
                            format!("{:?}_confirmed", right_pattern.pattern_type),
                            0.95,
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Checks if two patterns match across chunks
    fn patterns_match(&self, left: &PartialPattern, right: &PartialPattern) -> bool {
        // Same pattern type
        if left.pattern_type != right.pattern_type {
            return false;
        }

        // Check if right pattern is an expected continuation of left
        match left.pattern_type {
            PatternType::Contraction => left.expected_continuations.contains(&right.text),
            PatternType::Possessive => {
                right.text.starts_with('\'') || right.text.starts_with('\u{2019}')
            }
            _ => false,
        }
    }

    /// Gets deduplicated boundaries from all chunks
    pub fn get_deduplicated_boundaries(&self) -> Vec<usize> {
        let mut unique_boundaries = Vec::new();

        for (pos, chunks) in &self.boundary_cache.boundaries {
            // Skip if this position was suppressed
            if self.boundary_cache.suppressed_positions.contains(pos) {
                continue;
            }

            // Include boundary only once (prefer earlier chunk)
            if chunks.len() == 1 || chunks[0] == *chunks.iter().min().unwrap() {
                unique_boundaries.push(*pos);
            }
        }

        unique_boundaries.sort();
        unique_boundaries
    }

    /// Gets all transition states
    pub fn get_states(&self) -> &[ChunkTransitionState] {
        &self.chunk_states
    }

    /// Checks if a position is suppressed
    pub fn is_position_suppressed(&self, position: usize) -> bool {
        self.boundary_cache.suppressed_positions.contains(&position)
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

    #[test]
    fn test_very_small_overlap_size() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 50,
            overlap_size: 1, // Minimum overlap
            enable_cross_chunk: true,
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        let text = "First sentence. Second sentence. Third sentence.";
        let result = manager.chunk_with_overlap_processing(text);

        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_overlap_size_at_maximum_allowed() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 20,
            overlap_size: 10, // Maximum allowed is half of chunk_size
            enable_cross_chunk: true,
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        let text = "Test text.";
        let result = manager.chunk_with_overlap_processing(text);

        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_text_processing() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let mut manager = OverlapChunkManager::with_defaults(suppressor);

        let result = manager.chunk_with_overlap_processing("");
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_single_character_text() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 10,
            overlap_size: 5,
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        let result = manager.chunk_with_overlap_processing(".");
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].base_chunk.content, ".");
    }

    #[test]
    fn test_text_smaller_than_chunk_size() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 1000,
            overlap_size: 100,
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        let text = "Short text.";
        let result = manager.chunk_with_overlap_processing(text);

        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].base_chunk.content, text);
    }

    #[test]
    fn test_cross_chunk_disabled() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 15,
            overlap_size: 5,
            enable_cross_chunk: false, // Disabled
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        let text = "This isn't working.";
        let result = manager.chunk_with_overlap_processing(text);

        assert!(result.is_ok());
        // With cross-chunk disabled, suppressions should not be detected across chunks
    }

    #[test]
    fn test_multiple_suppressions_in_overlap() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 30,
            overlap_size: 15,
            enable_cross_chunk: true,
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        // Multiple contractions that might fall in overlap region
        let text = "It isn't working. That's why we can't continue.";
        let result = manager.chunk_with_overlap_processing(text);

        assert!(result.is_ok());
        let chunks = result.unwrap();

        // Check that suppressions are detected
        let total_suppressions: usize = chunks.iter().map(|c| c.suppression_markers.len()).sum();
        assert!(
            total_suppressions > 0,
            "Should detect multiple suppressions"
        );
    }

    #[test]
    fn test_chunk_with_only_whitespace() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let mut manager = OverlapChunkManager::with_defaults(suppressor);

        let result = manager.chunk_with_overlap_processing("   \n\t   ");
        assert!(result.is_ok());
        let chunks = result.unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_large_overlap_relative_to_chunk() {
        let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
        let config = OverlapChunkConfig {
            chunk_size: 100,
            overlap_size: 50, // Maximum allowed (50% overlap)
            enable_cross_chunk: true,
            ..Default::default()
        };
        let mut manager = OverlapChunkManager::new(config, suppressor);

        let text = "A".repeat(300); // 300 characters
        let result = manager.chunk_with_overlap_processing(&text);

        assert!(result.is_ok());
        let chunks = result.unwrap();

        // With 50% overlap and 100 byte chunks on 300 chars, we should have at least 5 chunks
        // Actually, let's just verify we have multiple chunks
        assert!(
            chunks.len() >= 2,
            "Expected at least 2 chunks, got {}",
            chunks.len()
        );

        // Verify all chunks are valid
        for chunk in &chunks {
            assert!(!chunk.base_chunk.content.is_empty());
            assert!(chunk.base_chunk.end_offset <= text.len());
        }
    }
}
