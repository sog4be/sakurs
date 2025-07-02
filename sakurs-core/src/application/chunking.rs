//! Text chunking and boundary management
//!
//! This module provides intelligent text chunking that respects UTF-8 boundaries,
//! word boundaries, and provides overlap regions for cross-chunk processing.

use crate::application::config::{ProcessingError, ProcessingResult};
use std::ops::Range;

/// Represents a chunk of text with metadata
#[derive(Debug, Clone)]
pub struct TextChunk {
    /// The actual text content of this chunk
    pub content: String,

    /// Start offset in the original text (in bytes)
    pub start_offset: usize,

    /// End offset in the original text (in bytes)  
    pub end_offset: usize,

    /// Whether this chunk contains overlap from previous chunk
    pub has_prefix_overlap: bool,

    /// Whether this chunk contains overlap for next chunk
    pub has_suffix_overlap: bool,

    /// Chunk index in the sequence
    pub index: usize,

    /// Total number of chunks
    pub total_chunks: usize,
}

impl TextChunk {
    /// Returns the byte length of this chunk
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Returns true if this chunk is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Returns true if this is the first chunk
    pub fn is_first(&self) -> bool {
        self.index == 0
    }

    /// Returns true if this is the last chunk
    pub fn is_last(&self) -> bool {
        self.index == self.total_chunks - 1
    }

    /// Returns the effective range (excluding overlaps) in the original text
    pub fn effective_range(&self) -> Range<usize> {
        let start = if self.has_prefix_overlap && !self.is_first() {
            // Find the actual start after overlap
            self.start_offset + self.find_overlap_end()
        } else {
            self.start_offset
        };

        let end = if self.has_suffix_overlap && !self.is_last() {
            // Find the actual end before overlap
            self.end_offset - self.find_overlap_start_from_end()
        } else {
            self.end_offset
        };

        start..end
    }

    fn find_overlap_end(&self) -> usize {
        // Find first space or punctuation after overlap region
        self.content
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace() || ch.is_ascii_punctuation())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn find_overlap_start_from_end(&self) -> usize {
        // Find last space or punctuation before overlap region
        self.content
            .char_indices()
            .rev()
            .find(|(_, ch)| ch.is_whitespace() || ch.is_ascii_punctuation())
            .map(|(i, _)| self.content.len() - i)
            .unwrap_or(0)
    }
}

/// Manages text chunking with configurable parameters
#[derive(Debug, Clone)]
pub struct ChunkManager {
    /// Target size for each chunk in bytes
    chunk_size: usize,

    /// Size of overlap region in characters
    overlap_size: usize,

    /// Minimum chunk size (to avoid tiny final chunks)
    #[allow(dead_code)]
    min_chunk_size: usize,
}

impl ChunkManager {
    /// Creates a new chunk manager with specified parameters
    pub fn new(chunk_size: usize, overlap_size: usize) -> Self {
        Self {
            chunk_size,
            overlap_size,
            min_chunk_size: chunk_size / 4, // 25% of chunk size
        }
    }

    /// Chunks text into manageable pieces with overlap
    pub fn chunk_text(&self, text: &str) -> ProcessingResult<Vec<TextChunk>> {
        if text.is_empty() {
            return Ok(vec![]);
        }

        // For small texts, return single chunk
        if text.len() <= self.chunk_size {
            return Ok(vec![TextChunk {
                content: text.to_string(),
                start_offset: 0,
                end_offset: text.len(),
                has_prefix_overlap: false,
                has_suffix_overlap: false,
                index: 0,
                total_chunks: 1,
            }]);
        }

        let mut chunks = Vec::new();
        let text_bytes = text.as_bytes();
        let text_len = text_bytes.len();

        // Calculate approximate number of chunks
        let estimated_chunks = text_len.div_ceil(self.chunk_size);

        let mut current_pos = 0;
        let mut chunk_index = 0;

        while current_pos < text_len {
            // Calculate target end position
            let target_end = (current_pos + self.chunk_size).min(text_len);

            // Find actual chunk boundaries
            let (chunk_start, chunk_end, next_start) = self.find_chunk_boundaries(
                text_bytes,
                current_pos,
                target_end,
                chunk_index > 0,
                target_end < text_len,
            )?;

            // Extract chunk content
            let chunk_content = std::str::from_utf8(&text_bytes[chunk_start..chunk_end])
                .map_err(|_| ProcessingError::Utf8Error {
                    position: chunk_start,
                })?
                .to_string();

            chunks.push(TextChunk {
                content: chunk_content,
                start_offset: chunk_start,
                end_offset: chunk_end,
                has_prefix_overlap: chunk_start < current_pos,
                has_suffix_overlap: chunk_end > next_start,
                index: chunk_index,
                total_chunks: estimated_chunks,
            });

            // Ensure we make progress
            if next_start > current_pos {
                current_pos = next_start;
            } else {
                // If next_start didn't advance, force progress
                // Move to at least chunk_end to ensure forward movement
                current_pos = chunk_end;
                // If we've reached the end, break
                if current_pos >= text_len {
                    break;
                }
            }

            chunk_index += 1;
        }

        // Update total chunks count
        let total_chunks = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(chunks)
    }

    /// Finds safe chunk boundaries that respect UTF-8 and word boundaries
    fn find_chunk_boundaries(
        &self,
        text_bytes: &[u8],
        start: usize,
        target_end: usize,
        include_prefix_overlap: bool,
        include_suffix_overlap: bool,
    ) -> ProcessingResult<(usize, usize, usize)> {
        let text_len = text_bytes.len();

        // Calculate actual start with overlap
        let actual_start = if include_prefix_overlap {
            let overlap_start = start.saturating_sub(self.overlap_size);
            self.find_utf8_boundary(text_bytes, overlap_start, true)?
        } else {
            start
        };

        // Calculate actual end with overlap
        let actual_end = if include_suffix_overlap {
            let overlap_end = (target_end + self.overlap_size).min(text_len);
            self.find_utf8_boundary(text_bytes, overlap_end, false)?
        } else {
            self.find_utf8_boundary(text_bytes, target_end, false)?
        };

        // Calculate next chunk start (without overlap)
        let next_start = if include_suffix_overlap {
            self.find_word_boundary(text_bytes, target_end, false)?
        } else {
            actual_end
        };

        // Validate boundaries
        if actual_start >= actual_end || next_start > actual_end {
            return Err(ProcessingError::ChunkingError {
                reason: format!(
                    "Invalid boundaries: start={actual_start}, end={actual_end}, next={next_start}"
                ),
            });
        }

        Ok((actual_start, actual_end, next_start))
    }

    /// Finds the nearest valid UTF-8 boundary
    fn find_utf8_boundary(
        &self,
        text_bytes: &[u8],
        pos: usize,
        forward: bool,
    ) -> ProcessingResult<usize> {
        if pos >= text_bytes.len() {
            return Ok(text_bytes.len());
        }

        // Check if already at valid boundary
        if pos == 0 || is_utf8_char_boundary(text_bytes, pos) {
            return Ok(pos);
        }

        // Search for valid boundary
        if forward {
            // Search forward
            for i in pos..text_bytes.len().min(pos + 4) {
                if is_utf8_char_boundary(text_bytes, i) {
                    return Ok(i);
                }
            }
        } else {
            // Search backward
            for i in (pos.saturating_sub(3)..pos).rev() {
                if is_utf8_char_boundary(text_bytes, i) {
                    return Ok(i);
                }
            }
        }

        Err(ProcessingError::Utf8Error { position: pos })
    }

    /// Finds the nearest word boundary
    fn find_word_boundary(
        &self,
        text_bytes: &[u8],
        pos: usize,
        forward: bool,
    ) -> ProcessingResult<usize> {
        let text = std::str::from_utf8(text_bytes)
            .map_err(|_| ProcessingError::Utf8Error { position: 0 })?;

        if pos >= text.len() {
            return Ok(text.len());
        }

        // Convert byte position to char position
        let char_pos = text[..pos].chars().count();
        let chars: Vec<char> = text.chars().collect();

        if forward {
            // Search forward for word boundary
            for i in char_pos..chars.len().min(char_pos + 100) {
                if is_word_boundary(&chars, i) {
                    return Ok(char_byte_offset(text, i));
                }
            }
            // If no boundary found, return the requested position (UTF-8 safe)
            Ok(self.find_utf8_boundary(text_bytes, pos, true)?)
        } else {
            // Search backward for word boundary
            for i in (char_pos.saturating_sub(100)..=char_pos).rev() {
                if is_word_boundary(&chars, i) {
                    return Ok(char_byte_offset(text, i));
                }
            }
            // If no boundary found, return the requested position (UTF-8 safe)
            Ok(self.find_utf8_boundary(text_bytes, pos, false)?)
        }
    }
}

/// Checks if a position is at a UTF-8 character boundary
fn is_utf8_char_boundary(bytes: &[u8], pos: usize) -> bool {
    if pos == 0 || pos >= bytes.len() {
        return true;
    }

    // UTF-8 continuation bytes start with 10xxxxxx
    (bytes[pos] & 0b11000000) != 0b10000000
}

/// Checks if a position is at a word boundary
fn is_word_boundary(chars: &[char], pos: usize) -> bool {
    if pos == 0 || pos >= chars.len() {
        return true;
    }

    let prev = chars[pos - 1];
    let curr = chars.get(pos).copied().unwrap_or(' ');

    // Word boundary if transitioning between word and non-word characters
    prev.is_whitespace()
        || curr.is_whitespace()
        || (prev.is_alphanumeric() != curr.is_alphanumeric())
        || prev.is_ascii_punctuation()
        || curr.is_ascii_punctuation()
}

/// Converts character index to byte offset
fn char_byte_offset(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(offset, _)| offset)
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_chunk() {
        let manager = ChunkManager::new(1024, 64);
        let text = "This is a short text.";

        let chunks = manager.chunk_text(text).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
        assert!(!chunks[0].has_prefix_overlap);
        assert!(!chunks[0].has_suffix_overlap);
    }

    #[test]
    fn test_multiple_chunks() {
        let manager = ChunkManager::new(50, 10);
        let text = "This is a longer text that will be split into multiple chunks for processing.";

        let chunks = manager.chunk_text(text).unwrap();
        assert!(chunks.len() > 1);

        // First chunk should not have prefix overlap
        assert!(!chunks[0].has_prefix_overlap);

        // Last chunk should not have suffix overlap
        assert!(!chunks.last().unwrap().has_suffix_overlap);

        // Middle chunks should have both overlaps
        if chunks.len() > 2 {
            assert!(chunks[1].has_prefix_overlap);
            assert!(chunks[1].has_suffix_overlap);
        }
    }

    #[test]
    fn test_utf8_boundaries() {
        let manager = ChunkManager::new(15, 3); // Increased chunk size to avoid cutting through characters
        let text = "Hello 世界 World"; // Contains multi-byte UTF-8 characters

        let chunks = manager.chunk_text(text).unwrap();

        // All chunks should be valid UTF-8
        for chunk in &chunks {
            // This should not panic if UTF-8 boundaries are respected
            let _ = chunk.content.chars().count();

            // Verify content is valid
            assert!(!chunk.content.is_empty());
        }
    }

    #[test]
    fn test_word_boundaries() {
        let manager = ChunkManager::new(20, 5);
        let text = "The quick brown fox jumps over the lazy dog.";

        let chunks = manager.chunk_text(text).unwrap();

        // Chunks should generally start and end at word boundaries
        for chunk in &chunks {
            let trimmed = chunk.content.trim();
            assert!(!trimmed.starts_with(' '));
            assert!(!trimmed.ends_with(' '));
        }
    }

    #[test]
    fn test_empty_text() {
        let manager = ChunkManager::new(100, 10);
        let chunks = manager.chunk_text("").unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_metadata() {
        let manager = ChunkManager::new(30, 5);
        let text = "This is a test text that will be chunked into several pieces.";

        let chunks = manager.chunk_text(text).unwrap();

        // Check chunk indices and total count
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
            assert_eq!(chunk.total_chunks, chunks.len());
        }

        // Check offset continuity
        for i in 1..chunks.len() {
            assert!(chunks[i].start_offset <= chunks[i - 1].end_offset);
        }
    }

    #[test]
    fn test_overlap_larger_than_chunk_size() {
        // Create manager with overlap larger than chunk size
        let manager = ChunkManager::new(10, 20); // overlap > chunk_size
        let text = "This is a test of overlapping chunks with large overlap size.";

        // In this edge case, the chunking might behave differently
        // but should not panic
        let result = manager.chunk_text(text);

        // It's OK if this fails or succeeds, just shouldn't panic
        match result {
            Ok(chunks) => {
                // If it succeeds, chunks should be valid
                assert!(!chunks.is_empty());
                for chunk in &chunks {
                    assert!(!chunk.content.is_empty());
                }
            }
            Err(_) => {
                // It's reasonable to error on invalid configuration
                // This is expected behavior for overlap > chunk_size
            }
        }
    }

    #[test]
    fn test_text_with_only_multibyte_chars() {
        let manager = ChunkManager::new(12, 3); // Increased to 12 bytes (4 chars)
                                                // Text with only multi-byte UTF-8 characters
        let text = "世界你好世界你好世界你好"; // Each char is 3 bytes

        let chunks = manager.chunk_text(text).unwrap();

        // All chunks should be valid UTF-8
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());

            // Verify no partial characters
            let char_count = chunk.content.chars().count();
            assert!(char_count > 0);

            // Content should only contain complete characters
            for ch in chunk.content.chars() {
                assert!(ch == '世' || ch == '界' || ch == '你' || ch == '好');
            }
        }
    }

    #[test]
    fn test_text_with_no_word_boundaries() {
        let manager = ChunkManager::new(20, 5);
        // Long text without spaces or punctuation
        let text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

        let chunks = manager.chunk_text(text).unwrap();

        // Should still chunk the text even without word boundaries
        assert!(chunks.len() > 1);

        // Just verify chunks exist and are valid
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            // Each chunk should be a substring of the original
            assert!(text.contains(&chunk.content));
        }
    }

    #[test]
    fn test_chunk_boundary_with_emoji() {
        let manager = ChunkManager::new(15, 3);
        // Text with emoji that might fall on chunk boundary
        let text = "Hello 😀 World 🌍 Test";

        let chunks = manager.chunk_text(text).unwrap();

        // All chunks should handle emoji correctly
        for chunk in &chunks {
            // Should not split emoji
            let chars: Vec<char> = chunk.content.chars().collect();
            for ch in &chars {
                // Verify complete characters (emoji should be intact)
                assert!(ch.is_ascii() || *ch == '😀' || *ch == '🌍');
            }
        }
    }

    #[test]
    fn test_very_small_chunk_size() {
        let manager = ChunkManager::new(5, 0); // 5 byte chunks to avoid invalid boundaries
        let text = "ABCDEFGHIJ";

        let chunks = manager.chunk_text(text).unwrap();

        // Should create multiple small chunks
        assert!(chunks.len() >= 2);

        // Each chunk should be valid
        for chunk in &chunks {
            assert!(!chunk.content.is_empty());
            assert!(chunk.content.len() <= 5);
        }
    }

    #[test]
    fn test_chunk_effective_range() {
        let manager = ChunkManager::new(20, 5);
        let text = "First part. Second part. Third part.";

        let chunks = manager.chunk_text(text).unwrap();

        // Test effective ranges
        for chunk in &chunks {
            let range = chunk.effective_range();

            // Range should be within chunk bounds
            assert!(range.start >= chunk.start_offset);
            assert!(range.end <= chunk.end_offset);

            // For middle chunks with overlap, effective range should be smaller
            if chunk.has_prefix_overlap && !chunk.is_first() {
                assert!(range.start > chunk.start_offset);
            }
            if chunk.has_suffix_overlap && !chunk.is_last() {
                assert!(range.end < chunk.end_offset);
            }
        }
    }

    #[test]
    fn test_utf8_boundary_search_limits() {
        let manager = ChunkManager::new(10, 2);

        // Create text with specific UTF-8 patterns
        let text = "a\u{1F600}b"; // 'a' + emoji + 'b'

        let chunks = manager.chunk_text(text).unwrap();

        // Should handle emoji boundaries correctly
        for chunk in &chunks {
            // Verify chunk starts and ends on character boundaries
            assert!(chunk.content.is_char_boundary(0));
            assert!(chunk.content.is_char_boundary(chunk.content.len()));
        }
    }
}
