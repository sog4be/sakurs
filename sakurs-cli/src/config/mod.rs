//! Configuration module

use serde::{Deserialize, Serialize};

/// CLI configuration structure
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CliConfig {
    /// Processing configuration
    #[serde(default)]
    pub processing: ProcessingConfig,

    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,

    /// Performance configuration
    #[serde(default)]
    pub performance: PerformanceConfig,
}

/// Processing-related configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessingConfig {
    /// Default language for processing
    pub default_language: String,

    /// Enable abbreviation detection
    pub detect_abbreviations: bool,

    /// Strict punctuation mode
    pub strict_punctuation: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            default_language: "english".to_string(),
            detect_abbreviations: true,
            strict_punctuation: false,
        }
    }
}

/// Output-related configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    /// Default output format
    pub default_format: String,

    /// Include metadata in output
    pub include_metadata: bool,

    /// Pretty print JSON output
    pub pretty_json: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: "text".to_string(),
            include_metadata: false,
            pretty_json: true,
        }
    }
}

/// Performance-related configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct PerformanceConfig {
    /// Threshold for parallel processing (MB)
    pub parallel_threshold_mb: u64,

    /// Chunk size for processing (KB)
    pub chunk_size_kb: u64,

    /// Number of worker threads (0 = auto)
    pub worker_threads: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            parallel_threshold_mb: 10,
            chunk_size_kb: 256,
            worker_threads: 0,
        }
    }
}
