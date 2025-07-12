//! Enhanced chunking with cross-chunk pattern detection
//!
//! This module provides an enhanced chunking system that can detect and handle
//! patterns (like contractions and possessives) that span chunk boundaries.

mod enhanced_chunk_manager;
mod overlap_processor;
mod state_tracker;
mod types;

pub use enhanced_chunk_manager::{EnhancedChunkConfig, EnhancedChunkManager};
pub use overlap_processor::OverlapProcessor;
pub use state_tracker::{CrossChunkStateTracker, StateTrackerConfig};
pub use types::{
    BoundaryAdjustment, ChunkTransitionState, OverlapResult, PartialPattern, PatternType,
    ProcessedChunk, SuppressionMarker, SuppressionReason,
};
