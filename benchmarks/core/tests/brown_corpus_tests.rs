//! Comprehensive tests for Brown Corpus data loading

use sakurs_benchmarks::data::brown_corpus;
use sakurs_benchmarks::error::BenchmarkError;

#[test]
fn test_is_available_function() {
    // Just verify the function executes without panic
    let _available = brown_corpus::is_available();
}

#[test]
fn test_small_sample_validity() {
    let sample = brown_corpus::small_sample();

    // Verify basic properties
    assert_eq!(sample.name, "brown_corpus_sample");
    assert!(!sample.text.is_empty());
    assert!(!sample.boundaries.is_empty());

    // Verify the sample text contains expected content
    assert!(sample.text.contains("Fulton County Grand Jury"));
    assert!(sample.text.contains("Atlanta"));

    // Verify boundaries are valid
    for &boundary in &sample.boundaries {
        assert!(boundary < sample.text.len());
    }

    // Verify validation passes
    assert!(sample.validate().is_ok());
}

#[test]
fn test_load_full_corpus_error_handling() {
    // If corpus is not available, should return proper error
    if !brown_corpus::is_available() {
        match brown_corpus::load_full_corpus() {
            Err(BenchmarkError::CorpusNotFound { corpus_name, .. }) => {
                assert_eq!(corpus_name, "Brown Corpus");
            }
            Ok(_) => panic!("Expected CorpusNotFound error"),
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }
}

#[test]
fn test_load_subset_with_valid_corpus() {
    // Skip if corpus not available
    if !brown_corpus::is_available() {
        return;
    }

    // Test various subset sizes
    let sizes = vec![0, 1, 10, 100];

    for size in sizes {
        let result = brown_corpus::load_subset(size);
        assert!(
            result.is_ok(),
            "Failed to load subset of size {}: {:?}",
            size,
            result.err()
        );

        let subset = result.unwrap();

        // Verify name includes subset size
        assert!(subset.name.contains("subset") || subset.name.contains("empty"));

        // Verify boundary count
        if size == 0 {
            assert_eq!(subset.boundaries.len(), 0);
            assert_eq!(subset.text.len(), 0);
        } else {
            assert!(subset.boundaries.len() <= size);
            assert!(!subset.text.is_empty());
        }

        // Verify validation passes
        assert!(subset.validate().is_ok());
    }
}

#[test]
fn test_subset_consistency() {
    // Skip if corpus not available
    if !brown_corpus::is_available() {
        return;
    }

    // Load two overlapping subsets
    let subset_10 = brown_corpus::load_subset(10).unwrap();
    let subset_20 = brown_corpus::load_subset(20).unwrap();

    // The first 10 sentences should be identical
    if subset_10.boundaries.len() > 0 {
        let text_10 = &subset_10.text;
        let text_20_prefix = &subset_20.text[..text_10.len()];

        assert_eq!(text_10, text_20_prefix, "Subset text should be consistent");

        // Boundaries should also match
        for (i, &boundary) in subset_10.boundaries.iter().enumerate() {
            assert_eq!(
                boundary, subset_20.boundaries[i],
                "Boundary mismatch at index {}",
                i
            );
        }
    }
}

#[test]
fn test_corpus_metadata_if_available() {
    // Skip if corpus not available
    if !brown_corpus::is_available() {
        return;
    }

    let corpus = brown_corpus::load_full_corpus().unwrap();

    // Basic sanity checks for Brown Corpus
    assert!(
        corpus.text.len() > 1_000_000,
        "Brown Corpus should be > 1MB"
    );
    assert!(
        corpus.boundaries.len() > 10_000,
        "Brown Corpus should have > 10k sentences"
    );

    // Verify boundaries are strictly increasing
    let mut prev = 0;
    for &boundary in &corpus.boundaries {
        assert!(boundary > prev, "Boundaries must be strictly increasing");
        prev = boundary;
    }
}
