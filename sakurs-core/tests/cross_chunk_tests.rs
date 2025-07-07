//! Integration tests for cross-chunk handling

use sakurs_core::domain::{
    cross_chunk::{
        ChunkMetadata, CrossChunkResolver, CrossChunkValidator, EnclosureStateTracker,
        EnhancedAbbreviationState, EnhancedPartialState, ValidationResult,
    },
    language::EnglishLanguageRules,
    state::{AbbreviationState, BoundaryCandidate, PartialState},
    types::DepthVec,
    BoundaryFlags,
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
    state1.abbreviation = AbbreviationState {
        dangling_dot: true,
        head_alpha: false,
    };
    state1.add_boundary_candidate(
        7, // After "Dr."
        DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
        BoundaryFlags::STRONG,
    );

    // Second chunk starts with " Smith"
    let mut state2 = PartialState::new(5);
    state2.chunk_length = 20;
    let state2_abbr = AbbreviationState {
        dangling_dot: false,
        head_alpha: true, // Starts with alphabetic
    };
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
        basic: AbbreviationState {
            dangling_dot: true,
            head_alpha: false,
        },
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
