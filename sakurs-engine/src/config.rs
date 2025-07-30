//! Configuration types for the engine

use crate::ExecutionMode;

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
    /// Execution mode selector
    pub execution_mode: ExecutionMode,
    /// Chunk sizing policy
    pub chunk_policy: ChunkPolicy,
    /// Number of threads for parallel execution (None = auto)
    pub threads: Option<usize>,
    /// Minimum text size for parallel processing
    pub parallel_threshold: usize,
    /// Adaptive threshold in bytes per core (None = use default 128KB)
    pub adaptive_threshold: Option<usize>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::Adaptive,
            chunk_policy: ChunkPolicy::default(),
            threads: None,
            parallel_threshold: 100_000, // 100KB
            adaptive_threshold: None,    // Use default 128KB per core
        }
    }
}

impl EngineConfig {
    /// Create a streaming configuration
    pub fn streaming() -> Self {
        Self {
            execution_mode: ExecutionMode::Streaming,
            chunk_policy: ChunkPolicy::Streaming {
                window_size: 64 * 1024, // 64KB window
                overlap: 1024,          // 1KB overlap
            },
            threads: Some(1),
            parallel_threshold: usize::MAX, // Never use parallel
            adaptive_threshold: None,
        }
    }

    /// Create a fast configuration optimized for speed
    pub fn fast() -> Self {
        Self {
            execution_mode: ExecutionMode::Adaptive,
            chunk_policy: ChunkPolicy::Fixed { size: 512 * 1024 }, // 512KB chunks
            threads: None,                                         // Use all available
            parallel_threshold: 50_000,                            // 50KB
            adaptive_threshold: Some(256 * 1024), // 256KB per core for faster switching
        }
    }

    /// Create a balanced configuration
    pub fn balanced() -> Self {
        Self::default()
    }
}
