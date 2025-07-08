//! Tests for boundary value objects

use crate::domain::types::boundary::*;

#[cfg(test)]
mod confirmed_boundary_tests {
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
        let boundary = ConfirmedBoundary::new(100, 0.123456789);

        // Confidence should be stored with f32 precision
        assert!((boundary.confidence - 0.123456789f32).abs() < 0.0000001);
    }
}

#[cfg(test)]
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
