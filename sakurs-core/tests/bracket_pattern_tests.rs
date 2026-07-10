use sakurs_core::{Config, Input, SentenceProcessor};

#[test]
fn test_bracket_pattern_detection() {
    // Test the specific ". [" pattern that was causing issues
    let test_cases = vec![
        (
            "First sentence. [Second sentence.] Third sentence.",
            // After the first period, after the closing bracket
            // (boundary-after-closers), and at the end of text.
            vec![15, 34, 50],
            "Basic bracket pattern",
        ),
        (
            "On the Syrian border . [ This killing ] of a person.",
            vec![22, 52],
            "EWT-style pattern with spaces",
        ),
        (
            "Test one. (Another test.) Final test.",
            // After "one.", after the closing paren (boundary-after-closers,
            // next word capitalized), and at the end of text.
            vec![9, 25, 37],
            "Parenthesis pattern",
        ),
        (
            "Quote test. \"This is quoted.\" Another sentence.",
            // After "test.", after the closing quote (boundary-after-closers),
            // and at the final period.
            vec![11, 29, 47],
            "Quote pattern",
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
