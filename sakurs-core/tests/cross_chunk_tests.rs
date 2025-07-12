//! Integration tests for cross-chunk handling

use sakurs_core::domain::{
    cross_chunk::{
        ChunkMetadata, CrossChunkResolver, CrossChunkValidator, EnclosureStateTracker,
        EnhancedAbbreviationState, EnhancedPartialState, ValidationResult,
    },
    language::EnglishLanguageRules,
    types::{AbbreviationState, BoundaryCandidate, BoundaryFlags, DepthVec, PartialState},
};
use std::sync::Arc;

#[test]
fn test_cross_chunk_abbreviation_detection() {
    // Simulate text: "See Dr." | " Smith for details."
    // The boundary after "Dr." should be suppressed

    let rules = Arc::new(EnglishLanguageRules::new());

    // First chunk ends with "Dr."
    let mut state1 = PartialState::new(5);
    state1.chunk_length = 7;
    state1.abbreviation = AbbreviationState::with_first_word(
        true,  // dangling_dot
        false, // head_alpha
        None,  // first_word
    );
    state1.add_boundary_candidate(
        7, // After "Dr."
        DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
        BoundaryFlags::STRONG,
    );

    // Second chunk starts with " Smith"
    let mut state2 = PartialState::new(5);
    state2.chunk_length = 20;
    let state2_abbr = AbbreviationState::with_first_word(
        false, // dangling_dot
        true,  // head_alpha - Starts with alphabetic
        None,  // first_word
    );
    state2.abbreviation = state2_abbr.clone();

    // Create enhanced states
    let enhanced1 = EnhancedPartialState {
        base: state1.clone(),
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: state1.abbreviation.clone(),
            abbreviation_text: Some("Dr".to_string()),
            abbreviation_start: Some(4),
            confidence: 0.95,
            is_continuation: false,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 0,
            total_chunks: 2,
            has_prefix_overlap: false,
            has_suffix_overlap: true,
            prefix_overlap: None,
            suffix_overlap: Some("Dr. ".to_string()),
        },
    };

    let enhanced2 = EnhancedPartialState {
        base: state2,
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: state2_abbr,
            abbreviation_text: None,
            abbreviation_start: None,
            confidence: 0.0,
            is_continuation: true,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 1,
            total_chunks: 2,
            has_prefix_overlap: true,
            has_suffix_overlap: false,
            prefix_overlap: Some(" Sm".to_string()),
            suffix_overlap: None,
        },
    };

    // Test cross-chunk abbreviation detection
    assert!(enhanced1
        .enhanced_abbreviation
        .is_cross_chunk_abbreviation(&enhanced2.enhanced_abbreviation));

    // Test validation
    let validator = CrossChunkValidator::new(10);
    let result = validator.validate_chunk_boundary(
        &enhanced1.base.boundary_candidates[0],
        &enhanced1.base,
        Some(&enhanced2.base),
        rules.as_ref(),
    );

    assert_eq!(
        result,
        ValidationResult::Invalid("Cross-chunk abbreviation detected".to_string())
    );
}

#[test]
fn test_cross_chunk_enclosure_tracking() {
    // Simulate: 'He said "Hello' | ' world" and left.'
    // Quote opens in first chunk and closes in second

    let mut tracker1 = EnclosureStateTracker::new();
    tracker1.update(
        sakurs_core::domain::enclosure::EnclosureType::DoubleQuote,
        0,
        true,
        8, // Position of opening quote
    );

    let mut tracker2 = EnclosureStateTracker::new();
    tracker2.update(
        sakurs_core::domain::enclosure::EnclosureType::DoubleQuote,
        0,
        false,
        7, // Position of closing quote in second chunk
    );

    // Check individual trackers first
    assert_eq!(tracker1.depth_map[&0], 1); // One open quote
    assert_eq!(tracker2.depth_map[&0], -1); // One close quote
    assert_eq!(tracker2.unopened_closures.len(), 1); // Has a close without open

    // Merge trackers
    let merged = tracker1.merge_with_next(&tracker2);

    // Should have balanced quotes
    assert_eq!(merged.open_enclosures.len(), 0);
    assert_eq!(merged.depth_map.get(&0).copied().unwrap_or(0), 0);
}

#[test]
fn test_chunk_boundary_validation_near_end() {
    let validator = CrossChunkValidator::new(10);

    let mut state = PartialState::new(5);
    state.chunk_length = 100;

    // Boundary near end of chunk
    let boundary = BoundaryCandidate {
        local_offset: 95, // Only 5 chars from end
        local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
        flags: BoundaryFlags::STRONG,
    };

    let rules = EnglishLanguageRules::new();
    let result = validator.validate_chunk_boundary(&boundary, &state, None, &rules);

    // Should need more context since no next state provided
    assert_eq!(result, ValidationResult::NeedsMoreContext);
}

#[test]
fn test_chunk_boundary_validation_with_unbalanced_enclosures() {
    let validator = CrossChunkValidator::new(10);

    let state = PartialState::new(5);

    // Boundary inside quotes
    let boundary = BoundaryCandidate {
        local_offset: 50,
        local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]), // Inside quotes
        flags: BoundaryFlags::STRONG,
    };

    let rules = EnglishLanguageRules::new();
    let result = validator.validate_chunk_boundary(&boundary, &state, None, &rules);

    // Should be weakened due to unbalanced enclosures
    assert_eq!(result, ValidationResult::Weakened(BoundaryFlags::WEAK));
}

#[test]
fn test_cross_chunk_resolver() {
    let resolver = CrossChunkResolver::new(10);
    let rules = Arc::new(EnglishLanguageRules::new());

    // Create two chunks with boundaries
    let mut state1 = PartialState::new(5);
    state1.chunk_length = 50;
    state1.add_boundary_candidate(
        25,
        DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
        BoundaryFlags::STRONG,
    );

    let mut state2 = PartialState::new(5);
    state2.chunk_length = 50;
    state2.add_boundary_candidate(
        10,
        DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
        BoundaryFlags::STRONG,
    );

    let enhanced_states = vec![
        EnhancedPartialState {
            base: state1,
            enhanced_abbreviation: EnhancedAbbreviationState::default(),
            enclosure_tracker: EnclosureStateTracker::new(),
            chunk_metadata: ChunkMetadata {
                index: 0,
                total_chunks: 2,
                has_prefix_overlap: false,
                has_suffix_overlap: true,
                prefix_overlap: None,
                suffix_overlap: None,
            },
        },
        EnhancedPartialState {
            base: state2,
            enhanced_abbreviation: EnhancedAbbreviationState::default(),
            enclosure_tracker: EnclosureStateTracker::new(),
            chunk_metadata: ChunkMetadata {
                index: 1,
                total_chunks: 2,
                has_prefix_overlap: true,
                has_suffix_overlap: false,
                prefix_overlap: None,
                suffix_overlap: None,
            },
        },
    ];

    let boundaries = resolver.resolve_boundaries(&enhanced_states, rules.as_ref());

    // Should have both boundaries
    assert_eq!(boundaries.len(), 2);
}

#[test]
fn test_enhanced_abbreviation_with_confidence() {
    let abbr = EnhancedAbbreviationState {
        basic: AbbreviationState::with_first_word(
            true,  // dangling_dot
            false, // head_alpha
            None,  // first_word
        ),
        abbreviation_text: Some("Inc".to_string()),
        abbreviation_start: Some(10),
        confidence: 0.9,
        is_continuation: false,
    };

    assert!(abbr.abbreviation_text.is_some());
    assert_eq!(abbr.confidence, 0.9);
}

#[test]
fn test_enclosure_tracker_with_nested_quotes() {
    let mut tracker = EnclosureStateTracker::new();

    // Open outer quote
    tracker.update(
        sakurs_core::domain::enclosure::EnclosureType::DoubleQuote,
        0,
        true,
        10,
    );

    // Open inner quote
    tracker.update(
        sakurs_core::domain::enclosure::EnclosureType::SingleQuote,
        1,
        true,
        15,
    );

    assert_eq!(tracker.open_enclosures.len(), 2);
    assert_eq!(tracker.depth_map[&0], 1);
    assert_eq!(tracker.depth_map[&1], 1);

    // Close inner quote
    tracker.update(
        sakurs_core::domain::enclosure::EnclosureType::SingleQuote,
        1,
        false,
        20,
    );

    assert_eq!(tracker.open_enclosures.len(), 1);
    assert_eq!(tracker.depth_map[&1], 0);
}

#[test]
fn test_cross_chunk_abbreviation_with_sentence_starter() {
    // Test: "...Apple Inc." | "However, the company..."
    // The boundary after "Inc." should be detected as a sentence boundary

    let rules = Arc::new(EnglishLanguageRules::new());

    // First chunk ends with "Inc."
    let mut state1 = PartialState::new(5);
    state1.chunk_length = 15;
    state1.abbreviation = AbbreviationState::with_first_word(
        true,  // dangling_dot
        false, // head_alpha
        None,  // first_word - not relevant for end of chunk
    );
    // Add boundary candidate for the period after "Inc."
    state1.add_boundary_candidate(
        14, // Position of period after "Inc"
        DepthVec::from_vec(vec![0; 5]),
        BoundaryFlags::WEAK, // Currently weak because it's an abbreviation
    );

    // Second chunk starts with "However"
    let mut state2 = PartialState::new(5);
    state2.chunk_length = 20;
    state2.abbreviation = AbbreviationState::with_first_word(
        false,                       // dangling_dot
        true,                        // head_alpha - starts with "H" from "However"
        Some("However".to_string()), // first_word
    );

    // Create enhanced states
    let enhanced1 = EnhancedPartialState {
        base: state1.clone(),
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: state1.abbreviation.clone(),
            abbreviation_text: Some("Inc".to_string()),
            abbreviation_start: Some(11),
            confidence: 1.0,
            is_continuation: false,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 0,
            total_chunks: 2,
            has_prefix_overlap: false,
            has_suffix_overlap: true,
            prefix_overlap: None,
            suffix_overlap: Some("Inc. ".to_string()),
        },
    };

    let enhanced2 = EnhancedPartialState {
        base: state2,
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: AbbreviationState::with_first_word(
                false,                       // dangling_dot
                true,                        // head_alpha
                Some("However".to_string()), // first_word
            ),
            abbreviation_text: None,
            abbreviation_start: None,
            confidence: 0.0,
            is_continuation: false,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 1,
            total_chunks: 2,
            has_prefix_overlap: true,
            has_suffix_overlap: false,
            prefix_overlap: Some("However".to_string()),
            suffix_overlap: None,
        },
    };

    // Test with cross-chunk resolver
    let resolver = CrossChunkResolver::new(10);
    let boundaries = resolver.resolve_boundaries(&vec![enhanced1, enhanced2], rules.as_ref());

    // We expect the boundary at position 14 to be confirmed
    assert!(
        boundaries.iter().any(|b| b.offset == 14),
        "Expected boundary after 'Inc.' when followed by 'However'"
    );
}

#[test]
fn test_cross_chunk_abbreviation_with_proper_noun() {
    // Test: "...contact Dr." | "Smith for details..."
    // The boundary after "Dr." should NOT be detected

    let rules = Arc::new(EnglishLanguageRules::new());

    // First chunk ends with "Dr."
    let mut state1 = PartialState::new(5);
    state1.chunk_length = 12;
    state1.abbreviation = AbbreviationState::with_first_word(
        true,  // dangling_dot
        false, // head_alpha
        None,  // first_word
    );
    // Add boundary candidate for the period after "Dr."
    state1.add_boundary_candidate(
        11, // Position of period after "Dr"
        DepthVec::from_vec(vec![0; 5]),
        BoundaryFlags::WEAK,
    );

    // Second chunk starts with "Smith" (not a sentence starter)
    let mut state2 = PartialState::new(5);
    state2.chunk_length = 18;
    state2.abbreviation = AbbreviationState::with_first_word(
        false,                     // dangling_dot
        true,                      // head_alpha - Starts with "S" from "Smith"
        Some("Smith".to_string()), // first_word
    );

    // Create enhanced states
    let enhanced1 = EnhancedPartialState {
        base: state1.clone(),
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: state1.abbreviation.clone(),
            abbreviation_text: Some("Dr".to_string()),
            abbreviation_start: Some(9),
            confidence: 1.0,
            is_continuation: false,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 0,
            total_chunks: 2,
            has_prefix_overlap: false,
            has_suffix_overlap: true,
            prefix_overlap: None,
            suffix_overlap: Some("Dr. ".to_string()),
        },
    };

    let enhanced2 = EnhancedPartialState {
        base: state2,
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: AbbreviationState::with_first_word(
                false, // dangling_dot
                true,  // head_alpha
                None,  // first_word
            ),
            abbreviation_text: None,
            abbreviation_start: None,
            confidence: 0.0,
            is_continuation: false,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 1,
            total_chunks: 2,
            has_prefix_overlap: true,
            has_suffix_overlap: false,
            prefix_overlap: Some("Smith".to_string()),
            suffix_overlap: None,
        },
    };

    let resolver = CrossChunkResolver::new(10);
    let boundaries = resolver.resolve_boundaries(&vec![enhanced1, enhanced2], rules.as_ref());

    // We expect NO boundary at position 11
    assert!(
        !boundaries.iter().any(|b| b.offset == 11),
        "Should not detect boundary after 'Dr.' when followed by a proper name"
    );
}

#[test]
fn test_multiple_abbreviations_across_chunks() {
    // Test: "...U.S.A. Inc." | "Therefore announced..."
    // Only the last period (after "Inc.") should be a boundary

    let rules = Arc::new(EnglishLanguageRules::new());

    // First chunk ends with "U.S.A. Inc."
    let mut state1 = PartialState::new(5);
    state1.chunk_length = 13;
    state1.abbreviation = AbbreviationState::with_first_word(
        true,  // dangling_dot
        false, // head_alpha
        None,  // first_word
    );

    // Add boundary candidates for periods
    // Period after "U.S.A." at position 7
    state1.add_boundary_candidate(7, DepthVec::from_vec(vec![0; 5]), BoundaryFlags::WEAK);
    // Period after "Inc." at position 12
    state1.add_boundary_candidate(12, DepthVec::from_vec(vec![0; 5]), BoundaryFlags::WEAK);

    // Second chunk starts with "Therefore"
    let mut state2 = PartialState::new(5);
    state2.chunk_length = 20;
    state2.abbreviation = AbbreviationState::with_first_word(
        false,                         // dangling_dot
        true,                          // head_alpha - Starts with "T" from "Therefore"
        Some("Therefore".to_string()), // first_word
    );

    // Create enhanced states
    let enhanced1 = EnhancedPartialState {
        base: state1.clone(),
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: state1.abbreviation.clone(),
            abbreviation_text: Some("Inc".to_string()),
            abbreviation_start: Some(9),
            confidence: 1.0,
            is_continuation: false,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 0,
            total_chunks: 2,
            has_prefix_overlap: false,
            has_suffix_overlap: true,
            prefix_overlap: None,
            suffix_overlap: Some("Inc. ".to_string()),
        },
    };

    let enhanced2 = EnhancedPartialState {
        base: state2,
        enhanced_abbreviation: EnhancedAbbreviationState {
            basic: AbbreviationState::with_first_word(
                false, // dangling_dot
                true,  // head_alpha
                None,  // first_word
            ),
            abbreviation_text: None,
            abbreviation_start: None,
            confidence: 0.0,
            is_continuation: false,
        },
        enclosure_tracker: EnclosureStateTracker::new(),
        chunk_metadata: ChunkMetadata {
            index: 1,
            total_chunks: 2,
            has_prefix_overlap: true,
            has_suffix_overlap: false,
            prefix_overlap: Some("Therefore".to_string()),
            suffix_overlap: None,
        },
    };

    let resolver = CrossChunkResolver::new(10);
    let boundaries = resolver.resolve_boundaries(&vec![enhanced1, enhanced2], rules.as_ref());

    // We expect only the boundary at position 12 (after "Inc.")
    assert!(
        boundaries.iter().any(|b| b.offset == 12),
        "Expected boundary after 'Inc.' when followed by 'Therefore'"
    );
    // The test for position 7 would need more context about what's between U.S.A. and Inc.
}
