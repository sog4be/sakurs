//! Integration tests for adaptive processing

use sakurs_core::domain::language::EnglishLanguageRules;
use sakurs_core::{AdaptiveProcessor, UnifiedProcessor};
use std::sync::Arc;

#[test]
fn test_adaptive_vs_unified_consistency() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let adaptive = AdaptiveProcessor::new(rules.clone());
    let unified = UnifiedProcessor::new(rules);

    let small_text = "This is a small test. It should work.";
    let medium_text = "This is a medium test. ".repeat(100);
    let large_text = "This is a large test. It has many sentences! Does it work? ".repeat(1000);

    let test_cases = vec![small_text, &medium_text, &large_text];

    for text in test_cases {
        let adaptive_result = adaptive.process(text).unwrap();
        let unified_result = unified.process(text).unwrap();
        let unified_boundaries: Vec<usize> = unified_result
            .boundaries
            .into_iter()
            .map(|b| b.offset)
            .collect();

        assert_eq!(
            adaptive_result,
            unified_boundaries,
            "Adaptive and unified processors should produce same results for text length {}",
            text.len()
        );
    }
}

#[test]
fn test_adaptive_performance_characteristics() {
    use std::time::Instant;

    let rules = Arc::new(EnglishLanguageRules::new());
    let adaptive = AdaptiveProcessor::new(rules);

    // Generate texts of different sizes
    let sizes = vec![
        (10, "tiny"),         // 10 bytes
        (1_000, "small"),     // 1KB
        (100_000, "medium"),  // 100KB
        (1_000_000, "large"), // 1MB
    ];

    for (size, label) in sizes {
        let base = "This is a test sentence. ";
        let repeat_count = size / base.len() + 1;
        let text = base.repeat(repeat_count);
        let text = &text[..size.min(text.len())];

        let start = Instant::now();
        let result = adaptive.process(text).unwrap();
        let elapsed = start.elapsed();

        println!(
            "{} text ({} bytes): {} boundaries found in {:?}",
            label,
            text.len(),
            result.len(),
            elapsed
        );

        // Tiny and small text may not have boundaries if they end mid-sentence
        if size >= 100 {
            assert!(
                !result.is_empty(),
                "Should find boundaries in {} text",
                label
            );
        }
    }
}

#[test]
fn test_adaptive_handles_edge_cases() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let adaptive = AdaptiveProcessor::new(rules);

    // Empty text
    let result = adaptive.process("").unwrap();
    assert!(result.is_empty());

    // Single character
    let result = adaptive.process("a").unwrap();
    assert!(result.is_empty());

    // Single sentence without period
    let result = adaptive.process("Hello world").unwrap();
    assert!(result.is_empty());

    // Single sentence with period
    let result = adaptive.process("Hello world.").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], 12);
}
