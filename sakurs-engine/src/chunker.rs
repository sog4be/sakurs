//! Text chunking utilities

use crate::{
    config::ChunkPolicy,
    error::{EngineError, Result},
};

/// A chunk of text with metadata
#[derive(Debug, Clone)]
pub struct TextChunk {
    /// The text content
    pub text: String,
    /// Byte offset in original text
    pub start: usize,
    /// Byte length
    pub len: usize,
}

/// Manages text chunking with UTF-8 safety
#[derive(Debug)]
pub struct ChunkManager {
    policy: ChunkPolicy,
}

impl ChunkManager {
    /// Create a new chunk manager
    pub fn new(policy: ChunkPolicy) -> Self {
        Self { policy }
    }

    /// Chunk text according to the policy
    pub fn chunk_text(&self, text: &str) -> Result<Vec<TextChunk>> {
        match self.policy {
            ChunkPolicy::Fixed { size } => self.chunk_fixed(text, size),
            ChunkPolicy::Auto { target_bytes } => self.chunk_auto(text, target_bytes),
            ChunkPolicy::Streaming {
                window_size,
                overlap,
            } => self.chunk_streaming(text, window_size, overlap),
        }
    }

    /// Fixed-size chunking
    fn chunk_fixed(&self, text: &str, chunk_size: usize) -> Result<Vec<TextChunk>> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let bytes = text.as_bytes();

        while start < bytes.len() {
            let mut end = (start + chunk_size).min(bytes.len());

            // Ensure we're at a valid UTF-8 boundary
            while end < bytes.len() && !is_char_boundary(bytes, end) {
                end -= 1;
            }

            if end <= start {
                return Err(EngineError::InvalidChunkBoundary { position: start });
            }

            chunks.push(TextChunk {
                text: text[start..end].to_string(),
                start,
                len: end - start,
            });

            start = end;
        }

        Ok(chunks)
    }

    /// Auto-sizing based on target size
    fn chunk_auto(&self, text: &str, target_bytes: usize) -> Result<Vec<TextChunk>> {
        // Determine optimal chunk count
        let total_bytes = text.len();
        let chunk_count = total_bytes.div_ceil(target_bytes).max(1);
        let chunk_size = total_bytes / chunk_count;

        self.chunk_fixed(text, chunk_size.max(1))
    }

    /// Streaming with overlap
    fn chunk_streaming(
        &self,
        text: &str,
        window_size: usize,
        overlap: usize,
    ) -> Result<Vec<TextChunk>> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let bytes = text.as_bytes();

        while start < bytes.len() {
            let end = (start + window_size).min(bytes.len());

            // Adjust to UTF-8 boundary
            let mut adjusted_end = end;
            while adjusted_end < bytes.len() && !is_char_boundary(bytes, adjusted_end) {
                adjusted_end -= 1;
            }

            chunks.push(TextChunk {
                text: text[start..adjusted_end].to_string(),
                start,
                len: adjusted_end - start,
            });

            // Move with overlap
            let next_start = adjusted_end.saturating_sub(overlap);
            if next_start >= bytes.len() {
                break;
            }

            // Ensure next start is at char boundary
            start = next_start;
            while start > 0 && !is_char_boundary(bytes, start) {
                start += 1;
            }
        }

        Ok(chunks)
    }
}

/// Check if position is at a UTF-8 character boundary
fn is_char_boundary(bytes: &[u8], pos: usize) -> bool {
    if pos == 0 || pos >= bytes.len() {
        return true;
    }

    // UTF-8 continuation bytes start with 0b10xxxxxx
    (bytes[pos] & 0b1100_0000) != 0b1000_0000
}

/// Compute prefix sum using Blelloch scan
pub fn prefix_sum<T, F>(values: &[T], identity: T, combine: F) -> Vec<T>
where
    T: Clone + Send + Sync,
    F: Fn(&T, &T) -> T + Send + Sync,
{
    if values.is_empty() {
        return vec![identity];
    }

    let mut result = vec![identity.clone()];
    let mut acc = identity;

    for value in values {
        acc = combine(&acc, value);
        result.push(acc.clone());
    }

    result
}
