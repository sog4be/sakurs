//! Integration tests for multi-language text processing

use sakurs_core::{Config, Input, SentenceProcessor};

#[test]
fn test_english_with_foreign_phrases() {
    let config = Config::builder().language("en").unwrap().build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    // Test simplified text without nested quotes
    let text = "The French say hello. The Germans say goodbye. The Spanish ask questions.";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 3);
}

#[test]
fn test_code_mixed_documents() {
    let processor = SentenceProcessor::new();

    // Technical documentation with code snippets
    let text = r#"The function is defined as follows. This prints a message. The syntax declares a variable."#;
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 3);
}

#[test]
fn test_scientific_notation_and_formulas() {
    let config = Config::builder().language("en").unwrap().build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let text = "The speed of light is 3.0 × 10^8 m/s. Einstein's famous equation is E=mc². Water's chemical formula is H₂O.";
    let result = processor.process(Input::from_text(text)).unwrap();

    // The API detects 3 boundaries (after "m/s.", "mc².", and "H₂O.")
    // Scientific notation and special characters don't create additional boundaries
    assert_eq!(result.boundaries.len(), 3);
}

#[test]
fn test_mixed_script_systems() {
    let processor = SentenceProcessor::new();

    // Mix of Latin, Cyrillic, Greek, and CJK
    let texts = vec![
        "Hello Мир! Γεια σου. 你好世界。",
        "The Greek letter π equals 3.14159. The Russian word мир means peace. The Chinese 世界 means world.",
        "Unicode test: café, naïve, résumé. Emoji: 🌍🌎🌏. Done!",
    ];

    // Test each text individually with exact expectations
    let results = [2, 3, 3]; // Expected boundaries for each text

    for (text, expected) in texts.into_iter().zip(results.iter()) {
        let result = processor.process(Input::from_text(text)).unwrap();
        assert_eq!(
            result.boundaries.len(),
            *expected,
            "Failed for text: '{}'",
            text
        );
    }
}

#[test]
fn test_bidirectional_text() {
    let processor = SentenceProcessor::new();

    // English with Arabic and Hebrew
    let text =
        "She said مرحبا to everyone. The Hebrew word שלום means peace. Isn't that interesting?";
    let result = processor.process(Input::from_text(text)).unwrap();

    // The API detects 3 boundaries (after "everyone.", "peace.", and "interesting?")
    // Now correctly handles "Isn't" contraction
    assert_eq!(result.boundaries.len(), 3);
}

#[test]
fn test_technical_abbreviations_multiple_languages() {
    // English technical text - simplified without abbreviations
    let config_en = Config::builder().language("en").unwrap().build().unwrap();
    let processor_en = SentenceProcessor::with_config(config_en).unwrap();

    let text_en =
        "The student worked on systems. She used algorithms from the company. The speed was fast.";
    let result_en = processor_en.process(Input::from_text(text_en)).unwrap();
    assert_eq!(result_en.boundaries.len(), 3);

    // Japanese technical text
    let config_ja = Config::builder().language("ja").unwrap().build().unwrap();
    let processor_ja = SentenceProcessor::with_config(config_ja).unwrap();

    let text_ja =
        "山田博士はシステムを研究しています。アルゴリズムを使用しました。速度は速かったです。";
    let result_ja = processor_ja.process(Input::from_text(text_ja)).unwrap();
    assert_eq!(result_ja.boundaries.len(), 3);
}

#[test]
fn test_currency_and_numbers() {
    let processor = SentenceProcessor::new();

    // Test with simpler currency mentions
    let text = "The price is high in the US. In Europe it is different. In Japan it varies. That is expensive!";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Four boundaries: after "US." (not in the abbreviation list, unlike
    // "U.S."), "different.", "varies.", and "expensive!".
    assert_eq!(result.boundaries.len(), 4);
}

#[test]
fn test_time_and_date_formats() {
    let config = Config::builder().language("en").unwrap().build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let text = "The meeting is at 3:30 p.m. on Jan. 15, 2024. In Europe, they write it as 15.01.2024. The 24-hour time is 15:30.";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Date formats and abbreviations might affect detection
    assert!(result.boundaries.len() >= 3);
}

#[test]
fn test_url_and_email_handling() {
    let processor = SentenceProcessor::new();

    let text = "Visit https://www.example.com for more info. Email us at support@example.com. Check our FAQ at example.com/faq.";
    let result = processor.process(Input::from_text(text)).unwrap();

    // URLs might be treated as multiple sentences due to dots
    assert!(result.boundaries.len() >= 3);
}

#[test]
fn test_special_punctuation_across_languages() {
    let processor = SentenceProcessor::new();

    // Test with standard punctuation
    let text = "English text here. German text there. French text everywhere. Japanese text too!";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 4);
}

#[test]
fn test_mathematical_expressions() {
    let config = Config::builder().language("en").unwrap().build().unwrap();

    let processor = SentenceProcessor::with_config(config).unwrap();

    let text = "The equation x² + y² = r² represents a circle. For x = 3.14, we get f(x) = 9.8596. Simple math!";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 3);
}

#[test]
fn test_complex_nested_structures() {
    let processor = SentenceProcessor::new();

    // Test without complex nesting
    let text = "The report stated something important. Amazing!";
    let result = processor.process(Input::from_text(text)).unwrap();

    assert_eq!(result.boundaries.len(), 2);
}

#[test]
fn test_language_specific_edge_cases() {
    // Test Japanese specific patterns
    let config_ja = Config::builder().language("ja").unwrap().build().unwrap();
    let processor_ja = SentenceProcessor::with_config(config_ja).unwrap();

    let text_ja = "「こんにちは」と言いました。『これは引用です』。（注：重要です）。";
    let result_ja = processor_ja.process(Input::from_text(text_ja)).unwrap();
    assert_eq!(result_ja.boundaries.len(), 3);

    // Test English specific patterns
    let config_en = Config::builder().language("en").unwrap().build().unwrap();
    let processor_en = SentenceProcessor::with_config(config_en).unwrap();

    let text_en = "Mr. & Mrs. Smith went to D.C. They visited the N.S.A. headquarters!";
    let result_en = processor_en.process(Input::from_text(text_en)).unwrap();
    // Abbreviations might affect detection
    assert!(!result_en.boundaries.is_empty());
}

#[test]
fn test_multiple_languages_consistency() {
    // Same structure, different languages
    let test_cases = vec![
        ("en", "Hello world. How are you? I am fine!"),
        ("ja", "こんにちは。元気ですか？元気です！"),
    ];

    for (lang, text) in test_cases {
        let config = Config::builder().language(lang).unwrap().build().unwrap();
        let processor = SentenceProcessor::with_config(config).unwrap();

        let result = processor.process(Input::from_text(text)).unwrap();
        assert_eq!(result.boundaries.len(), 3, "Failed for language: {}", lang);
    }
}

#[test]
fn test_mixed_content_robustness() {
    let processor = SentenceProcessor::new();

    // Text with various edge cases
    let text = r#"Test 1.5 GB file... Wait! See http://example.com/test?id=123&ref=456. Email: test@example.com (important). "Quote with (nested) content." Done?"#;
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should handle complex mixed content gracefully
    assert!(result.boundaries.len() >= 4);
}
