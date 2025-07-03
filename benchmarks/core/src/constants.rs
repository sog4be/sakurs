//! Constants for benchmarking
//!
//! This module defines common constants used across benchmarks
//! to avoid magic numbers and improve maintainability.

use std::time::Duration;

/// Standard text sizes for benchmarking
pub mod text_sizes {
    pub const SMALL: usize = 1_000;
    pub const MEDIUM: usize = 10_000;
    pub const LARGE: usize = 100_000;
    pub const HUGE: usize = 1_000_000;

    /// Default text sizes for throughput benchmarks
    pub const THROUGHPUT_SIZES: &[usize] = &[SMALL, MEDIUM, LARGE, HUGE];

    /// Default text sizes for accuracy benchmarks
    pub const ACCURACY_SIZES: &[usize] = &[SMALL, MEDIUM, 50_000];
}

/// Chunk sizes for parallel processing benchmarks
pub const CHUNK_SIZES: &[usize] = &[4_096, 16_384, 65_536, 262_144];

/// Thread counts for scalability benchmarks
pub const THREAD_COUNTS: &[usize] = &[1, 2, 4, 8];

/// Average sentence lengths for different complexity levels
pub mod sentence_lengths {
    /// Simple sentence: "This is a test sentence. "
    pub const SIMPLE: usize = 27;

    /// Medium complexity with abbreviations: "Dr. Smith said this. "
    pub const MEDIUM: usize = 40;

    /// Complex with quotes and numbers
    pub const COMPLEX: usize = 48;
}

/// Benchmark configuration profiles
pub mod bench_profiles {
    use super::*;

    /// Configuration for accuracy benchmarks
    pub const ACCURACY_SAMPLE_SIZE: usize = 50;
    pub const ACCURACY_MEASUREMENT_TIME: Duration = Duration::from_secs(5);
    pub const ACCURACY_WARMUP_TIME: Duration = Duration::from_secs(2);

    /// Configuration for performance benchmarks
    pub const PERFORMANCE_SAMPLE_SIZE: usize = 20;
    pub const PERFORMANCE_MEASUREMENT_TIME: Duration = Duration::from_secs(10);
    pub const PERFORMANCE_WARMUP_TIME: Duration = Duration::from_secs(3);

    /// Configuration for scalability benchmarks
    pub const SCALABILITY_SAMPLE_SIZE: usize = 10;
    pub const SCALABILITY_MEASUREMENT_TIME: Duration = Duration::from_secs(15);
    pub const SCALABILITY_WARMUP_TIME: Duration = Duration::from_secs(5);
}

/// Default parallel processing threshold
pub const DEFAULT_PARALLEL_THRESHOLD: usize = 10_000;
