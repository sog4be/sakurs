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

    // Three boundaries: after "Toyota.", "everyone.", and "Japanese!" —
    // the quoted spans (ASCII and Japanese quotes) do not leak boundaries.
    assert_eq!(
        result.boundaries.len(),
        3,
        "Expected 3 boundaries for mixed language with quotes"
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

    // Quotes suppress boundaries WITHIN them, not after closing quotes
    // Expected boundaries:
    // 1. After "yesterday." Then he left." (offset 61)
    // 2. After "Done." (offset 113)
    assert_eq!(result.boundaries.len(), 2);
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
fn test_boundaries_are_ordered() {
    let processor = SentenceProcessor::new();

    let text = "Strong boundary. Weak boundary maybe? Another!";
    let result = processor.process(Input::from_text(text)).unwrap();

    let offsets: Vec<usize> = result.boundaries.iter().map(|b| b.offset).collect();
    let mut sorted = offsets.clone();
    sorted.sort_unstable();
    assert_eq!(offsets, sorted);
}

/// Splits `text` at the reported boundaries (trimmed, empties dropped), the
/// way the adapters present sentences.
fn split_en(text: &str) -> Vec<String> {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let result = processor.process(Input::from_text(text)).unwrap();
    let mut sentences = Vec::new();
    let mut start = 0;
    for b in &result.boundaries {
        let piece = text[start..b.offset].trim();
        if !piece.is_empty() {
            sentences.push(piece.to_string());
        }
        start = b.offset;
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        sentences.push(tail.to_string());
    }
    sentences
}

#[test]
fn abbreviation_followed_by_non_letter_keeps_sentence_open() {
    // Comma, apostrophe, and digit continuations after an abbreviation dot
    // must not end the sentence.
    assert_eq!(
        split_en("The U.S., drafted the memo. It was long."),
        ["The U.S., drafted the memo.", "It was long."]
    );
    assert_eq!(
        split_en("That is JFK Jr.'s book."),
        ["That is JFK Jr.'s book."]
    );
    assert_eq!(
        split_en("Please turn to p. 55. Next sentence."),
        ["Please turn to p. 55.", "Next sentence."]
    );
    assert_eq!(
        split_en("See Memorandum No. 178 for details."),
        ["See Memorandum No. 178 for details."]
    );
}

#[test]
fn abbreviation_at_end_of_text_is_a_boundary() {
    assert_eq!(
        split_en("They closed the deal with Pitt, Briggs & Co."),
        ["They closed the deal with Pitt, Briggs & Co."]
    );
}

#[test]
fn abbreviation_lookup_applies_only_to_periods() {
    // "No" is a configured abbreviation ("No. 178"), but "No!" and "No?" end
    // with plain terminators and must split like any exclamation or question.
    assert_eq!(
        split_en("No! Is she happy? No! But why?"),
        ["No!", "Is she happy?", "No!", "But why?"]
    );
}

#[test]
fn double_terminator_runs_bind_as_one_boundary() {
    assert_eq!(
        split_en("Hello!! Long time no see."),
        ["Hello!!", "Long time no see."]
    );
    assert_eq!(
        split_en("Hello?? Who is there?"),
        ["Hello??", "Who is there?"]
    );
}
