//! Overlap-based chunking with cross-chunk pattern detection
//!
//! This module provides an overlap-based chunking system that can detect and handle
//! patterns (like contractions and possessives) that span chunk boundaries.

mod manager;
mod processor;
mod types;

// Re-export public types
pub use manager::{OverlapChunkConfig, OverlapChunkManager, StateTrackerConfig};
pub use processor::OverlapProcessor;
pub use types::{
    BoundaryAdjustment, ChunkTransitionState, OverlapResult, PartialPattern, PatternType,
    ProcessedChunk, SuppressionMarker, SuppressionReason,
};
