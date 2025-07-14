use sakurs_core::{Config, Input, SentenceProcessor};

#[test]
#[ignore = "TODO: Fix handling of very small chunk sizes (Issue #102)"]
fn test_bracket_pattern_detection() {
    // Test the specific ". [" pattern that was causing issues
    let test_cases = vec![
        (
            "First sentence. [Second sentence.] Third sentence.",
            vec![15, 50], // After first period and at end of text
            "Basic bracket pattern",
        ),
        (
            "On the Syrian border . [ This killing ] of a person.",
            vec![22, 52],
            "EWT-style pattern with spaces",
        ),
        (
            "Test one. (Another test.) Final test.",
            vec![9, 37], // Boundary after "one." and at end; inside parentheses is not a boundary
            "Parenthesis pattern",
        ),
        (
            "Quote test. \"This is quoted.\" Another sentence.",
            vec![11], // Boundary after "test." only; no period at end
            "Quote pattern with no final period",
        ),
    ];

    for (text, expected_offsets, description) in test_cases {
        println!("Testing: {}", description);

        // Test with default config (single-threaded)
        let proc = SentenceProcessor::new();
        let result = proc.process(Input::from_text(text)).unwrap();

        let offsets: Vec<usize> = result.boundaries.iter().map(|b| b.offset).collect();
        assert_eq!(
            offsets, expected_offsets,
            "Failed for '{}' with default config. Got {:?}, expected {:?}",
            description, offsets, expected_offsets
        );

        // Test with small chunks to force multiple chunks
        let config = Config::builder()
            .language("en")
            .unwrap()
            .chunk_size(20) // Very small chunks
            .overlap_size(5) // Small overlap for small chunks
            .threads(Some(2)) // Parallel processing
            .build()
            .unwrap();

        let proc = SentenceProcessor::with_config(config).unwrap();
        let result = proc.process(Input::from_text(text)).unwrap();

        let offsets: Vec<usize> = result.boundaries.iter().map(|b| b.offset).collect();
        assert_eq!(
            offsets, expected_offsets,
            "Failed for '{}' with parallel config. Got {:?}, expected {:?}",
            description, offsets, expected_offsets
        );
    }
}

#[test]
#[ignore = "TODO: Fix handling of very small chunk sizes (Issue #102)"]
fn test_ewt_pattern_with_various_chunk_sizes() {
    let text = "On the Syrian border . [ This killing ] was important. Another sentence here.";
    let expected_boundaries = vec![22, 54, 77]; // After ". [", after "important." and at end

    let chunk_sizes = vec![
        ("10 bytes", 10, 2), // chunk_size, overlap_size
        ("50 bytes", 50, 10),
        ("100 bytes", 100, 20),
        ("1KB", 1024, 256),
        ("256KB", 256 * 1024, 256),
    ];

    for (name, size, overlap) in chunk_sizes {
        println!("Testing with {} chunks", name);

        let config = Config::builder()
            .language("en")
            .unwrap()
            .chunk_size(size)
            .overlap_size(overlap)
            .build()
            .unwrap();

        let proc = SentenceProcessor::with_config(config).unwrap();
        let result = proc.process(Input::from_text(text.to_string())).unwrap();

        let offsets: Vec<usize> = result.boundaries.iter().map(|b| b.offset).collect();
        assert_eq!(
            offsets, expected_boundaries,
            "Failed with {} chunks. Got {:?}, expected {:?}",
            name, offsets, expected_boundaries
        );
    }
}
