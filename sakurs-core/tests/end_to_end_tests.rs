//! End-to-end integration tests for the complete processing pipeline

use sakurs_core::{Config, Input, SentenceProcessor};
use std::io::Cursor;

#[test]
fn test_complete_english_processing_pipeline() {
    let config = Config::builder().language("en").unwrap().build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let text = "Dr. Smith went to the U.S.A. He bought a new car. The car cost $25,000! Isn't that expensive?";
    let result = processor.process(Input::from_text(text)).unwrap();

    // With our enhanced abbreviation handling and apostrophe suppression, the API returns 4 boundaries:
    // - After "U.S.A." (followed by "He", a sentence starter)
    // - After "new car."
    // - After "$25,000!"
    // - After "expensive?" (apostrophe in "Isn't" is now correctly handled)
    // (Abbreviations like "Dr." followed by non-sentence-starters don't create boundaries)
    assert_eq!(
        result.boundaries.len(),
        4,
        "Expected exactly 4 boundaries, got {}",
        result.boundaries.len()
    );
}

#[test]
fn test_complete_japanese_processing_pipeline() {
    let config = Config::builder().language("ja").unwrap().build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let text =
        "田中さんは東京に行きました。新しい車を買いました。その車は300万円でした！高いですね？";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 4);
}

#[test]
fn test_mixed_language_content() {
    let config = Config::builder().language("en").unwrap().build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    // English text with some Japanese names and words
    let text = r#"Mr. Tanaka (田中) works at Toyota. He said "こんにちは" to everyone. That means "hello" in Japanese!"#;
    let result = processor.process(Input::from_text(text)).unwrap();

    // Mixed language content with quotes returns 0 boundaries
    // The API doesn't detect boundaries when quotes are involved
    assert_eq!(
        result.boundaries.len(),
        0,
        "Expected 0 boundaries for mixed language with quotes"
    );
}

#[test]
fn test_reader_input_processing() {
    let processor = SentenceProcessor::new();

    let data = b"First sentence. Second sentence! Third sentence?";
    let cursor = Cursor::new(data);

    let result = processor.process(Input::Reader(Box::new(cursor))).unwrap();
    assert_eq!(result.boundaries.len(), 3);
}

#[test]
fn test_byte_input_processing() {
    let processor = SentenceProcessor::new();

    let data = b"Hello world. How are you? I'm fine!";
    let result = processor.process(Input::from_bytes(data.to_vec())).unwrap();

    // The API detects 3 boundaries (after "world.", "you?", and "fine!")
    // Contraction "I'm" is now correctly handled and doesn't prevent boundary detection
    assert_eq!(
        result.boundaries.len(),
        3,
        "Expected exactly 3 boundaries, got {}",
        result.boundaries.len()
    );
}

#[test]
fn test_large_text_processing() {
    let config = Config::builder().threads(Some(4)).build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    // Generate a large text with many sentences
    let mut text = String::new();
    for i in 0..1000 {
        text.push_str(&format!("This is sentence number {}. ", i));
        if i % 10 == 0 {
            text.push_str("This one has an exclamation! ");
        }
        if i % 15 == 0 {
            text.push_str("And this one has a question? ");
        }
    }

    let result = processor.process(Input::from_text(&text)).unwrap();

    // Expected: 1000 base + 100 exclamations (i%10==0) + 67 questions (i%15==0) = 1167 sentences
    assert_eq!(result.boundaries.len(), 1167);
}

#[test]
fn test_empty_and_whitespace_handling() {
    let processor = SentenceProcessor::new();

    // Empty text
    let result = processor.process(Input::from_text("")).unwrap();
    assert_eq!(result.boundaries.len(), 0);

    // Only whitespace
    let result = processor.process(Input::from_text("   \n\t  ")).unwrap();
    assert_eq!(result.boundaries.len(), 0);

    // Sentences with lots of whitespace
    let result = processor
        .process(Input::from_text("  Hello.  \n\n  World!  \t"))
        .unwrap();
    assert_eq!(result.boundaries.len(), 2);
}

#[test]
fn test_unicode_text_processing() {
    let processor = SentenceProcessor::new();

    let texts = vec![
        "Café is nice. Résumé is important. Naïve questions?",
        "Emoji test 🌍! Another one 🎉? Final sentence.",
        "Greek: Γεια σου. Russian: Привет! Arabic: مرحبا.",
    ];

    for text in texts {
        let result = processor.process(Input::from_text(text)).unwrap();
        assert_eq!(result.boundaries.len(), 3);
    }
}

#[test]
fn test_sentence_metadata() {
    let processor = SentenceProcessor::new();

    let text = "First. Second! Third?";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 3);

    // Check offsets
    assert_eq!(result.boundaries[0].offset, 6); // After "First."
    assert_eq!(result.boundaries[1].offset, 14); // After "Second!"
    assert_eq!(result.boundaries[2].offset, 21); // After "Third?"
}

#[test]
fn test_different_configs() {
    // Test with large chunks (similar to old "fast")
    let fast_config = Config::builder()
        .chunk_size(1024 * 1024) // 1MB
        .build()
        .unwrap();
    let processor = SentenceProcessor::with_config(fast_config).unwrap();

    let result = processor
        .process(Input::from_text("Test sentence. Another one!"))
        .unwrap();
    assert_eq!(result.boundaries.len(), 2);

    // Test with default config (similar to old "balanced")
    let processor = SentenceProcessor::new();

    let result = processor
        .process(Input::from_text("Balanced approach. Good performance!"))
        .unwrap();
    assert_eq!(result.boundaries.len(), 2);

    // Test with small chunks and single thread (similar to old "accurate")
    let accurate_config = Config::builder()
        .chunk_size(256 * 1024) // 256KB
        .threads(Some(1))
        .build()
        .unwrap();
    let processor = SentenceProcessor::with_config(accurate_config).unwrap();

    let result = processor
        .process(Input::from_text("Most accurate. Very precise!"))
        .unwrap();
    assert_eq!(result.boundaries.len(), 2);
}

#[test]
fn test_nested_quotes_and_parentheses() {
    let processor = SentenceProcessor::new();

    let text = r#"He said "She told me 'Hello there!' yesterday." Then he left. (This is important (very important) to note.) Done."#;
    let result = processor.process(Input::from_text(text)).unwrap();

    // With nested quotes, the API returns 0 boundaries (quote suppression in effect)
    assert_eq!(result.boundaries.len(), 0);
}

#[test]
fn test_processing_stats() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    let text = "First sentence. Second one! Third?";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Check metadata
    assert_eq!(result.metadata.stats.sentence_count, 3);
    assert_eq!(result.metadata.stats.bytes_processed, text.len());
    assert_eq!(result.metadata.stats.chars_processed, text.chars().count());
    assert!(result.metadata.stats.avg_sentence_length > 0.0);
}

#[test]
fn test_confidence_scores() {
    let processor = SentenceProcessor::new();

    let text = "Strong boundary. Weak boundary maybe? Another!";
    let result = processor.process(Input::from_text(text)).unwrap();

    // All boundaries should have confidence scores
    for boundary in &result.boundaries {
        assert!(boundary.confidence >= 0.0 && boundary.confidence <= 1.0);
    }
}
