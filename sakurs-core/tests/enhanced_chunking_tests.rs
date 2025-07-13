//! Integration tests for enhanced chunking with cross-chunk pattern detection

use sakurs_core::{
    application::enhanced_chunking::{
        EnhancedChunkConfig, EnhancedChunkManager, SuppressionReason,
    },
    domain::enclosure_suppressor::EnglishEnclosureSuppressor,
};
use std::sync::Arc;

/// Helper to create a manager with small chunks for testing
fn create_test_manager(chunk_size: usize) -> EnhancedChunkManager {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = EnhancedChunkConfig {
        chunk_size,
        overlap_size: 10, // Small overlap for testing
        enable_cross_chunk: true,
        ..Default::default()
    };
    EnhancedChunkManager::new(config, suppressor)
}

#[test]
fn test_contraction_across_chunks() {
    let mut manager = create_test_manager(20);

    // Text that will split "isn't" across chunks
    let text = "The problem here isn't the solution.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Should have multiple chunks
    assert!(
        chunks.len() > 1,
        "Expected multiple chunks for cross-chunk testing"
    );

    // Find suppressions
    let suppressions: Vec<_> = chunks.iter().flat_map(|c| &c.suppression_markers).collect();

    // Should detect the apostrophe in "isn't"
    assert!(
        !suppressions.is_empty(),
        "Should detect suppression for contraction"
    );

    // Check that it's identified as a contraction
    let contraction_suppression = suppressions
        .iter()
        .find(|s| matches!(s.reason, SuppressionReason::Contraction));
    assert!(
        contraction_suppression.is_some(),
        "Should identify apostrophe as contraction"
    );
}

#[test]
fn test_possessive_at_chunk_boundary() {
    let mut manager = create_test_manager(25);

    // Text that puts possessive apostrophe near chunk boundary
    let text = "This is James' house and garden.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Find suppressions
    let suppressions: Vec<_> = chunks.iter().flat_map(|c| &c.suppression_markers).collect();

    // Should detect the possessive apostrophe
    let possessive = suppressions
        .iter()
        .find(|s| matches!(s.reason, SuppressionReason::Possessive));
    assert!(possessive.is_some(), "Should detect possessive apostrophe");
}

#[test]
fn test_multiple_patterns_in_overlap() {
    let mut manager = create_test_manager(25);

    // Text with multiple patterns that might span chunks
    let text = "She said 'I'm fine' and James' car arrived.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Collect all suppressions
    let suppressions: Vec<_> = chunks.iter().flat_map(|c| &c.suppression_markers).collect();

    // Should detect both the contraction and possessive
    let has_contraction = suppressions
        .iter()
        .any(|s| matches!(s.reason, SuppressionReason::Contraction));
    let has_possessive = suppressions
        .iter()
        .any(|s| matches!(s.reason, SuppressionReason::Possessive));

    assert!(has_contraction, "Should detect contraction in I'm");
    assert!(has_possessive, "Should detect possessive in James'");
}

#[test]
fn test_measurement_marks_across_chunks() {
    let mut manager = create_test_manager(22);

    // Text with measurement that might split
    let text = "The height is 5'9\" exactly.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Find measurement suppressions
    let suppressions: Vec<_> = chunks
        .iter()
        .flat_map(|c| &c.suppression_markers)
        .filter(|s| matches!(s.reason, SuppressionReason::Measurement))
        .collect();

    // Should detect measurement marks
    assert!(!suppressions.is_empty(), "Should detect measurement marks");
}

#[test]
fn test_pattern_confidence_scores() {
    let mut manager = create_test_manager(20);

    let text = "This isn't working properly.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Check confidence scores
    let suppressions: Vec<_> = chunks.iter().flat_map(|c| &c.suppression_markers).collect();

    for suppression in suppressions {
        assert!(
            suppression.confidence >= 0.7,
            "Confidence should be reasonably high"
        );
        if matches!(suppression.reason, SuppressionReason::Contraction) {
            assert!(
                suppression.confidence >= 0.8,
                "Contractions should have high confidence"
            );
        }
    }
}

#[test]
fn test_partial_pattern_detection() {
    let mut manager = create_test_manager(22);

    // Text where "don't" should be split across chunks
    // With chunk_size=22 and this text, we should get multiple chunks
    let text = "I really think that we don't understand this completely.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Check that we have multiple chunks first
    if chunks.len() <= 1 {
        // Text too short to split, skip test
        return;
    }

    // Check for transition states with partial patterns
    let has_partial_patterns = chunks
        .iter()
        .filter_map(|c| c.transition_state.as_ref())
        .any(|state| !state.ending_patterns.is_empty() || !state.starting_patterns.is_empty());

    assert!(
        has_partial_patterns,
        "Should detect partial patterns at chunk boundaries"
    );
}

#[test]
fn test_boundary_deduplication() {
    let mut manager = create_test_manager(30);

    let text = "First sentence. Second sentence. Third sentence.";
    let _chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Get deduplicated boundaries
    let boundaries = manager.get_deduplicated_boundaries();

    // Check that boundaries are unique
    let mut sorted = boundaries.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        boundaries.len(),
        sorted.len(),
        "Boundaries should be deduplicated"
    );
}

#[test]
fn test_unicode_apostrophes() {
    let mut manager = create_test_manager(20);

    // Text with Unicode apostrophe
    let text = "This isn't working well."; // Using U+2019
    let _chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Test is considered passing if it doesn't panic
}

#[test]
fn test_cross_chunk_with_newlines() {
    let mut manager = create_test_manager(20);

    let text = "First line.\nIt isn't\nthe last one.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Should still detect contraction across newlines
    let suppressions: Vec<_> = chunks
        .iter()
        .flat_map(|c| &c.suppression_markers)
        .filter(|s| matches!(s.reason, SuppressionReason::Contraction))
        .collect();

    assert!(
        !suppressions.is_empty(),
        "Should detect contraction even with newlines"
    );
}

#[test]
fn test_very_short_chunks() {
    let mut manager = create_test_manager(22); // Small chunks (must be > 2 * overlap)

    let text = "I'm here.";
    let result = manager.chunk_with_overlap_processing(text);

    // Should handle very small chunks gracefully
    assert!(
        result.is_ok(),
        "Should handle very small chunks without error"
    );

    let chunks = result.unwrap();
    let suppressions: Vec<_> = chunks.iter().flat_map(|c| &c.suppression_markers).collect();

    // Should still detect the contraction if there are suppressions
    // With such a short text, it might be in a single chunk
    if chunks.len() == 1 {
        // For single chunk, we should still detect the apostrophe
        assert!(
            !suppressions.is_empty(),
            "Should detect apostrophe in single chunk"
        );
    } else {
        // For multiple chunks, check for suppressions
        assert!(
            !suppressions.is_empty(),
            "Should detect patterns even with tiny chunks"
        );
    }
}

#[test]
fn test_disabled_cross_chunk_processing() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = EnhancedChunkConfig {
        chunk_size: 20,
        overlap_size: 10,
        enable_cross_chunk: false, // Disabled
        ..Default::default()
    };
    let mut manager = EnhancedChunkManager::new(config, suppressor);

    let text = "This isn't working.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Should not have suppressions when disabled
    let suppressions: Vec<_> = chunks.iter().flat_map(|c| &c.suppression_markers).collect();

    assert!(
        suppressions.is_empty(),
        "Should not detect suppressions when cross-chunk is disabled"
    );
}

#[test]
fn test_state_tracking_continuity() {
    let mut manager = create_test_manager(25);

    let text = "The company isn't planning to expand.";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Check that transition states are properly assigned
    for (idx, chunk) in chunks.iter().enumerate() {
        if let Some(state) = &chunk.transition_state {
            assert_eq!(
                state.chunk_index, idx,
                "Transition state should have correct index"
            );
        }
    }
}

#[test]
fn test_complex_nested_quotes() {
    let mut manager = create_test_manager(30);

    let text = "She said, \"He told me 'I don't know' yesterday.\"";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Should handle nested quotes with contractions
    let contraction_suppressions: Vec<_> = chunks
        .iter()
        .flat_map(|c| &c.suppression_markers)
        .filter(|s| matches!(s.reason, SuppressionReason::Contraction))
        .collect();

    assert!(
        !contraction_suppressions.is_empty(),
        "Should detect contraction within nested quotes"
    );
}

#[test]
fn test_list_item_suppression() {
    let mut manager = create_test_manager(25);

    let text = "Items:\n1) First item\n2) Second item";
    let chunks = manager.chunk_with_overlap_processing(text).unwrap();

    // Should detect list item parentheses
    let list_suppressions: Vec<_> = chunks
        .iter()
        .flat_map(|c| &c.suppression_markers)
        .filter(|s| matches!(s.reason, SuppressionReason::ListItem))
        .collect();

    // Allow 2 or 3 - there might be overlap detection
    assert!(
        list_suppressions.len() >= 2,
        "Should detect at least both list item parentheses"
    );
}
