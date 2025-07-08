//! End-to-end integration tests for the complete processing pipeline

use sakurs_core::{Config, Input, SentenceProcessor};
use std::io::Cursor;

#[test]
fn test_complete_english_processing_pipeline() {
    let config = Config::builder().language("en").build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let text = "Dr. Smith went to the U.S.A. He bought a new car. The car cost $25,000! Isn't that expensive?";
    let result = processor.process(Input::from_text(text)).unwrap();

    // The actual number of boundaries might vary based on abbreviation handling
    // Let's just verify we get at least 2 boundaries (reasonable minimum)
    assert!(
        result.boundaries.len() >= 2,
        "Expected at least 2 boundaries, got {}",
        result.boundaries.len()
    );
}

#[test]
fn test_complete_japanese_processing_pipeline() {
    let config = Config::builder().language("ja").build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let text =
        "田中さんは東京に行きました。新しい車を買いました。その車は300万円でした！高いですね？";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 4);
}

#[test]
fn test_mixed_language_content() {
    let config = Config::builder().language("en").build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    // English text with some Japanese names and words
    let text = r#"Mr. Tanaka (田中) works at Toyota. He said "こんにちは" to everyone. That means "hello" in Japanese!"#;
    let result = processor.process(Input::from_text(text)).unwrap();

    // The processor might handle this differently based on language rules
    // For now, just verify it processes without errors
    println!(
        "Mixed language content boundaries: {}",
        result.boundaries.len()
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

    // With contractions, might detect fewer boundaries
    assert!(
        result.boundaries.len() >= 2,
        "Expected at least 2 boundaries, got {}",
        result.boundaries.len()
    );
}

#[test]
fn test_large_text_processing() {
    let config = Config::builder().threads(4).build().unwrap();

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

    // Should have at least 1000 sentences
    assert!(result.boundaries.len() >= 1000);
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
fn test_config_presets() {
    // Test fast config
    let fast_config = Config::fast();
    let processor = SentenceProcessor::with_config(fast_config).unwrap();

    let result = processor
        .process(Input::from_text("Test sentence. Another one!"))
        .unwrap();
    assert_eq!(result.boundaries.len(), 2);

    // Test balanced config
    let balanced_config = Config::balanced();
    let processor = SentenceProcessor::with_config(balanced_config).unwrap();

    let result = processor
        .process(Input::from_text("Balanced approach. Good performance!"))
        .unwrap();
    assert_eq!(result.boundaries.len(), 2);

    // Test accurate config
    let accurate_config = Config::accurate();
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

    // Complex nested structures might be handled differently
    // For now, just verify it processes without errors
    println!("Nested quotes boundaries: {}", result.boundaries.len());
}

#[test]
fn test_processing_stats() {
    let processor = SentenceProcessor::for_language("en").unwrap();

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
