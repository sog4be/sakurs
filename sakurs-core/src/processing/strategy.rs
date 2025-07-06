//! Core traits and types for processing strategies

use crate::application::config::ProcessingResult as Result;

/// Trait for different text processing strategies
pub trait ProcessingStrategy: Send + Sync {
    /// Process text with this strategy
    fn process(&self, text: &str, config: &ProcessingConfig) -> Result<Vec<usize>>;

    /// Estimate if this strategy is suitable for given input
    fn suitability_score(&self, characteristics: &InputCharacteristics) -> f32;

    /// Get optimal configuration for this strategy
    fn optimal_config(&self, characteristics: &InputCharacteristics) -> ProcessingConfig;

    /// Strategy name for debugging and metrics
    fn name(&self) -> &'static str;
}

/// Configuration parameters for processing
#[derive(Debug, Clone)]
pub struct ProcessingConfig {
    /// Size of chunks for parallel processing
    pub chunk_size: usize,
    /// Number of threads for parallel processing
    pub thread_count: usize,
    /// Buffer size for streaming operations
    pub buffer_size: usize,
    /// Prefetch distance for cache optimization
    pub prefetch_distance: usize,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 65536,     // 64KB default
            thread_count: 1,       // Sequential by default
            buffer_size: 4194304,  // 4MB for streaming
            prefetch_distance: 32, // Cache line optimization
        }
    }
}

/// Characteristics of the input text
#[derive(Debug)]
pub struct InputCharacteristics {
    /// Size of input in bytes
    pub size_bytes: usize,
    /// Estimated character count
    pub estimated_char_count: usize,
    /// Whether input is from a stream
    pub is_streaming: bool,
    /// Available system memory in bytes
    pub available_memory: usize,
    /// Number of CPU cores
    pub cpu_count: usize,
}

impl InputCharacteristics {
    /// Analyze text to determine its characteristics
    pub fn from_text(text: &str) -> Self {
        let size_bytes = text.len();
        let estimated_char_count = text.chars().count();

        Self {
            size_bytes,
            estimated_char_count,
            is_streaming: false,
            available_memory: Self::get_available_memory(),
            cpu_count: num_cpus::get(),
        }
    }

    /// Get available system memory (simplified)
    fn get_available_memory() -> usize {
        // Simplified: assume 1GB available
        // In production, would query system
        1_073_741_824
    }

    /// Check if this is a small file
    pub fn is_small(&self) -> bool {
        self.size_bytes < 75_000 // 75KB - optimal threshold for sequential
    }

    /// Check if this is a medium file
    pub fn is_medium(&self) -> bool {
        self.size_bytes >= 75_000 && self.size_bytes < 10_485_760 // 75KB - 10MB
    }

    /// Check if this is a large file
    pub fn is_large(&self) -> bool {
        self.size_bytes >= 10_485_760 && self.size_bytes < 104_857_600 // 10MB - 100MB
    }

    /// Check if this is a very large file
    pub fn is_very_large(&self) -> bool {
        self.size_bytes >= 104_857_600 // 100MB+
    }
}
