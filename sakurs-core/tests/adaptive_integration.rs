//! Integration tests for adaptive processing

use sakurs_core::domain::language::EnglishLanguageRules;
use sakurs_core::UnifiedProcessor;
use std::sync::Arc;

#[test]
fn test_adaptive_vs_unified_consistency() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let unified = UnifiedProcessor::new(rules);

    let small_text = "This is a small test. It should work.";
    let medium_text = "This is a medium test. ".repeat(100);
    let large_text = "This is a large test. It has many sentences! Does it work? ".repeat(1000);

    let test_cases = vec![small_text, &medium_text, &large_text];

    for text in test_cases {
        // Test both adaptive and regular processing methods
        let adaptive_result = unified.process_adaptive(text).unwrap();
        let unified_result = unified.process(text).unwrap();
        let unified_boundaries: Vec<usize> = unified_result
            .boundaries
            .into_iter()
            .map(|b| b.offset)
            .collect();

        assert_eq!(
            adaptive_result,
            unified_boundaries,
            "Adaptive and unified processing should produce identical results for text of length {}",
            text.len()
        );
    }
}

#[test]
fn test_adaptive_strategy_selection() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let processor = UnifiedProcessor::new(rules);

    // Small text should use sequential processing
    let small_text = "Small text.";
    let small_result = processor.process_adaptive(small_text);
    assert!(small_result.is_ok());

    // Large text should trigger parallel or streaming
    let large_text = "Large text. ".repeat(10000);
    let large_result = processor.process_adaptive(&large_text);
    assert!(large_result.is_ok());

    // Both should produce valid results
    assert!(!small_result.unwrap().is_empty());
    assert!(!large_result.unwrap().is_empty());
}
