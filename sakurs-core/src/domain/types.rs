//! Type definitions for the domain layer
//!
//! This module consolidates all type definitions used across the domain layer,
//! including boundary representations, state tracking, and optimized collections.

use super::monoid::{Monoid, MonoidReduce};
use smallvec::SmallVec;

// ============================================================================
// Boundary Types
// ============================================================================

/// Represents a sentence boundary with metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Boundary {
    /// Byte offset of the boundary in the text
    pub offset: usize,
    /// Boundary classification flags
    pub flags: BoundaryFlags,
}

/// Classification flags for sentence boundaries
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundaryFlags {
    /// Strong boundary (e.g., period followed by capital letter)
    pub is_strong: bool,
    /// Boundary detected after abbreviation resolution
    pub from_abbreviation: bool,
}

impl BoundaryFlags {
    pub const STRONG: Self = Self {
        is_strong: true,
        from_abbreviation: false,
    };
    pub const WEAK: Self = Self {
        is_strong: false,
        from_abbreviation: false,
    };
    pub const FROM_ABBR: Self = Self {
        is_strong: true,
        from_abbreviation: true,
    };

    /// Check if this flags contains the STRONG flag
    pub fn contains(&self, other: Self) -> bool {
        if other.is_strong && !self.is_strong {
            return false;
        }
        if other.from_abbreviation && !self.from_abbreviation {
            return false;
        }
        true
    }
}

impl Boundary {
    /// Creates a new boundary at the specified offset
    pub fn new(offset: usize) -> Self {
        Self {
            offset,
            flags: BoundaryFlags::WEAK,
        }
    }

    /// Creates a new boundary with specific flags
    pub fn with_flags(offset: usize, flags: BoundaryFlags) -> Self {
        Self { offset, flags }
    }

    /// Sets the may_be_abbrev flag (compatibility method)
    pub fn set_may_be_abbrev(&mut self) {
        self.flags.from_abbreviation = true;
    }

    /// Gets the may_be_abbrev flag (compatibility method)
    pub fn may_be_abbrev(&self) -> bool {
        self.flags.from_abbreviation
    }
}

/// Represents a confirmed sentence boundary
/// Note: This type is currently only used in tests
#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmedBoundary {
    /// Position in the text
    pub position: usize,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

impl ConfirmedBoundary {
    /// Create a new confirmed boundary
    pub fn new(position: usize, confidence: f32) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&confidence),
            "Confidence must be between 0.0 and 1.0"
        );
        Self {
            position,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Create a high-confidence boundary
    pub fn high_confidence(position: usize) -> Self {
        Self::new(position, 1.0)
    }

    /// Create a medium-confidence boundary
    pub fn medium_confidence(position: usize) -> Self {
        Self::new(position, 0.75)
    }
}

/// Boundary candidate found during scanning phase
///
/// Unlike confirmed boundaries, candidates store local depth information
/// to enable deferred boundary resolution in the reduce phase
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryCandidate {
    /// Offset within the chunk (not global offset)
    pub local_offset: usize,

    /// Local depths at this position (relative to chunk start)
    pub local_depths: DepthVec,

    /// Boundary classification flags
    pub flags: BoundaryFlags,
}

// ============================================================================
// State Types
// ============================================================================

/// Delta entry for tracking enclosure state across chunks
#[derive(Debug, Clone, PartialEq)]
pub struct DeltaEntry {
    /// Net count: opening delimiters - closing delimiters
    pub net: i32,
    /// Minimum cumulative sum observed during chunk processing
    pub min: i32,
}

impl DeltaEntry {
    /// Creates a new delta entry
    pub fn new(net: i32, min: i32) -> Self {
        Self { net, min }
    }

    /// Identity delta entry (no enclosures)
    pub fn identity() -> Self {
        Self { net: 0, min: 0 }
    }

    /// Combines two delta entries
    /// Following the mathematical definition: δ₁ ⊕ δ₂ = (n₁ + n₂, min(m₁, n₁ + m₂))
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            net: self.net + other.net,
            min: self.min.min(self.net + other.min),
        }
    }
}

/// Simplified abbreviation state for cross-chunk tracking
///
/// Based on the NAACL 2024 paper's linear cross-chunk abbreviation detection
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AbbreviationState {
    /// Ends with period preceded by alphabetic character (e.g., "Dr.")
    pub dangling_dot: bool,
    /// Starts with alphabetic character (e.g., "Smith")
    pub head_alpha: bool,
}

impl AbbreviationState {
    /// Creates a new abbreviation state
    pub fn new(dangling_dot: bool, head_alpha: bool) -> Self {
        Self {
            dangling_dot,
            head_alpha,
        }
    }

    /// Identity abbreviation state (empty chunk)
    pub fn identity() -> Self {
        Self {
            dangling_dot: false,
            head_alpha: false,
        }
    }

    /// Combines two abbreviation states
    /// For sequential chunks: left.combine(right)
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            // Only the rightmost chunk's dangling dot matters
            dangling_dot: other.dangling_dot,
            // Only the leftmost chunk's head alpha matters
            head_alpha: self.head_alpha,
        }
    }

    /// Returns true if this represents a cross-chunk abbreviation
    pub fn is_cross_chunk_abbr(&self, other: &Self) -> bool {
        self.dangling_dot && other.head_alpha
    }
}

/// Context for abbreviation detection
#[derive(Clone, Debug, Default)]
pub struct AbbreviationContext {
    /// Whether we're currently in an abbreviation
    pub in_abbreviation: bool,
    /// The abbreviation text if any
    pub current_abbreviation: Option<String>,
    /// Position where abbreviation started
    pub start_position: Option<usize>,
}

impl AbbreviationContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking an abbreviation
    pub fn start_abbreviation(&mut self, text: String, position: usize) {
        self.in_abbreviation = true;
        self.current_abbreviation = Some(text);
        self.start_position = Some(position);
    }

    /// End abbreviation tracking
    pub fn end_abbreviation(&mut self) {
        self.in_abbreviation = false;
        self.current_abbreviation = None;
        self.start_position = None;
    }
}

// ============================================================================
// Collection Types (SmallVec Optimizations)
// ============================================================================

/// Optimized vector for boundary candidates
/// Most chunks have < 32 boundary candidates
pub type BoundaryVec = SmallVec<[BoundaryCandidate; 32]>;

/// Optimized vector for delta entries
/// Enclosure tracking rarely needs > 16 entries per chunk
pub type DeltaVec = SmallVec<[DeltaEntry; 16]>;

/// Optimized vector for local depths
/// Enclosure depth rarely exceeds 8 levels
pub type DepthVec = SmallVec<[i32; 8]>;

/// Optimized vector for small integer collections
pub type SmallIntVec = SmallVec<[i32; 4]>;

// ============================================================================
// Algorithm State Types
// ============================================================================

/// Partial state representation for the Delta-Stack algorithm
///
/// This represents the state ⟨B, Δ, A⟩ as described in the algorithm documentation:
/// - B: Set of boundary candidates (not yet confirmed)
/// - Δ: Vector of delta entries for different enclosure types  
/// - A: Abbreviation state for cross-chunk handling
#[derive(Debug, Clone, PartialEq)]
pub struct PartialState {
    /// Boundary candidates found in this chunk
    pub boundary_candidates: BoundaryVec,

    /// Delta stack entries for enclosure tracking
    pub deltas: DeltaVec,

    /// Abbreviation state for cross-chunk processing
    pub abbreviation: AbbreviationState,

    /// Length of the text chunk this state represents
    pub chunk_length: usize,
}

impl PartialState {
    /// Creates a new partial state
    pub fn new(enclosure_count: usize) -> Self {
        Self {
            boundary_candidates: BoundaryVec::new(),
            deltas: DeltaVec::from_vec(vec![DeltaEntry::identity(); enclosure_count]),
            abbreviation: AbbreviationState::identity(),
            chunk_length: 0,
        }
    }

    /// Adds a boundary candidate to this state
    pub fn add_boundary_candidate(
        &mut self,
        local_offset: usize,
        local_depths: DepthVec,
        flags: BoundaryFlags,
    ) {
        self.boundary_candidates.push(BoundaryCandidate {
            local_offset,
            local_depths,
            flags,
        });
    }

    /// Sets the delta entry for a specific enclosure type
    pub fn set_delta(&mut self, enclosure_id: usize, delta: DeltaEntry) {
        if enclosure_id < self.deltas.len() {
            self.deltas[enclosure_id] = delta;
        }
    }

    /// Sets the abbreviation state
    pub fn set_abbreviation(&mut self, abbr: AbbreviationState) {
        self.abbreviation = abbr;
    }

    /// Returns true if we're currently inside any enclosure
    pub fn is_inside_enclosure(&self) -> bool {
        self.deltas.iter().any(|delta| delta.net > 0)
    }
}

impl Monoid for PartialState {
    fn identity() -> Self {
        Self {
            boundary_candidates: BoundaryVec::new(),
            deltas: DeltaVec::new(),
            abbreviation: AbbreviationState::identity(),
            chunk_length: 0,
        }
    }

    fn combine(&self, other: &Self) -> Self {
        // Ensure both states have the same number of enclosure types
        let max_deltas = self.deltas.len().max(other.deltas.len());

        // Combine boundary candidates, adjusting offsets for the right chunk
        let mut combined_candidates = self.boundary_candidates.clone();
        for candidate in &other.boundary_candidates {
            combined_candidates.push(BoundaryCandidate {
                local_offset: candidate.local_offset + self.chunk_length,
                local_depths: candidate.local_depths.clone(),
                flags: candidate.flags,
            });
        }

        // Note: Cross-chunk abbreviation handling will be done in the reduce phase
        // when we have access to global depths

        // Combine delta entries
        let mut combined_deltas = DeltaVec::with_capacity(max_deltas);
        let identity = DeltaEntry::identity();
        for i in 0..max_deltas {
            let left_delta = self.deltas.get(i).unwrap_or(&identity);
            let right_delta = other.deltas.get(i).unwrap_or(&identity);
            combined_deltas.push(left_delta.combine(right_delta));
        }

        // Combine abbreviation states
        let combined_abbr = self.abbreviation.combine(&other.abbreviation);

        Self {
            boundary_candidates: combined_candidates,
            deltas: combined_deltas,
            abbreviation: combined_abbr,
            chunk_length: self.chunk_length + other.chunk_length,
        }
    }
}

impl MonoidReduce for PartialState {}

impl Default for PartialState {
    fn default() -> Self {
        Self::identity()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    include!("types_tests.rs");
}
