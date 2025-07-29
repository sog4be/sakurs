//! Configuration types for the engine

/// Chunking policy for text processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkPolicy {
    /// Fixed chunk size in bytes
    Fixed {
        /// Size of each chunk in bytes
        size: usize,
    },
    /// Automatic sizing based on L2 cache
    Auto {
        /// Target size for each chunk in bytes
        target_bytes: usize,
    },
    /// Streaming with overlap window
    Streaming {
        /// Size of the streaming window in bytes
        window_size: usize,
        /// Overlap size between chunks in bytes
        overlap: usize,
    },
}

impl Default for ChunkPolicy {
    fn default() -> Self {
        ChunkPolicy::Auto {
            target_bytes: 256 * 1024,
        } // 256KB default
    }
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Chunk sizing policy
    pub chunk_policy: ChunkPolicy,
    /// Number of threads for parallel execution (None = auto)
    pub threads: Option<usize>,
    /// Minimum text size for parallel processing
    pub parallel_threshold: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            chunk_policy: ChunkPolicy::default(),
            threads: None,
            parallel_threshold: 100_000, // 100KB
        }
    }
}

impl EngineConfig {
    /// Create a streaming configuration
    pub fn streaming() -> Self {
        Self {
            chunk_policy: ChunkPolicy::Streaming {
                window_size: 64 * 1024, // 64KB window
                overlap: 1024,          // 1KB overlap
            },
            threads: Some(1),
            parallel_threshold: usize::MAX, // Never use parallel
        }
    }

    /// Create a fast configuration optimized for speed
    pub fn fast() -> Self {
        Self {
            chunk_policy: ChunkPolicy::Fixed { size: 512 * 1024 }, // 512KB chunks
            threads: None,                                         // Use all available
            parallel_threshold: 50_000,                            // 50KB
        }
    }

    /// Create a balanced configuration
    pub fn balanced() -> Self {
        Self::default()
    }
}
