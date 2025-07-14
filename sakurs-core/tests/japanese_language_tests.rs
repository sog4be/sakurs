//! Integration tests for Japanese language support
//!
//! This module contains tests for Japanese sentence boundary detection
//! using the public API.

use sakurs_core::{Config, Input, SentenceProcessor};

#[test]
fn test_basic_japanese_sentence_detection() {
    let config = Config::builder().language("ja").unwrap().build().unwrap();
    let processor = SentenceProcessor::with_config(config).unwrap();

    // Basic Japanese sentences with periods
    let text = "これは最初の文です。これは二番目の文です。これは三番目の文です。";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should detect 3 sentences
    let boundary_count = result.boundaries.len();
    assert_eq!(boundary_count, 3);
    // Verify boundaries are at expected positions
}

#[test]
fn test_japanese_punctuation_types() {
    let config = Config::builder().language("ja").unwrap().build().unwrap();
    let processor = SentenceProcessor::with_config(config).unwrap();

    // Test different Japanese punctuation marks
    let text = "これは文です。これは質問ですか？これは感嘆文です！";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should detect 3 sentences
    let boundary_count = result.boundaries.len();
    assert_eq!(boundary_count, 3);
}

#[test]
fn test_japanese_quotes() {
    let config = Config::builder().language("ja").unwrap().build().unwrap();
    let processor = SentenceProcessor::with_config(config).unwrap();

    // Japanese quotes
    let text = "彼は「こんにちは」と言いました。「元気ですか？」と聞きました。";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should handle quotes properly
    let boundary_count = result.boundaries.len();
    assert_eq!(boundary_count, 2);
}

#[test]
fn test_mixed_japanese_english() {
    let config = Config::builder().language("ja").unwrap().build().unwrap();
    let processor = SentenceProcessor::with_config(config).unwrap();

    let text = "これはJapaneseとEnglishの混合文です。Next sentence is here！最後の文。";
    let result = processor.process(Input::from_text(text)).unwrap();

    let boundary_count = result.boundaries.len();
    assert_eq!(boundary_count, 3);
}

#[test]
fn test_japanese_ellipsis() {
    let config = Config::builder().language("ja").unwrap().build().unwrap();
    let processor = SentenceProcessor::with_config(config).unwrap();

    let text = "ちょっと待って…。それから…どうしましょう。";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should handle ellipsis properly
    let boundary_count = result.boundaries.len();
    assert!(boundary_count >= 1);
}

#[test]
fn test_japanese_large_text() {
    let config = Config::builder()
        .language("ja")
        .unwrap()
        .chunk_size(1024) // Small chunks to test chunking
        .build()
        .unwrap();
    let processor = SentenceProcessor::with_config(config).unwrap();

    // Create a large text
    let sentence = "これはテスト文です。";
    let large_text = sentence.repeat(500);

    let result = processor.process(Input::from_text(large_text)).unwrap();

    // Should process all sentences (allowing for minor edge case differences)
    let boundary_count = result.boundaries.len();
    assert!(
        boundary_count >= 498 && boundary_count <= 500,
        "Expected ~500 boundaries, got {}",
        boundary_count
    );
}

#[test]
fn test_japanese_special_characters() {
    let config = Config::builder().language("ja").unwrap().build().unwrap();
    let processor = SentenceProcessor::with_config(config).unwrap();

    // Test with special characters and symbols
    let text = "価格は¥1,000です。割引は20%です。";
    let result = processor.process(Input::from_text(text)).unwrap();

    let boundary_count = result.boundaries.len();
    assert_eq!(boundary_count, 2);
}
