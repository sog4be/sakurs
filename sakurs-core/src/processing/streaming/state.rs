//! State management for streaming processing with bounded memory

use std::collections::VecDeque;

/// Maximum number of recent boundaries to keep in memory
const MAX_BOUNDARY_HISTORY: usize = 1000;

/// State manager for streaming sentence processing
#[derive(Debug)]
pub struct StreamingState {
    /// Total number of sentences found
    sentence_count: usize,
    /// Current byte position in the stream
    current_position: usize,
    /// Recent sentence boundaries (for context)
    recent_boundaries: VecDeque<usize>,
    /// Pending boundary from previous chunk
    pending_boundary: Option<usize>,
    /// Offset adjustment for boundary positions
    position_offset: usize,
}

impl Default for StreamingState {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingState {
    /// Create a new streaming state
    pub fn new() -> Self {
        Self {
            sentence_count: 0,
            current_position: 0,
            recent_boundaries: VecDeque::with_capacity(MAX_BOUNDARY_HISTORY),
            pending_boundary: None,
            position_offset: 0,
        }
    }

    /// Update state with new boundaries from current chunk
    pub fn update(&mut self, chunk_boundaries: &[usize], chunk_offset: usize) {
        // Process pending boundary if exists
        if let Some(boundary) = self.pending_boundary.take() {
            self.add_boundary(boundary);
        }

        // Add new boundaries with offset adjustment
        for &boundary in chunk_boundaries {
            let absolute_boundary = boundary + chunk_offset + self.position_offset;
            self.add_boundary(absolute_boundary);
        }

        // Update position
        self.current_position = chunk_offset + self.position_offset;
    }

    /// Add a boundary to the state
    fn add_boundary(&mut self, boundary: usize) {
        self.sentence_count += 1;
        self.recent_boundaries.push_back(boundary);

        // Maintain bounded memory
        if self.recent_boundaries.len() > MAX_BOUNDARY_HISTORY {
            self.recent_boundaries.pop_front();
        }
    }

    /// Set a pending boundary for the next chunk
    pub fn set_pending_boundary(&mut self, boundary: Option<usize>) {
        self.pending_boundary = boundary;
    }

    /// Update position offset (for handling look-ahead overlap)
    pub fn update_position_offset(&mut self, offset: usize) {
        self.position_offset += offset;
    }

    /// Get total sentence count
    pub fn sentence_count(&self) -> usize {
        self.sentence_count
    }

    /// Get current position in stream
    pub fn current_position(&self) -> usize {
        self.current_position
    }

    /// Get recent boundaries (for debugging/validation)
    pub fn recent_boundaries(&self) -> &VecDeque<usize> {
        &self.recent_boundaries
    }

    /// Create a checkpoint for recovery
    pub fn checkpoint(&self) -> StateCheckpoint {
        StateCheckpoint {
            sentence_count: self.sentence_count,
            current_position: self.current_position,
            position_offset: self.position_offset,
            last_boundary: self.recent_boundaries.back().copied(),
        }
    }

    /// Restore from checkpoint
    pub fn restore(&mut self, checkpoint: StateCheckpoint) {
        self.sentence_count = checkpoint.sentence_count;
        self.current_position = checkpoint.current_position;
        self.position_offset = checkpoint.position_offset;
        self.recent_boundaries.clear();
        if let Some(boundary) = checkpoint.last_boundary {
            self.recent_boundaries.push_back(boundary);
        }
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.sentence_count = 0;
        self.current_position = 0;
        self.recent_boundaries.clear();
        self.pending_boundary = None;
        self.position_offset = 0;
    }
}

/// Checkpoint for state recovery
#[derive(Debug, Clone)]
pub struct StateCheckpoint {
    sentence_count: usize,
    current_position: usize,
    position_offset: usize,
    last_boundary: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_update() {
        let mut state = StreamingState::new();

        // First chunk boundaries
        state.update(&[10, 25, 40], 0);
        assert_eq!(state.sentence_count(), 3);

        // Second chunk boundaries with offset
        state.update(&[15, 30], 50);
        assert_eq!(state.sentence_count(), 5);
    }

    #[test]
    fn test_bounded_memory() {
        let mut state = StreamingState::new();

        // Add more boundaries than the limit
        let boundaries: Vec<usize> = (0..MAX_BOUNDARY_HISTORY + 100).map(|i| i * 10).collect();

        for chunk in boundaries.chunks(100) {
            state.update(chunk, 0);
        }

        // Should only keep recent boundaries
        assert!(state.recent_boundaries.len() <= MAX_BOUNDARY_HISTORY);
    }

    #[test]
    fn test_checkpoint_restore() {
        let mut state = StreamingState::new();
        state.update(&[10, 20, 30], 0);

        let checkpoint = state.checkpoint();

        // Modify state
        state.update(&[40, 50], 30);

        // Restore
        state.restore(checkpoint);
        assert_eq!(state.sentence_count(), 3);
        assert_eq!(state.current_position(), 0);
    }
}
