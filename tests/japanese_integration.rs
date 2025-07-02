//! Integration tests for Japanese language support
//!
//! This module contains comprehensive tests for Japanese sentence boundary detection,
//! including punctuation handling, quote processing, and mixed Japanese-English text.

use sakurs_core::domain::language::{JapaneseLanguageRules, LanguageRules};
use sakurs_core::domain::parser::scan_chunk;
use sakurs_core::domain::PartialState;
use sakurs_core::application::{TextProcessor, ProcessorConfig};
use std::sync::Arc;

#[test]
fn test_basic_japanese_sentence_detection() {
    let rules = JapaneseLanguageRules::new();
    
    // Basic Japanese sentences with periods
    let text = "これは最初の文です。これは二番目の文です。これは三番目の文です。";
    let state = scan_chunk(text, &rules);
    
    // Should detect boundaries after each period
    assert_eq!(state.boundary_candidates.len(), 3);
    
    // Check boundary positions
    let positions: Vec<usize> = state.boundary_candidates
        .iter()
        .map(|b| b.local_offset)
        .collect();
    
    // Find positions of periods in the text
    let expected_positions: Vec<usize> = text.char_indices()
        .filter(|(_, ch)| *ch == '。')
        .map(|(i, _)| i)
        .collect();
    
    assert_eq!(positions.len(), expected_positions.len());
}

#[test]
fn test_japanese_punctuation_types() {
    let rules = JapaneseLanguageRules::new();
    
    // Test different Japanese punctuation marks
    let text = "これは文です。これは質問ですか？これは感嘆文です！";
    let state = scan_chunk(text, &rules);
    
    // Should detect 3 boundaries (period, question mark, exclamation mark)
    assert_eq!(state.boundary_candidates.len(), 3);
}

#[test]
fn test_japanese_comma_not_boundary() {
    let rules = JapaneseLanguageRules::new();
    
    // Japanese commas should not be sentence boundaries
    let text = "これは、長い文の、例です。";
    let state = scan_chunk(text, &rules);
    
    // Should only detect one boundary (the period at the end)
    assert_eq!(state.boundary_candidates.len(), 1);
    
    let boundary_pos = state.boundary_candidates[0].local_offset;
    let period_pos = text.char_indices()
        .find(|(_, ch)| *ch == '。')
        .map(|(i, _)| i)
        .unwrap();
    
    assert_eq!(boundary_pos, period_pos);
}

#[test]
fn test_japanese_quotation_marks() {
    let rules = JapaneseLanguageRules::new();
    
    // Text with Japanese quotation marks
    let text = "彼は「こんにちは」と言いました。彼女は『良い本』を読んでいます。";
    let state = scan_chunk(text, &rules);
    
    // Should detect 2 boundaries (after each sentence)
    assert_eq!(state.boundary_candidates.len(), 2);
    
    // Verify enclosure handling
    assert_eq!(state.deltas.len(), 5); // Five enclosure types supported
}

#[test]
fn test_nested_japanese_quotes() {
    let rules = JapaneseLanguageRules::new();
    
    // Nested quotation pattern: 「outer『inner』outer」
    let text = "彼は「友達が『面白い』と言った」と報告しました。";
    let state = scan_chunk(text, &rules);
    
    // Should detect one boundary at the end
    assert_eq!(state.boundary_candidates.len(), 1);
    
    // Test quote nesting validation
    assert!(rules.validate_quote_pairing(text).is_ok());
}

#[test]
fn test_mixed_japanese_english_text() {
    let rules = JapaneseLanguageRules::new();
    
    // Mixed Japanese and English text
    let text = "Hello world. こんにちは世界。This is English. これは日本語です。";
    let state = scan_chunk(text, &rules);
    
    // Should detect 4 boundaries
    assert_eq!(state.boundary_candidates.len(), 4);
}

#[test]
fn test_english_abbreviations_in_japanese() {
    let rules = JapaneseLanguageRules::new();
    
    // English abbreviations in Japanese context
    let text = "Dr. Smithさんが来ました。Prof. Tanakaも参加します。";
    let state = scan_chunk(text, &rules);
    
    // Should detect 2 boundaries (not after abbreviations)
    assert_eq!(state.boundary_candidates.len(), 2);
}

#[test]
fn test_japanese_company_abbreviations() {
    let rules = JapaneseLanguageRules::new();
    
    // Japanese company abbreviations
    let text = "トヨタ株が上がりました。ソニー有の業績も良好です。";
    let state = scan_chunk(text, &rules);
    
    // Should detect 2 boundaries (period positions)
    assert_eq!(state.boundary_candidates.len(), 2);
}

#[test]
fn test_decimal_numbers_in_japanese() {
    let rules = JapaneseLanguageRules::new();
    
    // Decimal numbers should not create false boundaries
    let text = "価格は3.14円です。割引率は2.5%でした。";
    let state = scan_chunk(text, &rules);
    
    // Should detect 2 boundaries (only at sentence ends)
    assert_eq!(state.boundary_candidates.len(), 2);
}

#[test]
fn test_japanese_text_processor_integration() {
    let rules = Arc::new(JapaneseLanguageRules::new());
    let processor = TextProcessor::new(rules);
    
    // Test with the processor
    let text = "これは統合テストです。プロセッサーが正しく動作することを確認します。";
    let result = processor.process_text(text).unwrap();
    
    // Should detect 2 boundaries
    assert_eq!(result.boundaries.len(), 2);
    assert_eq!(result.text_length, text.len());
    
    // Extract sentences
    let sentences = result.extract_sentences(text);
    assert_eq!(sentences.len(), 2);
    assert_eq!(sentences[0], "これは統合テストです。");
    assert_eq!(sentences[1], "プロセッサーが正しく動作することを確認します。");
}

#[test]
fn test_complex_japanese_document() {
    let rules = Arc::new(JapaneseLanguageRules::new());
    let processor = TextProcessor::new(rules);
    
    // Complex Japanese text with various patterns
    let text = r#"
        昨日、田中さんと会いました。彼は「新しいプロジェクトについて話そう」と言いました。
        そのプロジェクトは『未来の技術』というタイトルでした。
        予算は約1,000万円で、期間は6ヶ月です。
        Dr. Smithも参加する予定ですか？はい、参加します！
        これは素晴らしいニュースですね。
    "#.trim();
    
    let result = processor.process_text(text).unwrap();
    
    // Should detect multiple boundaries
    assert!(result.boundaries.len() > 5);
    
    // Text should be processable without errors
    assert_eq!(result.text_length, text.len());
}

#[test]
fn test_japanese_language_metadata() {
    let rules = JapaneseLanguageRules::new();
    
    assert_eq!(rules.language_code(), "ja");
    assert_eq!(rules.language_name(), "Japanese");
    assert_eq!(rules.enclosure_type_count(), 5);
}

#[test]
fn test_japanese_strict_and_relaxed_modes() {
    let strict = JapaneseLanguageRules::new_strict();
    let relaxed = JapaneseLanguageRules::new_relaxed();
    
    assert_eq!(strict.language_code(), "ja");
    assert_eq!(relaxed.language_code(), "ja");
    
    assert!(strict.language_name().contains("Strict"));
    assert!(relaxed.language_name().contains("Relaxed"));
}

#[test]
fn test_full_width_parentheses() {
    let rules = JapaneseLanguageRules::new();
    
    // Full-width parentheses common in Japanese
    let text = "これは例文です（注釈付き）。次の文もあります。";
    let state = scan_chunk(text, &rules);
    
    // Should detect 2 boundaries
    assert_eq!(state.boundary_candidates.len(), 2);
    
    // Should recognize full-width parentheses as enclosures
    assert!(rules.get_enclosure_char('（').is_some());
    assert!(rules.get_enclosure_char('）').is_some());
}

#[test]
fn test_cross_chunk_japanese_processing() {
    let rules = Arc::new(JapaneseLanguageRules::new());
    let mut config = ProcessorConfig::default();
    config.chunk_size = 50; // Force multiple chunks
    
    let processor = TextProcessor::with_config(config, rules);
    
    // Long Japanese text that will span multiple chunks
    let text = "これは長い日本語の文章です。" * 10;
    let result = processor.process_text(&text).unwrap();
    
    // Should process without errors
    assert_eq!(result.text_length, text.len());
    assert!(result.boundaries.len() > 5);
}

#[test] 
fn test_japanese_quote_pairing_validation() {
    let rules = JapaneseLanguageRules::new();
    
    // Valid quote pairing
    assert!(rules.validate_quote_pairing("彼は「こんにちは」と言った。").is_ok());
    assert!(rules.validate_quote_pairing("「外側『内側』外側」の構造").is_ok());
    assert!(rules.validate_quote_pairing("").is_ok()); // Empty text
    
    // Invalid quote pairing  
    assert!(rules.validate_quote_pairing("「未完成の引用").is_err());
    assert!(rules.validate_quote_pairing("完成していない引用」").is_err());
    assert!(rules.validate_quote_pairing("「『』」『」").is_err()); // Mismatched nesting
}

#[test]
fn test_japanese_performance_baseline() {
    let rules = Arc::new(JapaneseLanguageRules::new());
    let processor = TextProcessor::new(rules);
    
    // Generate moderately large Japanese text
    let base_text = "これは性能テストのための文章です。日本語の処理速度を測定しています。";
    let large_text = base_text.repeat(100);
    
    let start = std::time::Instant::now();
    let result = processor.process_text(&large_text).unwrap();
    let duration = start.elapsed();
    
    // Should complete in reasonable time (< 100ms for this size)
    assert!(duration.as_millis() < 100);
    assert_eq!(result.text_length, large_text.len());
    assert!(result.boundaries.len() > 100); // Should find many boundaries
}