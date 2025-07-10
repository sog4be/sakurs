/// Tests for domain types

#[cfg(test)]
mod boundary_tests {
    use super::super::*;

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
        let boundary = ConfirmedBoundary::new(100, 0.123456789);

        // Confidence should be stored with f32 precision
        assert!((boundary.confidence - 0.123456789f32).abs() < 0.0000001);
    }
}

#[cfg(test)]
mod state_tests {
    use super::super::*;

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

#[cfg(test)]
mod abbreviation_context_tests {
    use super::super::*;

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