//! State representation for the Delta-Stack algorithm
//!
//! This module implements the core state representation ⟨B, Δ, A⟩ where:
//! - B: Boundary set with detected sentence boundaries
//! - Δ: Delta stack tracking enclosure states
//! - A: Abbreviation state for cross-chunk handling

// Re-exported from types module

// All type definitions have been moved to types.rs

#[cfg(test)]
mod tests {
    use crate::domain::types::*;
    use crate::domain::Monoid;

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
    fn test_language_rules_integration() {
        use crate::domain::language::MockLanguageRules;

        let rules = MockLanguageRules::english();

        // Test text with abbreviation that should not be a sentence boundary
        let text = "Dr. Smith is here. This is a test.";
        // Use application parser for testing
        use crate::application::parser::{ParseStrategy, ParsingInput, SequentialParser};
        let parser = SequentialParser::new();
        let result = parser.parse(ParsingInput::Text(text), &rules).unwrap();
        let state = match result {
            crate::application::parser::ParsingOutput::State(s) => *s,
            _ => panic!("Expected single state"),
        };

        // scan_chunk records ALL candidates - the reduce phase will filter them
        // So we should have candidates at all period positions
        let boundary_positions: Vec<usize> = state
            .boundary_candidates
            .iter()
            .map(|b| b.local_offset)
            .collect();

        // The scan phase records candidates with language rule decisions
        // If language rules mark "Dr." as NotBoundary, it won't be recorded
        // Only real boundaries marked as Boundary will be recorded
        assert_eq!(boundary_positions.len(), 2); // Should have 2 boundaries
        assert!(boundary_positions.contains(&18)); // After "here."
        assert!(boundary_positions.contains(&34)); // After "test."
    }
}
