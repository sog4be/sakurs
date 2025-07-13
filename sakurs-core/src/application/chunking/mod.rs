//! Text chunking functionality for efficient processing
//!
//! This module provides both basic and overlap-based chunking strategies
//! for processing large texts efficiently while maintaining UTF-8 safety
//! and handling cross-chunk patterns.

pub mod base;
pub mod overlap;

// Re-export base chunking types
pub use base::{ChunkManager, TextChunk};

// Re-export overlap chunking types for backward compatibility
pub use overlap::{
    BoundaryAdjustment, ChunkTransitionState, OverlapChunkConfig, OverlapChunkManager,
    OverlapResult, PartialPattern, PatternType, ProcessedChunk, SuppressionMarker,
    SuppressionReason,
};
