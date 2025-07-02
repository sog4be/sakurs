//! Tests for cross-chunk enclosure handling with nested quotes
//!
//! These tests verify that the Δ-Stack algorithm correctly handles
//! nested enclosures that span across chunk boundaries.

use sakurs_core::application::{ProcessorConfig, UnifiedProcessor};
use sakurs_core::domain::language::EnglishLanguageRules;
use sakurs_core::domain::state::Boundary;
use std::sync::Arc;

/// Extract sentences from text based on boundaries
fn extract_sentences(text: &str, boundaries: &[Boundary]) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for boundary in boundaries {
        if boundary.offset > start && boundary.offset <= text.len() {
            let sentence = text[start..boundary.offset].trim().to_string();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            start = boundary.offset;
        }
    }

    // Add final sentence if any
    if start < text.len() {
        let sentence = text[start..].trim().to_string();
        if !sentence.is_empty() {
            sentences.push(sentence);
        }
    }

    sentences
}

/// Generate a complex text with nested quotes that will span multiple chunks
fn generate_nested_quote_text() -> String {
    // Create a text that's large enough to be split into multiple chunks
    // with nested quotes spanning across boundaries
    let mut text = String::new();

    // Start with some regular sentences
    text.push_str("This is the beginning of our document. ");
    text.push_str("We need some initial content before the complex part. ");

    // Add filler to ensure we hit chunk boundaries
    for i in 0..50 {
        text.push_str(&format!("This is filler sentence number {}. ", i));
    }

    // Now add the complex nested quote structure
    text.push_str("The professor said, \"In his famous work, Shakespeare wrote 'To be, or not to be,' which has been interpreted in many ways. ");

    // Add more content inside the outer quote to force chunk split
    for i in 0..100 {
        text.push_str(&format!("The analysis continues with point number {}. ", i));
    }

    // Continue the nested quote
    text.push_str("Another scholar argued, 'This interpretation misses the point,' but the debate continues.\" ");

    // Add more content after
    for i in 0..50 {
        text.push_str(&format!("Post-quote content sentence {}. ", i));
    }

    // Add another complex nested structure
    text.push_str("She remarked, \"The article stated: 'The data shows \"significant results\" in the study.' This is groundbreaking.\" ");

    text
}

/// Generate text with Japanese-style nested quotes
fn generate_japanese_style_nested_quotes() -> String {
    let mut text = String::new();

    // Initial content
    for i in 0..30 {
        text.push_str(&format!("前文の内容その{}。", i));
    }

    // Nested Japanese quotes: 「」 and 『』
    text.push_str("彼は言った、「この本の中で、著者は『人生とは何か』と問いかけている。");

    // Force chunk boundary inside nested quotes
    for i in 0..80 {
        text.push_str(&format!("引用内の説明文その{}。", i));
    }

    text.push_str("さらに『答えは一つではない』とも述べている。」と。");

    // More content
    for i in 0..30 {
        text.push_str(&format!("後文の内容その{}。", i));
    }

    text
}

/// Generate text with mixed quote styles and parentheses
fn generate_mixed_enclosures() -> String {
    let mut text = String::new();

    // Build up to chunk boundary
    for i in 0..40 {
        text.push_str(&format!("Sentence number {} ends here. ", i));
    }

    // Complex mixed enclosure
    text.push_str("The report (which stated \"the results show 'significant improvement' in performance\") was controversial. ");

    // Force chunk split inside nested structure
    for i in 0..60 {
        text.push_str(&format!("Additional analysis point {} is discussed. ", i));
    }

    // Another mixed structure spanning chunks
    text.push_str("According to the manual (see section \"Configuration\" under 'Advanced Settings'), the following applies. ");

    // More content
    for i in 0..40 {
        text.push_str(&format!("Final section sentence {}. ", i));
    }

    text
}

#[test]
fn test_nested_quotes_across_chunks() {
    let text = generate_nested_quote_text();
    let rules = Arc::new(EnglishLanguageRules::new());

    // Configure for small chunks to ensure splits occur within quotes
    let mut config = ProcessorConfig::default();
    config.chunk_size = 1024; // Small chunks to force splits
    config.overlap_size = 50; // Reasonable overlap

    let processor = UnifiedProcessor::with_config(rules, config);

    // Process the text
    let result = processor.process(&text).unwrap();

    // Verify no sentences are detected inside the quotes
    let sentences = extract_sentences(&text, &result.boundaries);

    for (i, sentence) in sentences.iter().enumerate() {
        // Check if this sentence is inside quotes by examining the text before it
        let sentence_start = text.find(sentence).unwrap();
        let before_text = &text[..sentence_start];
        let double_quotes = before_text.matches('"').count();
        let single_quotes = before_text.matches('\'').count();

        // If quotes are unbalanced, we're inside a quote
        if double_quotes % 2 != 0 || single_quotes % 2 != 0 {
            // Check if this is actually a complete quoted sentence
            let is_complete_quote =
                sentence.starts_with('"') && sentence.ends_with('"') || sentence.contains("\" ");

            if !is_complete_quote {
                panic!(
                    "Sentence boundary detected inside quotes at position {}: '{}'",
                    sentence_start, sentence
                );
            }
        }
    }

    // Verify we found the expected sentence boundaries
    assert!(sentences.len() > 100, "Should detect many sentences");

    // Verify chunk boundaries don't affect quote handling
    println!("Processed {} chunks", result.metrics.chunk_count);
    assert!(
        result.metrics.chunk_count > 5,
        "Text should be split into multiple chunks"
    );
}

#[test]
fn test_japanese_nested_quotes_across_chunks() {
    let text = generate_japanese_style_nested_quotes();
    let rules = Arc::new(EnglishLanguageRules::new()); // Note: using English rules for now

    let mut config = ProcessorConfig::default();
    config.chunk_size = 512; // Even smaller chunks for Japanese text
    config.overlap_size = 30;

    let processor = UnifiedProcessor::with_config(rules, config);
    let result = processor.process(&text).unwrap();

    // Verify proper handling of nested quotes
    // This is a simplified check since we're using English rules
    let sentences = extract_sentences(&text, &result.boundaries);
    assert!(sentences.len() > 50, "Should detect sentences");
    assert!(result.metrics.chunk_count > 3, "Should use multiple chunks");
}

#[test]
fn test_mixed_enclosures_across_chunks() {
    let text = generate_mixed_enclosures();
    let rules = Arc::new(EnglishLanguageRules::new());

    let mut config = ProcessorConfig::default();
    config.chunk_size = 800;
    config.overlap_size = 40;

    let processor = UnifiedProcessor::with_config(rules, config);
    let result = processor.process(&text).unwrap();

    // Verify parentheses don't interfere with sentence detection
    let sentences = extract_sentences(&text, &result.boundaries);

    for sentence in &sentences {
        // Check for common issues
        assert!(!sentence.trim().is_empty(), "Empty sentence detected");
        assert!(
            sentence.trim().ends_with('.')
                || sentence.trim().ends_with('!')
                || sentence.trim().ends_with('?'),
            "Sentence doesn't end with proper punctuation: '{}'",
            sentence.trim()
        );
    }

    assert!(sentences.len() > 80, "Should detect many sentences");
}

#[test]
fn test_enclosure_depth_tracking_across_chunks() {
    // Create a pathological case with deeply nested quotes
    let mut text = String::new();

    // Build up text
    for i in 0..30 {
        text.push_str(&format!("Initial sentence {}. ", i));
    }

    // Start deep nesting that will span chunks
    text.push_str("He said, \"She told me, 'They wrote: \"The study found 'significant results' in the data.\" This is important.' I agree.\" ");

    // Add more content
    for i in 0..30 {
        text.push_str(&format!("Final sentence {}. ", i));
    }

    let rules = Arc::new(EnglishLanguageRules::new());
    let mut config = ProcessorConfig::default();
    config.chunk_size = 400; // Force chunk split in the middle of nested quotes

    let processor = UnifiedProcessor::with_config(rules, config);
    let result = processor.process(&text).unwrap();

    // The important quote sentence should be detected as a single sentence
    let sentences = extract_sentences(&text, &result.boundaries);
    let quote_sentences: Vec<_> = sentences.iter().filter(|s| s.contains("He said")).collect();

    assert_eq!(
        quote_sentences.len(),
        1,
        "Complex nested quote should be detected as single sentence"
    );
}

#[test]
fn test_parallel_vs_sequential_consistency_with_nested_quotes() {
    let text = generate_nested_quote_text();
    let rules = Arc::new(EnglishLanguageRules::new());

    // Process with parallel enabled
    let mut config = ProcessorConfig::default();
    config.chunk_size = 1024;

    let processor = UnifiedProcessor::with_config(rules, config);

    // Process with multiple threads (parallel)
    let parallel_result = processor.process_with_threads(&text, 4).unwrap();

    // Process with single thread (sequential)
    let sequential_result = processor.process_with_threads(&text, 1).unwrap();

    // Results should be identical
    assert_eq!(
        parallel_result.boundaries.len(),
        sequential_result.boundaries.len(),
        "Parallel and sequential processing should find same number of boundaries"
    );

    // Check each boundary matches
    for (p_boundary, s_boundary) in parallel_result
        .boundaries
        .iter()
        .zip(sequential_result.boundaries.iter())
    {
        assert_eq!(
            p_boundary.offset, s_boundary.offset,
            "Boundary positions should match between parallel and sequential"
        );
        assert_eq!(
            p_boundary.flags, s_boundary.flags,
            "Boundary flags should match"
        );
    }
}

#[test]
fn test_quote_at_exact_chunk_boundary() {
    let mut text = String::new();

    // Create text where quote mark falls exactly at chunk boundary
    let chunk_size = 100;

    // Fill exactly to chunk boundary minus 1
    for _ in 0..chunk_size / 10 - 1 {
        text.push_str("Fill text. ");
    }

    // Position quote right at boundary
    text.push_str("He said");
    assert!(text.len() < chunk_size);
    text.push_str("\""); // This quote should be right at/near boundary
    assert!(text.len() >= chunk_size - 1 && text.len() <= chunk_size + 1);

    // Continue quote content
    text.push_str("This quote spans across the chunk boundary and continues for a while. ");
    text.push_str("It should be handled correctly.\" ");

    // Add more sentences
    for i in 0..10 {
        text.push_str(&format!("Follow-up sentence {}. ", i));
    }

    let rules = Arc::new(EnglishLanguageRules::new());
    let mut config = ProcessorConfig::default();
    config.chunk_size = chunk_size;
    config.overlap_size = 10;

    let processor = UnifiedProcessor::with_config(rules, config);
    let result = processor.process(&text).unwrap();

    // Verify the quote is handled as one sentence
    let sentences = extract_sentences(&text, &result.boundaries);
    let quote_sentences: Vec<_> = sentences.iter().filter(|s| s.contains("He said")).collect();

    assert_eq!(
        quote_sentences.len(),
        1,
        "Quote at chunk boundary should be one sentence"
    );

    // Verify chunks were actually created
    assert!(
        result.metrics.chunk_count >= 2,
        "Should create multiple chunks"
    );
}
