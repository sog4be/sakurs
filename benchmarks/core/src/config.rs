//! Benchmark configuration constants and utilities
//!
//! This module centralizes configuration values used throughout the benchmark suite
//! to avoid magic numbers and make the code more maintainable.

use std::env;

/// Default number of warmup runs before actual benchmarking
pub const DEFAULT_WARMUP_RUNS: usize = 3;

/// Standard subset sizes for incremental benchmarking
pub const STANDARD_SUBSET_SIZES: &[usize] = &[100, 1000, 5000];

/// Small subset sizes for quick tests
pub const SMALL_SUBSET_SIZES: &[usize] = &[100, 1000];

/// Timeout for subprocess operations in milliseconds
pub const SUBPROCESS_TIMEOUT_MS: u64 = 60_000; // 1 minute

/// Default chunk size for parallel processing
pub const DEFAULT_CHUNK_SIZE: usize = 50_000;

/// Minimum text size to trigger parallel processing
pub const PARALLEL_THRESHOLD: usize = 100_000;

/// Get the number of warmup runs from environment or use default
pub fn get_warmup_runs() -> usize {
    env::var("SAKURS_BENCHMARK_WARMUP_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_WARMUP_RUNS)
}

/// Get subset sizes from environment or use defaults
pub fn get_subset_sizes(use_small: bool) -> Vec<usize> {
    if let Ok(sizes_str) = env::var("SAKURS_BENCHMARK_SUBSET_SIZES") {
        sizes_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect()
    } else if use_small {
        SMALL_SUBSET_SIZES.to_vec()
    } else {
        STANDARD_SUBSET_SIZES.to_vec()
    }
}

/// Get subprocess timeout from environment or use default
pub fn get_subprocess_timeout_ms() -> u64 {
    env::var("SAKURS_SUBPROCESS_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(SUBPROCESS_TIMEOUT_MS)
}

/// Configuration for benchmark runs
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub warmup_runs: usize,
    pub subset_sizes: Vec<usize>,
    pub subprocess_timeout_ms: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warmup_runs: get_warmup_runs(),
            subset_sizes: get_subset_sizes(false),
            subprocess_timeout_ms: get_subprocess_timeout_ms(),
        }
    }
}

impl BenchmarkConfig {
    /// Create a configuration for quick tests
    pub fn quick() -> Self {
        Self {
            warmup_runs: 1,
            subset_sizes: vec![100],
            subprocess_timeout_ms: 10_000,
        }
    }

    /// Create a configuration for comprehensive benchmarks
    pub fn comprehensive() -> Self {
        Self {
            warmup_runs: 5,
            subset_sizes: vec![100, 500, 1000, 2500, 5000, 10000],
            subprocess_timeout_ms: 120_000,
        }
    }
}
