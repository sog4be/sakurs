//! Overlap-based chunking with cross-chunk pattern detection
//!
//! This module provides an overlap-based chunking system that can detect and handle
//! patterns (like contractions and possessives) that span chunk boundaries.

mod constants;
mod overlap_chunk_manager;
mod overlap_processor;
mod pattern_detector;
mod state_tracker;
mod types;

pub use overlap_chunk_manager::{OverlapChunkConfig, OverlapChunkManager};
pub use overlap_processor::OverlapProcessor;
pub use state_tracker::{CrossChunkStateTracker, StateTrackerConfig};
pub use types::{
    BoundaryAdjustment, ChunkTransitionState, OverlapResult, PartialPattern, PatternType,
    ProcessedChunk, SuppressionMarker, SuppressionReason,
};
