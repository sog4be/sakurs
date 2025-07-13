//! Constants for enhanced chunking configuration

/// Default chunk size in bytes (64KB)
pub const DEFAULT_CHUNK_SIZE: usize = 65536;

/// Default overlap size in bytes for cross-chunk pattern detection
pub const DEFAULT_OVERLAP_SIZE: usize = 32;

/// Default minimum confidence threshold for pattern detection
pub const DEFAULT_MIN_CONFIDENCE: f32 = 0.7;

/// Default window size for boundary deduplication
pub const DEFAULT_DEDUP_WINDOW: usize = 32;

/// Maximum line offset for list item detection
pub const MAX_LIST_ITEM_LINE_OFFSET: usize = 10;

/// Number of characters to look ahead/behind for context
pub const CONTEXT_CHAR_COUNT: usize = 3;

/// Confidence scores for different pattern types
pub mod confidence {
    /// High confidence for contractions with full context
    pub const CONTRACTION_HIGH: f32 = 0.95;

    /// Lower confidence for contractions with partial context
    pub const CONTRACTION_LOW: f32 = 0.8;

    /// Confidence for possessive patterns
    pub const POSSESSIVE: f32 = 0.9;

    /// Confidence for measurement patterns
    pub const MEASUREMENT: f32 = 0.95;

    /// Confidence for list item patterns
    pub const LIST_ITEM: f32 = 0.85;

    /// Default confidence for generic patterns
    pub const GENERIC_PATTERN: f32 = 0.7;
}
