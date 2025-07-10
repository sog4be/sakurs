//! Configuration and error handling for the application layer
//!
//! This module provides configuration options for performance tuning
//! and comprehensive error types for robust error handling.

use thiserror::Error;

/// Configuration options for text processing
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Target size for each chunk in bytes
    pub chunk_size: usize,

    /// Minimum text size to trigger parallel processing
    pub parallel_threshold: usize,

    /// Maximum number of threads to use (None = use all available)
    pub max_threads: Option<usize>,

    /// Size of overlap between chunks for cross-boundary detection
    pub overlap_size: usize,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64 * 1024,           // 64KB chunks
            parallel_threshold: 1024 * 1024, // 1MB threshold for parallel
            max_threads: None,               // Use all available cores
            overlap_size: 256,               // 256 char overlap
        }
    }
}

impl ProcessorConfig {
    /// Creates a new builder for ProcessorConfig
    pub fn builder() -> ProcessorConfigBuilder {
        ProcessorConfigBuilder::new()
    }

    /// Creates a configuration optimized for small texts
    pub fn small_text() -> Self {
        Self {
            chunk_size: 8 * 1024,           // 8KB chunks
            parallel_threshold: usize::MAX, // Never use parallel
            overlap_size: 64,               // Smaller overlap
            ..Default::default()
        }
    }

    /// Creates a configuration optimized for large texts
    pub fn large_text() -> Self {
        Self {
            chunk_size: 256 * 1024,         // 256KB chunks
            parallel_threshold: 512 * 1024, // 512KB threshold
            max_threads: None,              // Use all available cores
            overlap_size: 512,              // Larger overlap
        }
    }

    /// Creates a configuration optimized for streaming
    pub fn streaming() -> Self {
        Self {
            chunk_size: 32 * 1024,          // 32KB chunks
            parallel_threshold: 256 * 1024, // 256KB threshold
            max_threads: Some(2),           // Limited parallelism
            overlap_size: 128,              // Moderate overlap
        }
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), ProcessingError> {
        if self.chunk_size == 0 {
            return Err(ProcessingError::InvalidConfig {
                reason: "Chunk size must be greater than 0".to_string(),
            });
        }

        if self.overlap_size >= self.chunk_size {
            return Err(ProcessingError::InvalidConfig {
                reason: "Overlap size must be less than chunk size".to_string(),
            });
        }

        if let Some(threads) = self.max_threads {
            if threads == 0 {
                return Err(ProcessingError::InvalidConfig {
                    reason: "Max threads must be greater than 0".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Convert to strategies ProcessingConfig
    pub fn to_processing_config(&self) -> crate::application::strategies::ProcessingConfig {
        crate::application::strategies::ProcessingConfig {
            chunk_size: self.chunk_size,
            thread_count: self.max_threads.unwrap_or_else(num_cpus::get),
            buffer_size: self.chunk_size * 4, // 4x chunk size for buffer
            overlap_size: self.overlap_size,
        }
    }
}

/// Errors that can occur during text processing
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// Text exceeds maximum size limit
    #[error("Text too large for processing: {size} bytes (max: {max} bytes)")]
    TextTooLarge { size: usize, max: usize },

    /// Invalid configuration parameters
    #[error("Invalid configuration: {reason}")]
    InvalidConfig { reason: String },

    /// Error during parallel processing
    #[error("Parallel processing failed")]
    ParallelError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// UTF-8 encoding error
    #[error("Invalid UTF-8 in text at position {position}")]
    Utf8Error { position: usize },

    /// Chunk boundary calculation error
    #[error("Failed to calculate chunk boundaries: {reason}")]
    ChunkingError { reason: String },

    /// UTF-8 boundary detection failed
    #[error("Failed to find UTF-8 boundary at position {position}")]
    Utf8BoundaryError { position: usize },

    /// Word boundary detection failed
    #[error("Failed to find word boundary near position {position}")]
    WordBoundaryError { position: usize },

    /// Invalid chunk configuration
    #[error("Invalid chunk boundaries: start={start}, end={end}, next={next}")]
    InvalidChunkBoundaries {
        start: usize,
        end: usize,
        next: usize,
    },

    /// Memory allocation failure
    #[error("Memory allocation failed: {reason}")]
    AllocationError { reason: String },

    /// I/O error (for future file operations)
    #[error("I/O operation failed")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    /// Language rules error
    #[error("Language rules processing failed: {reason}")]
    LanguageRulesError { reason: String },

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for processing operations
pub type ProcessingResult<T> = Result<T, ProcessingError>;

/// Builder for ProcessorConfig with fluent API
#[derive(Debug, Clone)]
pub struct ProcessorConfigBuilder {
    config: ProcessorConfig,
}

impl ProcessorConfigBuilder {
    /// Creates a new builder with default values
    pub fn new() -> Self {
        Self {
            config: ProcessorConfig::default(),
        }
    }

    /// Sets the chunk size in bytes
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.config.chunk_size = size;
        self
    }

    /// Sets the parallel processing threshold
    pub fn parallel_threshold(mut self, threshold: usize) -> Self {
        self.config.parallel_threshold = threshold;
        self
    }

    /// Sets the maximum number of threads
    pub fn max_threads(mut self, threads: Option<usize>) -> Self {
        self.config.max_threads = threads;
        self
    }

    /// Sets the overlap size between chunks
    pub fn overlap_size(mut self, size: usize) -> Self {
        self.config.overlap_size = size;
        self
    }

    /// Builds the configuration, validating parameters
    pub fn build(self) -> ProcessingResult<ProcessorConfig> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Builds the configuration without validation (for testing)
    pub fn build_unchecked(self) -> ProcessorConfig {
        self.config
    }
}

impl Default for ProcessorConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance metrics collected during processing
#[derive(Debug, Clone, Default)]
pub struct ProcessingMetrics {
    /// Total processing time in microseconds
    pub total_time_us: u64,

    /// Time spent in chunking
    pub chunking_time_us: u64,

    /// Time spent in parallel processing
    pub parallel_time_us: u64,

    /// Time spent in result merging
    pub merge_time_us: u64,

    /// Number of chunks processed
    pub chunk_count: usize,

    /// Number of threads used
    pub thread_count: usize,

    /// Total bytes processed
    pub bytes_processed: usize,

    /// Number of boundaries detected
    pub boundaries_found: usize,
}

impl ProcessingMetrics {
    /// Calculates throughput in MB/s
    pub fn throughput_mbps(&self) -> f64 {
        if self.total_time_us == 0 {
            return 0.0;
        }

        let mb = self.bytes_processed as f64 / (1024.0 * 1024.0);
        let seconds = self.total_time_us as f64 / 1_000_000.0;
        mb / seconds
    }

    /// Calculates parallel efficiency (0.0 to 1.0)
    pub fn parallel_efficiency(&self) -> f64 {
        if self.thread_count <= 1 || self.parallel_time_us == 0 {
            return 1.0;
        }

        // Ideal parallel time would be total_time / thread_count
        let ideal_time = self.total_time_us as f64 / self.thread_count as f64;
        let actual_time = self.parallel_time_us as f64;

        (ideal_time / actual_time).min(1.0)
    }
}

/// Thread pool configuration
#[cfg(feature = "parallel")]
#[derive(Debug, Clone)]
pub struct ThreadPoolConfig {
    /// Number of worker threads
    pub num_threads: usize,

    /// Stack size for worker threads (in bytes)
    pub stack_size: Option<usize>,

    /// Thread name prefix
    pub thread_name_prefix: String,
}

#[cfg(feature = "parallel")]
impl Default for ThreadPoolConfig {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get(),
            stack_size: None,
            thread_name_prefix: "sakurs-worker".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ProcessorConfig::default();
        assert_eq!(config.chunk_size, 64 * 1024);
        assert_eq!(config.parallel_threshold, 1024 * 1024);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        // Invalid chunk size
        let config = ProcessorConfig {
            chunk_size: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid overlap size
        let config = ProcessorConfig {
            chunk_size: 1024,
            overlap_size: 2048,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid thread count
        let config = ProcessorConfig {
            max_threads: Some(0),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_preset_configs() {
        let small = ProcessorConfig::small_text();
        assert_eq!(small.chunk_size, 8 * 1024);
        assert_eq!(small.parallel_threshold, usize::MAX);

        let large = ProcessorConfig::large_text();
        assert_eq!(large.chunk_size, 256 * 1024);

        let streaming = ProcessorConfig::streaming();
        assert_eq!(streaming.max_threads, Some(2));
    }

    #[test]
    fn test_processing_metrics() {
        let metrics = ProcessingMetrics {
            bytes_processed: 10 * 1024 * 1024, // 10MB
            total_time_us: 1_000_000,          // 1 second
            ..Default::default()
        };

        assert_eq!(metrics.throughput_mbps(), 10.0);

        let metrics_parallel = ProcessingMetrics {
            bytes_processed: 10 * 1024 * 1024,
            total_time_us: 1_000_000,
            thread_count: 4,
            parallel_time_us: 300_000, // 0.3 seconds
            ..Default::default()
        };
        assert!(metrics_parallel.parallel_efficiency() > 0.8);
    }
}
