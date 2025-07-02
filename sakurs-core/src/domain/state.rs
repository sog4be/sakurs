//! State representation for the Delta-Stack algorithm
//!
//! This module implements the core state representation ⟨B, Δ, A⟩ where:
//! - B: Boundary set with detected sentence boundaries
//! - Δ: Delta stack tracking enclosure states
//! - A: Abbreviation state for cross-chunk handling

use super::monoid::{Monoid, MonoidReduce};
use std::collections::BTreeSet;

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
}

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
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            net: self.net + other.net,
            min: self.min.min(self.net + other.min),
        }
    }
}

/// Abbreviation state for handling cross-chunk abbreviations
#[derive(Debug, Clone, PartialEq)]
pub struct AbbreviationState {
    /// True if the chunk ends with a potential abbreviation dot
    pub dangling_dot: bool,
    /// True if the chunk starts with alphabetic characters
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

    /// Identity abbreviation state
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

/// Partial state representation for the Delta-Stack algorithm
///
/// This represents the state ⟨B, Δ, A⟩ as described in the algorithm documentation:
/// - B: Set of detected sentence boundaries
/// - Δ: Vector of delta entries for different enclosure types  
/// - A: Abbreviation state for cross-chunk handling
#[derive(Debug, Clone, PartialEq)]
pub struct PartialState {
    /// Set of detected sentence boundaries
    pub boundaries: BTreeSet<Boundary>,
    /// Delta stack entries for enclosure tracking
    pub deltas: Vec<DeltaEntry>,
    /// Abbreviation state for cross-chunk processing
    pub abbreviation: AbbreviationState,
    /// Length of the text chunk this state represents
    pub chunk_length: usize,
}

impl PartialState {
    /// Creates a new partial state
    pub fn new(enclosure_count: usize) -> Self {
        Self {
            boundaries: BTreeSet::new(),
            deltas: vec![DeltaEntry::identity(); enclosure_count],
            abbreviation: AbbreviationState::identity(),
            chunk_length: 0,
        }
    }

    /// Adds a boundary to this state
    pub fn add_boundary(&mut self, offset: usize, flags: BoundaryFlags) {
        self.boundaries.insert(Boundary { offset, flags });
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

    /// Adjusts all boundary offsets by the given amount
    #[allow(dead_code)]
    fn adjust_offsets(&mut self, offset: usize) {
        let adjusted_boundaries: BTreeSet<Boundary> = self
            .boundaries
            .iter()
            .map(|b| Boundary {
                offset: b.offset + offset,
                flags: b.flags,
            })
            .collect();
        self.boundaries = adjusted_boundaries;
    }
}

impl Monoid for PartialState {
    fn identity() -> Self {
        Self {
            boundaries: BTreeSet::new(),
            deltas: Vec::new(),
            abbreviation: AbbreviationState::identity(),
            chunk_length: 0,
        }
    }

    fn combine(&self, other: &Self) -> Self {
        // Ensure both states have the same number of enclosure types
        let max_deltas = self.deltas.len().max(other.deltas.len());

        // Combine boundaries, adjusting offsets for the right chunk
        let mut combined_boundaries = self.boundaries.clone();
        for boundary in &other.boundaries {
            combined_boundaries.insert(Boundary {
                offset: boundary.offset + self.chunk_length,
                flags: boundary.flags,
            });
        }

        // Handle cross-chunk abbreviations
        if self.abbreviation.is_cross_chunk_abbr(&other.abbreviation) {
            // Remove boundaries that are affected by cross-chunk abbreviation
            // This is a simplified version - full implementation would need more context
            combined_boundaries.retain(|b| {
                // Keep boundaries that aren't at the chunk boundary
                b.offset != self.chunk_length
            });
        }

        // Combine delta entries
        let mut combined_deltas = Vec::with_capacity(max_deltas);
        let identity = DeltaEntry::identity();
        for i in 0..max_deltas {
            let left_delta = self.deltas.get(i).unwrap_or(&identity);
            let right_delta = other.deltas.get(i).unwrap_or(&identity);
            combined_deltas.push(left_delta.combine(right_delta));
        }

        // Combine abbreviation states
        let combined_abbr = self.abbreviation.combine(&other.abbreviation);

        Self {
            boundaries: combined_boundaries,
            deltas: combined_deltas,
            abbreviation: combined_abbr,
            chunk_length: self.chunk_length + other.chunk_length,
        }
    }
}

impl MonoidReduce for PartialState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_ordering() {
        let b1 = Boundary {
            offset: 10,
            flags: BoundaryFlags::STRONG,
        };
        let b2 = Boundary {
            offset: 20,
            flags: BoundaryFlags::WEAK,
        };
        assert!(b1 < b2);
    }

    #[test]
    fn test_delta_entry_combine() {
        let d1 = DeltaEntry::new(2, -1); // 2 opens, went down to -1
        let d2 = DeltaEntry::new(-1, -3); // 1 close, went down to -3

        let combined = d1.combine(&d2);
        assert_eq!(combined.net, 1); // 2 + (-1) = 1
        assert_eq!(combined.min, -1); // min(-1, 2 + (-3)) = min(-1, -1) = -1
    }

    #[test]
    fn test_abbreviation_state_combine() {
        let left = AbbreviationState::new(true, false); // ends with dot
        let right = AbbreviationState::new(false, true); // starts with alpha

        let combined = left.combine(&right);
        assert!(!combined.dangling_dot); // takes right's dangling_dot
        assert!(!combined.head_alpha); // takes left's head_alpha
    }

    #[test]
    fn test_cross_chunk_abbreviation_detection() {
        let left = AbbreviationState::new(true, false); // ends with dot
        let right = AbbreviationState::new(false, true); // starts with alpha

        assert!(left.is_cross_chunk_abbr(&right));
    }

    #[test]
    fn test_partial_state_identity() {
        let state = PartialState::identity();
        assert!(state.boundaries.is_empty());
        assert!(state.deltas.is_empty());
        assert!(!state.abbreviation.dangling_dot);
        assert!(!state.abbreviation.head_alpha);
        assert_eq!(state.chunk_length, 0);
    }

    #[test]
    fn test_partial_state_combine() {
        let mut left = PartialState::new(2);
        left.add_boundary(5, BoundaryFlags::STRONG);
        left.chunk_length = 10;
        left.deltas[0] = DeltaEntry::new(1, 0);

        let mut right = PartialState::new(2);
        right.add_boundary(3, BoundaryFlags::WEAK);
        right.chunk_length = 8;
        right.deltas[0] = DeltaEntry::new(-1, -1);

        let combined = left.combine(&right);

        assert_eq!(combined.chunk_length, 18);
        assert_eq!(combined.boundaries.len(), 2);
        assert_eq!(combined.deltas[0].net, 0); // 1 + (-1) = 0

        // Check boundary offset adjustment
        let boundary_offsets: Vec<usize> = combined.boundaries.iter().map(|b| b.offset).collect();
        assert!(boundary_offsets.contains(&5)); // original left boundary
        assert!(boundary_offsets.contains(&13)); // right boundary adjusted by 10
    }

    #[test]
    fn test_monoid_properties() {
        let state1 = PartialState::new(1);
        let identity = PartialState::identity();

        // Identity property
        assert_eq!(state1.combine(&identity), state1);
        assert_eq!(identity.combine(&state1), state1);

        // Associativity (simplified test)
        let state2 = PartialState::new(1);
        let state3 = PartialState::new(1);

        let left_assoc = state1.combine(&state2).combine(&state3);
        let right_assoc = state1.combine(&state2.combine(&state3));

        // For empty states, should be equal
        assert_eq!(left_assoc.boundaries.len(), right_assoc.boundaries.len());
        assert_eq!(left_assoc.chunk_length, right_assoc.chunk_length);
    }
}
