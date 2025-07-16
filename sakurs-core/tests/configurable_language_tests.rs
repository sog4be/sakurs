//! Integration tests for configurable language rules
//!
//! Note: Some tests are currently disabled as they need to be updated
//! for the new configurable language rules system behavior.

// Re-enabling specific tests with analysis of correct expected values
use sakurs_core::{Input, SentenceProcessor};

#[test]
fn test_english_configurable_basic() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "Hello world. This is a test.";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 2);
    assert_eq!(result.boundaries[0].offset, 12); // After "Hello world."
    assert_eq!(result.boundaries[1].offset, 28); // After "This is a test."
}

#[test]
fn test_english_abbreviations() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "Dr. Smith works at Apple Inc. and lives on Main St. in the city.";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should recognize abbreviations and not split
    assert_eq!(result.boundaries.len(), 1);
    assert_eq!(result.boundaries[0].offset, 64); // Only after the final period
}

#[test]
fn test_english_ellipsis_handling() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Test ellipsis followed by capital letter (should be boundary)
    let text = "Wait... Then he left.";
    let result = processor.process(Input::from_text(text)).unwrap();
    assert_eq!(result.boundaries.len(), 2);

    // Test ellipsis followed by lowercase (should not be boundary)
    let text2 = "Wait... then he left.";
    let result2 = processor.process(Input::from_text(text2)).unwrap();
    assert_eq!(result2.boundaries.len(), 1);
}

#[test]
fn test_english_pattern_recognition() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Test surprised question pattern
    let text = "What!? Really?";
    let result = processor.process(Input::from_text(text)).unwrap();
    assert_eq!(result.boundaries.len(), 2);
    assert_eq!(result.boundaries[0].offset, 6); // After "What!?"
    assert_eq!(result.boundaries[1].offset, 14); // After "Really?"
}

#[test]
fn test_english_enclosure_suppression() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Test apostrophe in contractions
    let text = "It's great. Don't worry.";
    let result = processor.process(Input::from_text(text)).unwrap();
    assert_eq!(result.boundaries.len(), 2);

    // Test list item parentheses
    let text2 = "Items: 1) First item 2) Second item.";
    let result2 = processor.process(Input::from_text(text2)).unwrap();
    assert_eq!(result2.boundaries.len(), 1); // Only at the end
}

#[test]
fn test_japanese_configurable_basic() {
    let processor = SentenceProcessor::with_language("ja").unwrap();
    let text = "こんにちは。世界。";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 2);
    assert_eq!(result.boundaries[0].offset, 18); // After "こんにちは。"
    assert_eq!(result.boundaries[1].offset, 27); // After "世界。"
}

#[test]
fn test_japanese_mixed_punctuation() {
    let processor = SentenceProcessor::with_language("ja").unwrap();
    let text = "質問があります？答えはYesです。";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 2);
}

#[test]
fn test_japanese_enclosures() {
    let processor = SentenceProcessor::with_language("ja").unwrap();
    let text = "彼は「こんにちは」と言った。";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 1);
    assert_eq!(result.boundaries[0].offset, 42); // After the final 。
}

#[test]
fn test_configurable_performance() {
    // Test that configurable implementation maintains good performance
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Generate a large text
    let sentences = vec![
        "This is sentence one.",
        "Dr. Smith said hello.",
        "What!? Really?",
        "The price is $29.99 today.",
        "Visit https://example.com for more info.",
    ];

    let large_text = sentences.repeat(1000).join(" ");

    let start = std::time::Instant::now();
    let result = processor.process(Input::from_text(&large_text)).unwrap();
    let duration = start.elapsed();

    // Should process 5000 sentences in under 100ms
    assert!(duration.as_millis() < 100);
    assert_eq!(result.boundaries.len(), 5000);
}

#[test]
fn test_edge_cases_with_configurable() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Empty text
    let result = processor.process(Input::from_text("")).unwrap();
    assert_eq!(result.boundaries.len(), 0);

    // Only whitespace
    let result = processor.process(Input::from_text("   \n\t  ")).unwrap();
    assert_eq!(result.boundaries.len(), 0);

    // No punctuation
    let result = processor.process(Input::from_text("Hello world")).unwrap();
    assert_eq!(result.boundaries.len(), 0);

    // Only punctuation
    let result = processor.process(Input::from_text("...!?")).unwrap();
    assert_eq!(result.boundaries.len(), 1);
}
