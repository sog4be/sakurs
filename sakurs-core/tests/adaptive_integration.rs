//! Integration tests for adaptive processing

use sakurs_core::api::{Input, SentenceProcessor};

#[test]
fn test_adaptive_vs_explicit_thread_consistency() {
    let small_text = "This is a small test. It should work.";
    let medium_text = "This is a medium test. ".repeat(100);
    let large_text = "This is a large test. It has many sentences! Does it work? ".repeat(1000);

    let test_cases = vec![small_text, &medium_text, &large_text];

    for text in test_cases {
        // Test adaptive (default) processing
        let adaptive_processor = SentenceProcessor::new();
        let adaptive_result = adaptive_processor.process(Input::from_text(text)).unwrap();

        // Test explicit sequential processing
        let sequential_processor = SentenceProcessor::with_config(
            sakurs_core::api::Config::builder()
                .language("en")
                .unwrap()
                .threads(Some(1))
                .build()
                .unwrap(),
        )
        .unwrap();
        let sequential_result = sequential_processor
            .process(Input::from_text(text))
            .unwrap();

        // Compare boundary positions
        let adaptive_boundaries: Vec<usize> = adaptive_result
            .boundaries
            .iter()
            .map(|b| b.offset)
            .collect();
        let sequential_boundaries: Vec<usize> = sequential_result
            .boundaries
            .iter()
            .map(|b| b.offset)
            .collect();

        assert_eq!(
            adaptive_boundaries,
            sequential_boundaries,
            "Adaptive and sequential processing should produce identical results for text of length {}",
            text.len()
        );
    }
}

#[test]
fn test_adaptive_strategy_selection() {
    let processor = SentenceProcessor::new();

    // Small text should use sequential processing
    let small_text = "Small text.";
    let small_result = processor.process(Input::from_text(small_text));
    assert!(small_result.is_ok());

    // Large text should trigger parallel processing
    let large_text = "Large text. ".repeat(10000);
    let large_result = processor.process(Input::from_text(&large_text));
    assert!(large_result.is_ok());

    // Both should produce valid results
    assert!(!small_result.unwrap().boundaries.is_empty());
    assert!(!large_result.unwrap().boundaries.is_empty());
}
