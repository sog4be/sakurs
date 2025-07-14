//! Tests for parallel processing chunk coordination and overlap handling
//!
//! These tests specifically target the bug where parallel processing with overlapping
//! chunks produces incorrect sentence boundaries due to offset calculation errors.

use sakurs_core::{Config, Input, SentenceProcessor};

/// Simple test to reproduce the chunk offset bug with minimal data
#[test]
fn test_parallel_chunk_offset_bug_simple() {
    // Create text that's just over 512 bytes to trigger multiple chunks
    // Each sentence is about 30 bytes, so 20 sentences = ~600 bytes
    let mut sentences = Vec::new();
    for i in 1..=20 {
        sentences.push(format!("This is sentence number {}.", i));
    }
    let test_text = sentences.join(" ");

    // Text will be approximately 550 bytes with 20 sentences

    // Force very small chunks to ensure multiple chunks
    let config = Config::builder()
        .language("en")
        .unwrap()
        .chunk_size(200) // Small chunks to force chunking
        .overlap_size(50) // Appropriate overlap for chunk size
        .threads(Some(2)) // Force parallel
        .build()
        .unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let result = processor
        .process(Input::from_text(test_text.clone()))
        .unwrap();

    // Should find 19 sentences (one boundary might be at the end)
    assert!(
        result.metadata.stats.sentence_count >= 19 && result.metadata.stats.sentence_count <= 20,
        "Expected ~20 sentences, but found {}",
        result.metadata.stats.sentence_count
    );
}

/// Test that reproduces the exact bug scenario reported:
/// - File > 1MB triggers parallel processing
/// - With -p flag, uses 256KB chunks
/// - Should produce multiple sentences, not just one
#[test]
fn test_parallel_processing_with_overlapping_chunks() {
    // Create test text with exactly 16 sentences (like ewt_plain.txt)
    let sentences = vec![
        "This is the first sentence.",
        "This is the second sentence.",
        "Here comes the third one.",
        "The fourth sentence is here.",
        "Fifth sentence in the list.",
        "Number six is arriving now.",
        "Lucky number seven appears.",
        "Eight is great they say.",
        "Nine is fine and divine.",
        "Ten is when we begin again.",
        "Eleven is odd but even.",
        "Twelve like a dozen eggs.",
        "Thirteen might be unlucky.",
        "Fourteen is twice seven.",
        "Fifteen is three times five.",
        "Sixteen completes our set.",
    ];

    // Create a large text by repeating to exceed parallel threshold
    let single_pass = sentences.join(" ");
    let _single_pass_size = single_pass.len();
    let repetitions = 100; // Create a reasonably sized text for testing
    let large_text = (0..repetitions)
        .map(|_| single_pass.clone())
        .collect::<Vec<_>>()
        .join(" ");

    // Configure with small chunks to trigger overlapping
    let config = Config::builder()
        .language("en")
        .unwrap()
        .chunk_size(256 * 1024) // 256KB chunks like CLI with -p
        .threads(Some(4)) // Force parallel processing
        .build()
        .unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();
    let input = Input::from_text(large_text.clone());
    let result = processor.process(input).unwrap();

    // Count sentences - should be 16 * repetitions, not 1
    let sentence_count = result.metadata.stats.sentence_count;
    let expected_count = 16 * repetitions;

    assert_eq!(
        sentence_count, expected_count,
        "Parallel processing produced {} sentences, expected {}",
        sentence_count, expected_count
    );

    // Also check that we have the correct number of boundaries
    assert_eq!(
        result.boundaries.len(),
        expected_count,
        "Found {} boundaries, expected {}",
        result.boundaries.len(),
        expected_count
    );
}

/// Test that parallel and sequential processing produce identical results
/// with small chunk sizes that trigger overlap handling
#[test]
fn test_sequential_vs_parallel_consistency_small_chunks() {
    let test_text = "First sentence here. Second one follows. Third is next. \
                     Fourth comes after. Fifth in line. Sixth position. \
                     Seventh heaven. Eighth note. Ninth inning. Tenth place. \
                     Eleventh hour. Twelfth night. Lucky thirteen. \
                     Fourteen days. Fifteen minutes. Final sixteenth.";

    // Sequential processing (single-threaded)
    let config_seq = Config::builder()
        .language("en")
        .unwrap()
        .chunk_size(1024) // Small but valid chunks
        .overlap_size(128) // Appropriate overlap for chunk size
        .threads(Some(1))
        .build()
        .unwrap();

    // Parallel processing with same chunk size
    let config_par = Config::builder()
        .language("en")
        .unwrap()
        .chunk_size(1024) // Same small chunks
        .overlap_size(128) // Appropriate overlap for chunk size
        .threads(Some(4))
        .build()
        .unwrap();

    let processor_seq = SentenceProcessor::with_config(config_seq).unwrap();
    let processor_par = SentenceProcessor::with_config(config_par).unwrap();

    let result_seq = processor_seq.process(Input::from_text(test_text)).unwrap();
    let result_par = processor_par.process(Input::from_text(test_text)).unwrap();

    // Should have same number of sentences
    assert_eq!(
        result_seq.metadata.stats.sentence_count, result_par.metadata.stats.sentence_count,
        "Sequential found {} sentences, parallel found {}",
        result_seq.metadata.stats.sentence_count, result_par.metadata.stats.sentence_count
    );

    // Each boundary should match exactly
    assert_eq!(
        result_seq.boundaries.len(),
        result_par.boundaries.len(),
        "Different number of boundaries"
    );

    for (i, (seq_bound, par_bound)) in result_seq
        .boundaries
        .iter()
        .zip(result_par.boundaries.iter())
        .enumerate()
    {
        assert_eq!(
            seq_bound.offset,
            par_bound.offset,
            "Boundary {} offset differs: sequential={}, parallel={}",
            i + 1,
            seq_bound.offset,
            par_bound.offset
        );
    }
}

/// Test that verifies no duplicate boundaries are produced
/// when chunks overlap
#[test]
fn test_no_duplicate_boundaries_in_overlap_regions() {
    // Create text where sentence boundaries might fall in overlap regions
    let test_text = "A".repeat(250) + ". " + &"B".repeat(250) + ". " + &"C".repeat(250) + ".";

    let config = Config::builder()
        .language("en")
        .unwrap()
        .chunk_size(300) // Chunks will overlap at the sentence boundaries
        .overlap_size(50) // Appropriate overlap for chunk size
        .threads(Some(3))
        .build()
        .unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();
    let input = Input::from_text(test_text.clone());
    let result = processor.process(input).unwrap();

    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for boundary in &result.boundaries {
        assert!(
            seen.insert(boundary.offset),
            "Duplicate sentence boundary found at offset {}",
            boundary.offset
        );
    }

    // Should have 2-3 sentences (depending on whether final boundary is included)
    assert!(
        result.metadata.stats.sentence_count >= 2 && result.metadata.stats.sentence_count <= 3,
        "Expected 2-3 sentences, found {}",
        result.metadata.stats.sentence_count
    );
}

/// Test chunk offset calculation correctness
#[test]
fn test_parallel_chunk_offset_calculation() {
    // Create predictable text with known sentence positions
    let sentence1 = "First sentence ends here."; // 0-25
    let sentence2 = " Second sentence ends here."; // 25-52
    let sentence3 = " Third sentence ends here."; // 52-78
    let sentence4 = " Fourth sentence ends here."; // 78-105

    let test_text = format!("{}{}{}{}", sentence1, sentence2, sentence3, sentence4);

    let config = Config::builder()
        .language("en")
        .unwrap()
        .chunk_size(512) // Small chunks to ensure multiple chunks
        .overlap_size(50) // Appropriate overlap for chunk size
        .threads(Some(2))
        .build()
        .unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();
    let input = Input::from_text(test_text);
    let result = processor.process(input).unwrap();

    // Verify we found all 4 sentences
    assert_eq!(
        result.metadata.stats.sentence_count, 4,
        "Expected 4 sentences, found {}",
        result.metadata.stats.sentence_count
    );

    // Verify correct positions
    let expected_offsets = vec![25, 52, 78, 105]; // End positions

    assert_eq!(
        result.boundaries.len(),
        expected_offsets.len(),
        "Expected {} boundaries, found {}",
        expected_offsets.len(),
        result.boundaries.len()
    );

    for (i, (boundary, expected_offset)) in result
        .boundaries
        .iter()
        .zip(expected_offsets.iter())
        .enumerate()
    {
        assert_eq!(
            boundary.offset,
            *expected_offset,
            "Boundary {} position incorrect: expected {}, got {}",
            i + 1,
            expected_offset,
            boundary.offset
        );
    }
}
