//! Tests specifically for UTF-8 boundary safety in overlap-based chunking

use sakurs_core::{
    application::chunking::{OverlapChunkConfig, OverlapChunkManager},
    domain::enclosure_suppressor::EnglishEnclosureSuppressor,
};
use std::sync::Arc;

#[test]
fn test_utf8_multibyte_apostrophe_safety() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = OverlapChunkConfig {
        chunk_size: 30,
        overlap_size: 15,
        enable_cross_chunk: true,
        ..Default::default()
    };
    let mut manager = OverlapChunkManager::new(config, suppressor);

    // Test with Unicode apostrophe U+2019 (3 bytes in UTF-8)
    let text = "This isn\u{2019}t working. That\u{2019}s good."; // Using U+2019
    let result = manager.chunk_with_overlap_processing(text);

    // Should not panic and process successfully
    assert!(result.is_ok());
    let chunks = result.unwrap();
    assert!(!chunks.is_empty());
}

#[test]
fn test_utf8_japanese_with_apostrophes() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = OverlapChunkConfig {
        chunk_size: 40,
        overlap_size: 20,
        enable_cross_chunk: true,
        ..Default::default()
    };
    let mut manager = OverlapChunkManager::new(config, suppressor);

    // Mix of Japanese characters and apostrophes
    let text = "これは「テスト」です。It\u{2019}s working！それは良い。";
    let result = manager.chunk_with_overlap_processing(text);

    // Should handle multi-byte characters safely
    assert!(result.is_ok());
    let chunks = result.unwrap();
    assert!(!chunks.is_empty());
}

#[test]
fn test_utf8_emoji_boundaries() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = OverlapChunkConfig {
        chunk_size: 20,
        overlap_size: 10,
        enable_cross_chunk: true,
        ..Default::default()
    };
    let mut manager = OverlapChunkManager::new(config, suppressor);

    // Emojis are 4 bytes in UTF-8
    let text = "Hello 👋 world\u{2019}s best! 🎉";
    let result = manager.chunk_with_overlap_processing(text);

    // Should handle 4-byte UTF-8 characters
    assert!(result.is_ok());
}

#[test]
fn test_utf8_mixed_quotes() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = OverlapChunkConfig {
        chunk_size: 35,
        overlap_size: 15,
        enable_cross_chunk: true,
        ..Default::default()
    };
    let mut manager = OverlapChunkManager::new(config, suppressor);

    // Mix of different quote types (curly quotes are multi-byte)
    let text = "She said \u{201C}I\u{2019}m here\u{201D} and he\u{2019}s gone."; // U+201C, U+201D, U+2019
    let result = manager.chunk_with_overlap_processing(text);

    assert!(result.is_ok());
    let chunks = result.unwrap();

    // Check that suppressions were detected
    let has_suppressions = chunks.iter().any(|c| !c.suppression_markers.is_empty());
    assert!(has_suppressions, "Should detect suppression patterns");
}

#[test]
fn test_utf8_chunk_boundary_at_multibyte() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = OverlapChunkConfig {
        chunk_size: 20, // Small size to force splits
        overlap_size: 8,
        enable_cross_chunk: true,
        ..Default::default()
    };
    let mut manager = OverlapChunkManager::new(config, suppressor);

    // Text where chunk boundary might fall inside multi-byte char
    let text = "Test™ product\u{2019}s name"; // ™ is 3 bytes
    let result = manager.chunk_with_overlap_processing(text);

    // Should handle boundaries safely
    assert!(result.is_ok());
}

#[test]
fn test_utf8_cyrillic_text() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = OverlapChunkConfig {
        chunk_size: 25,
        overlap_size: 12,
        enable_cross_chunk: true,
        ..Default::default()
    };
    let mut manager = OverlapChunkManager::new(config, suppressor);

    // Cyrillic characters are 2 bytes each
    let text = "Это тест. It\u{2019}s тест!";
    let result = manager.chunk_with_overlap_processing(text);

    assert!(result.is_ok());
}

#[test]
fn test_utf8_very_small_chunks_with_multibyte() {
    let suppressor = Arc::new(EnglishEnclosureSuppressor::new());
    let config = OverlapChunkConfig {
        chunk_size: 10, // Small but valid
        overlap_size: 4,
        enable_cross_chunk: true,
        ..Default::default()
    };
    let mut manager = OverlapChunkManager::new(config, suppressor);

    // Each Chinese character is 3 bytes
    let text = "我\u{2019}s 好";
    let result = manager.chunk_with_overlap_processing(text);

    // Should handle even with tiny chunks
    assert!(result.is_ok());
}
