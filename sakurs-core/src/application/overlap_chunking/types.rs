//! Types for overlap-based chunking

use crate::application::chunking::TextChunk;
use std::collections::HashMap;

/// A processed chunk with additional metadata for cross-chunk handling
#[derive(Debug, Clone)]
pub struct ProcessedChunk {
    /// The base text chunk
    pub base_chunk: TextChunk,

    /// Suppression markers for positions in this chunk
    pub suppression_markers: Vec<SuppressionMarker>,

    /// State information for chunk transitions
    pub transition_state: Option<ChunkTransitionState>,

    /// Boundaries detected in overlap regions (for deduplication)
    pub overlap_boundaries: Vec<usize>,
}

/// Marker for a suppressed enclosure position
#[derive(Debug, Clone, PartialEq)]
pub struct SuppressionMarker {
    /// Position in the chunk (byte offset)
    pub position: usize,

    /// Character that was suppressed
    pub character: char,

    /// Reason for suppression
    pub reason: SuppressionReason,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Whether this was detected in an overlap region
    pub from_overlap: bool,
}

/// Reasons for suppressing an enclosure
#[derive(Debug, Clone, PartialEq)]
pub enum SuppressionReason {
    /// Contraction (e.g., "isn't", "don't")
    Contraction,

    /// Possessive (e.g., "James'", "students'")
    Possessive,

    /// Measurement mark (e.g., 5'9", 45°30')
    Measurement,

    /// List item (e.g., "1)", "a)")
    ListItem,

    /// Cross-chunk pattern detected
    CrossChunkPattern { pattern: String },
}

/// Result of processing an overlap region
#[derive(Debug, Clone)]
pub struct OverlapResult {
    /// Suppression markers found in the overlap
    pub suppressions: Vec<SuppressionMarker>,

    /// Adjustments to boundary positions
    pub boundary_adjustments: Vec<BoundaryAdjustment>,

    /// Extended context used for detection
    pub extended_context: String,

    /// Partial patterns that may continue
    pub partial_patterns: Vec<PartialPattern>,
}

/// Adjustment to a boundary position
#[derive(Debug, Clone)]
pub struct BoundaryAdjustment {
    /// Original boundary position
    pub original_position: usize,

    /// New position (or None if boundary should be removed)
    pub adjusted_position: Option<usize>,

    /// Reason for adjustment
    pub reason: String,
}

/// A pattern that may continue across chunks
#[derive(Debug, Clone)]
pub struct PartialPattern {
    /// The partial text (e.g., "isn" at chunk end)
    pub text: String,

    /// Expected continuation patterns
    pub expected_continuations: Vec<String>,

    /// Pattern type
    pub pattern_type: PatternType,
}

/// Types of patterns that can span chunks
#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    Contraction,
    Possessive,
    Abbreviation,
    Quote,
}

/// State information for chunk transitions
#[derive(Debug, Clone)]
pub struct ChunkTransitionState {
    /// Index of this chunk
    pub chunk_index: usize,

    /// Suppression markers that affect the next chunk
    pub forward_suppressions: Vec<SuppressionMarker>,

    /// Partial patterns at chunk end
    pub ending_patterns: Vec<PartialPattern>,

    /// Partial patterns at chunk start
    pub starting_patterns: Vec<PartialPattern>,

    /// Open enclosures at chunk end
    pub open_enclosures: HashMap<usize, char>,

    /// Confidence scores for cross-chunk detections
    pub pattern_confidences: HashMap<String, f32>,
}

impl ProcessedChunk {
    /// Creates a new processed chunk from a base chunk
    pub fn from_base(base_chunk: TextChunk) -> Self {
        Self {
            base_chunk,
            suppression_markers: Vec::new(),
            transition_state: None,
            overlap_boundaries: Vec::new(),
        }
    }

    /// Adds a suppression marker
    pub fn add_suppression(&mut self, marker: SuppressionMarker) {
        // Check if this position is already suppressed
        if !self
            .suppression_markers
            .iter()
            .any(|m| m.position == marker.position)
        {
            self.suppression_markers.push(marker);
        }
    }

    /// Checks if a position should be suppressed
    pub fn is_suppressed(&self, position: usize) -> bool {
        self.suppression_markers
            .iter()
            .any(|m| m.position == position)
    }

    /// Gets suppression markers in a range
    pub fn suppressions_in_range(&self, start: usize, end: usize) -> Vec<&SuppressionMarker> {
        self.suppression_markers
            .iter()
            .filter(|m| m.position >= start && m.position < end)
            .collect()
    }
}

impl ChunkTransitionState {
    /// Creates a new transition state
    pub fn new(chunk_index: usize) -> Self {
        Self {
            chunk_index,
            forward_suppressions: Vec::new(),
            ending_patterns: Vec::new(),
            starting_patterns: Vec::new(),
            open_enclosures: HashMap::new(),
            pattern_confidences: HashMap::new(),
        }
    }

    /// Adds a pattern confidence score
    pub fn add_pattern_confidence(&mut self, pattern: String, confidence: f32) {
        self.pattern_confidences.insert(pattern, confidence);
    }

    /// Gets the confidence for a pattern
    pub fn get_pattern_confidence(&self, pattern: &str) -> Option<f32> {
        self.pattern_confidences.get(pattern).copied()
    }
}
