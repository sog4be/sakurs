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

    /// Check if this flags contains all the flags set in `other`
    ///
    /// Returns true if all flags that are set in `other` are also set in `self`.
    /// This is useful for checking if a boundary meets certain flag requirements.
    ///
    /// # Example
    /// ```ignore
    /// let flags = BoundaryFlags { is_strong: true, from_abbreviation: true };
    /// assert!(flags.contains(BoundaryFlags::STRONG)); // true, has is_strong
    /// assert!(flags.contains(BoundaryFlags::FROM_ABBR)); // true, has both flags
    /// ```
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
    /// First word of the chunk (for sentence starter detection)
    pub first_word: Option<String>,
}

impl AbbreviationState {
    /// Creates a new abbreviation state
    pub fn new(dangling_dot: bool, head_alpha: bool) -> Self {
        Self {
            dangling_dot,
            head_alpha,
            first_word: None,
        }
    }

    /// Creates a new abbreviation state with first word
    pub fn with_first_word(
        dangling_dot: bool,
        head_alpha: bool,
        first_word: Option<String>,
    ) -> Self {
        Self {
            dangling_dot,
            head_alpha,
            first_word,
        }
    }

    /// Identity abbreviation state (empty chunk)
    pub fn identity() -> Self {
        Self {
            dangling_dot: false,
            head_alpha: false,
            first_word: None,
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
            // Only the leftmost chunk's first word matters
            first_word: self.first_word.clone(),
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
    use super::*;
    use crate::domain::Monoid;

    mod boundary_tests {
        use super::*;

        #[test]
        fn test_new_with_valid_confidence() {
            // Test various valid confidence values
            let test_cases = vec![
                (100, 0.0),
                (200, 0.5),
                (300, 1.0),
                (400, 0.25),
                (500, 0.75),
                (600, 0.999),
                (700, 0.001),
            ];

            for (position, confidence) in test_cases {
                let boundary = ConfirmedBoundary::new(position, confidence);
                assert_eq!(boundary.position, position);
                assert_eq!(boundary.confidence, confidence);
                assert!(boundary.confidence >= 0.0 && boundary.confidence <= 1.0);
            }
        }

        #[test]
        #[cfg(not(debug_assertions))]
        fn test_new_with_invalid_confidence_clamping() {
            // Test confidence values outside valid range get clamped (in release mode)
            let test_cases = vec![
                (100, -0.5, 0.0),   // Below minimum
                (200, -100.0, 0.0), // Far below minimum
                (300, 1.5, 1.0),    // Above maximum
                (400, 100.0, 1.0),  // Far above maximum
            ];

            for (position, input_confidence, expected_confidence) in test_cases {
                let boundary = ConfirmedBoundary::new(position, input_confidence);
                assert_eq!(boundary.position, position);
                assert_eq!(boundary.confidence, expected_confidence);
            }
        }

        #[test]
        #[cfg(debug_assertions)]
        #[should_panic(expected = "Confidence must be between 0.0 and 1.0")]
        fn test_new_with_invalid_confidence_panics_in_debug() {
            // Test that invalid confidence causes panic in debug mode
            ConfirmedBoundary::new(100, -0.5);
        }

        #[test]
        #[cfg(debug_assertions)]
        #[should_panic(expected = "Confidence must be between 0.0 and 1.0")]
        fn test_new_with_too_high_confidence_panics_in_debug() {
            // Test that confidence > 1.0 causes panic in debug mode
            ConfirmedBoundary::new(100, 1.5);
        }

        #[test]
        fn test_high_confidence_factory() {
            let boundary = ConfirmedBoundary::high_confidence(1000);
            assert_eq!(boundary.position, 1000);
            assert_eq!(boundary.confidence, 1.0);
        }

        #[test]
        fn test_medium_confidence_factory() {
            let boundary = ConfirmedBoundary::medium_confidence(2000);
            assert_eq!(boundary.position, 2000);
            assert_eq!(boundary.confidence, 0.75);
        }

        #[test]
        fn test_boundary_equality() {
            let b1 = ConfirmedBoundary::new(100, 0.8);
            let b2 = ConfirmedBoundary::new(100, 0.8);
            let b3 = ConfirmedBoundary::new(100, 0.9);
            let b4 = ConfirmedBoundary::new(200, 0.8);

            assert_eq!(b1, b2);
            assert_ne!(b1, b3); // Different confidence
            assert_ne!(b1, b4); // Different position
        }

        #[test]
        fn test_boundary_cloning() {
            let original = ConfirmedBoundary::new(500, 0.95);
            let cloned = original.clone();

            assert_eq!(cloned.position, original.position);
            assert_eq!(cloned.confidence, original.confidence);
            assert_eq!(cloned, original);
        }

        #[test]
        fn test_boundary_debug_format() {
            let boundary = ConfirmedBoundary::new(123, 0.85);
            let debug_str = format!("{:?}", boundary);

            // Debug format should contain both position and confidence
            assert!(debug_str.contains("123"));
            assert!(debug_str.contains("0.85"));
        }

        #[test]
        fn test_edge_case_positions() {
            // Test with edge case position values
            let test_cases = vec![
                0,          // Minimum position
                1,          // Small position
                usize::MAX, // Maximum position
            ];

            for position in test_cases {
                let boundary = ConfirmedBoundary::new(position, 0.5);
                assert_eq!(boundary.position, position);
            }
        }

        #[test]
        fn test_confidence_precision() {
            // Test that confidence maintains reasonable precision
            let boundary = ConfirmedBoundary::new(100, 0.123_456_79);

            // Confidence should be stored with f32 precision
            assert!((boundary.confidence - 0.123_456_79_f32).abs() < 0.0000001);
        }
    }

    mod state_tests {
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
            assert!(state.boundary_candidates.is_empty());
            assert!(state.deltas.is_empty());
            assert!(!state.abbreviation.dangling_dot);
            assert!(!state.abbreviation.head_alpha);
            assert_eq!(state.chunk_length, 0);
        }

        #[test]
        fn test_partial_state_combine() {
            let mut left = PartialState::new(2);
            left.add_boundary_candidate(5, DepthVec::from_vec(vec![0, 0]), BoundaryFlags::STRONG);
            left.chunk_length = 10;
            left.deltas[0] = DeltaEntry::new(1, 0);

            let mut right = PartialState::new(2);
            right.add_boundary_candidate(3, DepthVec::from_vec(vec![0, 0]), BoundaryFlags::WEAK);
            right.chunk_length = 8;
            right.deltas[0] = DeltaEntry::new(-1, -1);

            let combined = left.combine(&right);

            assert_eq!(combined.chunk_length, 18);
            assert_eq!(combined.boundary_candidates.len(), 2);
            assert_eq!(combined.deltas[0].net, 0); // 1 + (-1) = 0

            // Check boundary offset adjustment
            let boundary_offsets: Vec<usize> = combined
                .boundary_candidates
                .iter()
                .map(|b| b.local_offset)
                .collect();
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
            assert_eq!(
                left_assoc.boundary_candidates.len(),
                right_assoc.boundary_candidates.len()
            );
            assert_eq!(left_assoc.chunk_length, right_assoc.chunk_length);
        }

        #[test]
        fn test_partial_state_boundary_candidates() {
            // This test verifies that PartialState can properly store boundary candidates
            // which is the core responsibility of this type
            let mut state = PartialState::new(1);

            // Add boundary candidates at different positions
            state.add_boundary_candidate(18, DepthVec::from_vec(vec![0]), BoundaryFlags::STRONG);
            state.add_boundary_candidate(34, DepthVec::from_vec(vec![0]), BoundaryFlags::STRONG);

            // Verify candidates are stored correctly
            assert_eq!(state.boundary_candidates.len(), 2);

            let positions: Vec<usize> = state
                .boundary_candidates
                .iter()
                .map(|b| b.local_offset)
                .collect();

            assert_eq!(positions, vec![18, 34]);

            // Verify flags are preserved
            assert_eq!(state.boundary_candidates[0].flags, BoundaryFlags::STRONG);
            assert_eq!(state.boundary_candidates[1].flags, BoundaryFlags::STRONG);
        }
    }

    mod abbreviation_context_tests {
        use super::*;

        #[test]
        fn test_new_creates_empty_context() {
            let context = AbbreviationContext::new();

            assert!(!context.in_abbreviation);
            assert_eq!(context.current_abbreviation, None);
            assert_eq!(context.start_position, None);
        }

        #[test]
        fn test_default_creates_empty_context() {
            let context = AbbreviationContext::default();

            assert!(!context.in_abbreviation);
            assert_eq!(context.current_abbreviation, None);
            assert_eq!(context.start_position, None);
        }

        #[test]
        fn test_start_abbreviation() {
            let mut context = AbbreviationContext::new();

            context.start_abbreviation("Dr".to_string(), 42);

            assert!(context.in_abbreviation);
            assert_eq!(context.current_abbreviation, Some("Dr".to_string()));
            assert_eq!(context.start_position, Some(42));
        }

        #[test]
        fn test_end_abbreviation() {
            let mut context = AbbreviationContext::new();

            // Start an abbreviation
            context.start_abbreviation("Inc".to_string(), 100);
            assert!(context.in_abbreviation);

            // End the abbreviation
            context.end_abbreviation();

            assert!(!context.in_abbreviation);
            assert_eq!(context.current_abbreviation, None);
            assert_eq!(context.start_position, None);
        }

        #[test]
        fn test_abbreviation_state_transitions() {
            let mut context = AbbreviationContext::new();

            // Initial state
            assert!(!context.in_abbreviation);

            // Start first abbreviation
            context.start_abbreviation("Mr".to_string(), 0);
            assert!(context.in_abbreviation);
            assert_eq!(context.current_abbreviation, Some("Mr".to_string()));
            assert_eq!(context.start_position, Some(0));

            // End abbreviation
            context.end_abbreviation();
            assert!(!context.in_abbreviation);

            // Start second abbreviation
            context.start_abbreviation("Prof".to_string(), 50);
            assert!(context.in_abbreviation);
            assert_eq!(context.current_abbreviation, Some("Prof".to_string()));
            assert_eq!(context.start_position, Some(50));
        }

        #[test]
        fn test_overwrite_abbreviation() {
            let mut context = AbbreviationContext::new();

            // Start first abbreviation
            context.start_abbreviation("Dr".to_string(), 10);

            // Start second abbreviation without ending first
            context.start_abbreviation("Mrs".to_string(), 20);

            // Should have the second abbreviation
            assert!(context.in_abbreviation);
            assert_eq!(context.current_abbreviation, Some("Mrs".to_string()));
            assert_eq!(context.start_position, Some(20));
        }

        #[test]
        fn test_empty_abbreviation_string() {
            let mut context = AbbreviationContext::new();

            // Start with empty string
            context.start_abbreviation("".to_string(), 0);

            assert!(context.in_abbreviation);
            assert_eq!(context.current_abbreviation, Some("".to_string()));
            assert_eq!(context.start_position, Some(0));
        }

        #[test]
        fn test_large_position_values() {
            let mut context = AbbreviationContext::new();

            // Test with large position values
            context.start_abbreviation("etc".to_string(), usize::MAX);

            assert!(context.in_abbreviation);
            assert_eq!(context.start_position, Some(usize::MAX));
        }

        #[test]
        fn test_unicode_abbreviations() {
            let mut context = AbbreviationContext::new();

            // Test with Unicode abbreviations
            context.start_abbreviation("株".to_string(), 100);
            assert_eq!(context.current_abbreviation, Some("株".to_string()));

            context.start_abbreviation("有限会社".to_string(), 200);
            assert_eq!(context.current_abbreviation, Some("有限会社".to_string()));
        }

        #[test]
        fn test_clone_context() {
            let mut original = AbbreviationContext::new();
            original.start_abbreviation("Ltd".to_string(), 42);

            let cloned = original.clone();

            assert_eq!(cloned.in_abbreviation, original.in_abbreviation);
            assert_eq!(cloned.current_abbreviation, original.current_abbreviation);
            assert_eq!(cloned.start_position, original.start_position);
        }

        #[test]
        fn test_debug_format() {
            let mut context = AbbreviationContext::new();
            context.start_abbreviation("Co".to_string(), 123);

            let debug_str = format!("{:?}", context);

            // Debug format should show the state
            assert!(debug_str.contains("true")); // in_abbreviation
            assert!(debug_str.contains("Co"));
            assert!(debug_str.contains("123"));
        }

        #[test]
        fn test_multiple_end_calls() {
            let mut context = AbbreviationContext::new();

            // Start abbreviation
            context.start_abbreviation("Inc".to_string(), 50);

            // End multiple times
            context.end_abbreviation();
            context.end_abbreviation(); // Should be safe to call multiple times

            assert!(!context.in_abbreviation);
            assert_eq!(context.current_abbreviation, None);
            assert_eq!(context.start_position, None);
        }
    }
}
