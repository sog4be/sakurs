//! State tracking for cross-chunk processing

use super::{
    constants::{DEFAULT_DEDUP_WINDOW, DEFAULT_MIN_CONFIDENCE},
    types::*,
};
use crate::application::config::ProcessingResult;
use std::collections::HashMap;

/// Tracks state across chunk boundaries for pattern detection
pub struct CrossChunkStateTracker {
    /// States for each chunk transition
    chunk_states: Vec<ChunkTransitionState>,

    /// Cache of detected boundaries for deduplication
    boundary_cache: BoundaryCache,

    /// Configuration for state tracking
    config: StateTrackerConfig,
}

/// Cache for boundary deduplication
#[derive(Debug, Default)]
pub struct BoundaryCache {
    /// Map of position to chunk indices where boundary was detected
    boundaries: HashMap<usize, Vec<usize>>,

    /// Suppressed boundary positions
    suppressed_positions: Vec<usize>,
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

impl CrossChunkStateTracker {
    /// Creates a new state tracker
    pub fn new(config: StateTrackerConfig) -> Self {
        Self {
            chunk_states: Vec::new(),
            boundary_cache: BoundaryCache::default(),
            config,
        }
    }

    /// Creates a new state tracker with default configuration
    pub fn with_defaults() -> Self {
        Self::new(StateTrackerConfig::default())
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

impl BoundaryCache {
    /// Clears the cache
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.boundaries.clear();
        self.suppressed_positions.clear();
    }

    /// Gets the number of cached boundaries
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.boundaries.len()
    }

    /// Checks if the cache is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.boundaries.is_empty()
    }
}
