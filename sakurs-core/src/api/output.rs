//! Output types for unified API

use std::time::Duration;

/// Processing output with rich metadata
#[derive(Debug, Clone)]
pub struct Output {
    /// Sentence boundaries found
    pub boundaries: Vec<Boundary>,
    /// Processing metadata
    pub metadata: ProcessingMetadata,
}

/// A sentence boundary with detailed information
#[derive(Debug, Clone)]
pub struct Boundary {
    /// Byte offset in the original text
    pub offset: usize,
    /// Character offset in the original text
    pub char_offset: usize,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Optional context for debugging
    pub context: Option<BoundaryContext>,
}

/// Context information for a boundary (for debugging)
#[derive(Debug, Clone)]
pub struct BoundaryContext {
    /// Text before the boundary
    pub before: String,
    /// Text after the boundary
    pub after: String,
    /// Reason for the boundary
    pub reason: String,
}

/// Metadata about the processing
#[derive(Debug, Clone)]
pub struct ProcessingMetadata {
    /// Total processing duration
    pub duration: Duration,
    /// Strategy used for processing
    pub strategy_used: String,
    /// Number of chunks processed
    pub chunks_processed: usize,
    /// Peak memory usage in bytes
    pub memory_peak: usize,
    /// Additional statistics
    pub stats: ProcessingStats,
}

/// Additional processing statistics
#[derive(Debug, Clone)]
pub struct ProcessingStats {
    /// Total bytes processed
    pub bytes_processed: usize,
    /// Total characters processed
    pub chars_processed: usize,
    /// Number of sentences found
    pub sentence_count: usize,
    /// Average sentence length in characters
    pub avg_sentence_length: f32,
}

impl Output {
    /// Create output from delta stack processing result
    pub(crate) fn from_delta_stack_result(
        result: crate::application::DeltaStackResult,
        text: &str,
        duration: Duration,
    ) -> Self {
        // Calculate character offsets for each byte boundary (and the total
        // character count, avoiding a second full pass over the text)
        let (char_boundaries, total_chars) = Self::calculate_char_offsets(text, &result.boundaries);

        let boundaries = result
            .boundaries
            .into_iter()
            .zip(char_boundaries)
            .map(|(offset, char_offset)| Boundary {
                offset,
                char_offset,
                confidence: 1.0, // DeltaStack algorithm has high confidence
                context: None,
            })
            .collect::<Vec<_>>();

        let sentence_count = boundaries.len();
        let avg_sentence_length = if sentence_count > 0 {
            total_chars as f32 / sentence_count as f32
        } else {
            0.0
        };

        // Determine strategy used based on thread count
        let strategy_used = if result.thread_count > 1 {
            format!("parallel ({} threads)", result.thread_count)
        } else {
            "sequential".to_string()
        };

        Self {
            boundaries,
            metadata: ProcessingMetadata {
                duration,
                strategy_used,
                chunks_processed: result.chunk_count,
                memory_peak: 0, // Future: memory tracking integration
                stats: ProcessingStats {
                    bytes_processed: text.len(),
                    chars_processed: total_chars,
                    sentence_count,
                    avg_sentence_length,
                },
            },
        }
    }

    /// Calculate character offsets from byte offsets.
    ///
    /// `byte_offsets` must be sorted ascending and lie on character
    /// boundaries (guaranteed by the boundary merge step). Counting each
    /// inter-boundary segment with the standard library's optimized
    /// word-at-a-time counter is much faster than a char-by-char walk.
    /// Returns the character offset for each byte offset plus the total
    /// character count.
    fn calculate_char_offsets(text: &str, byte_offsets: &[usize]) -> (Vec<usize>, usize) {
        let mut char_offsets = Vec::with_capacity(byte_offsets.len());
        let mut chars = 0usize;
        let mut prev = 0usize;
        for &off in byte_offsets {
            chars += text[prev..off].chars().count();
            char_offsets.push(chars);
            prev = off;
        }
        let total = chars + text[prev..].chars().count();
        (char_offsets, total)
    }
}
