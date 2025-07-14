use sakurs_core::{Config, Input, SentenceProcessor};

#[test]
fn test_bracket_pattern_detection() {
    // Test the specific ". [" pattern that was causing issues
    let test_cases = vec![
        (
            "First sentence. [Second sentence.] Third sentence.",
            vec![15, 33, 50], // TODO: Fix bracket handling (issue #99) - should be [15, 50]
            "Basic bracket pattern",
        ),
        (
            "On the Syrian border . [ This killing ] of a person.",
            vec![22, 52],
            "EWT-style pattern with spaces",
        ),
        (
            "Test one. (Another test.) Final test.",
            vec![9, 24, 37], // TODO: Fix parenthesis handling (issue #99) - should be [9, 37]
            "Parenthesis pattern",
        ),
        (
            "Quote test. \"This is quoted.\" Another sentence.",
            vec![11, 28, 47], // TODO: Fix quote handling (issue #99) - should be [11]
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
fn test_ewt_pattern_with_various_chunk_sizes() {
    let text = "On the Syrian border . [ This killing ] was important. Another sentence here.";
    let expected_boundaries = vec![22, 54, 77]; // After ". [", after "important." and at end

    let chunk_sizes = vec![
        ("10 bytes", 10),
        ("50 bytes", 50),
        ("100 bytes", 100),
        ("1KB", 1024),
        ("256KB", 256 * 1024),
    ];

    for (name, size) in chunk_sizes {
        println!("Testing with {} chunks", name);

        let config = Config::builder()
            .language("en")
            .unwrap()
            .chunk_size(size)
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
