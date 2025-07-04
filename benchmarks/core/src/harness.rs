//! Common benchmark harness utilities
//!
//! This module provides shared configuration and utilities for Criterion benchmarks
//! to reduce code duplication across different benchmark files.

use criterion::{Criterion, Throughput};
use std::time::Duration;

/// Configure a Criterion benchmark group with standard settings
pub fn configure_criterion() -> Criterion {
    Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .sample_size(50)
        .noise_threshold(0.05)
}

/// Standard benchmark group configuration
pub struct BenchmarkConfig {
    pub warm_up_time: Duration,
    pub measurement_time: Duration,
    pub sample_size: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            warm_up_time: Duration::from_secs(1),
            measurement_time: Duration::from_secs(3),
            sample_size: 50,
        }
    }
}

impl BenchmarkConfig {
    /// Create a configuration for quick benchmarks
    pub fn quick() -> Self {
        Self {
            warm_up_time: Duration::from_millis(500),
            measurement_time: Duration::from_secs(1),
            sample_size: 20,
        }
    }

    /// Create a configuration for thorough benchmarks
    pub fn thorough() -> Self {
        Self {
            warm_up_time: Duration::from_secs(3),
            measurement_time: Duration::from_secs(10),
            sample_size: 100,
        }
    }
}

/// Setup throughput measurement for text-based benchmarks
pub fn setup_throughput(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    text_len: usize,
) {
    group.throughput(Throughput::Bytes(text_len as u64));
}

/// Common benchmark ID formatting
pub fn format_benchmark_id(name: &str, variant: &str) -> String {
    format!("{}/{}", name, variant)
}

/// Helper to run a benchmark with error handling
pub fn run_benchmark_with_fallback<F, G>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    id: &str,
    primary_fn: F,
    fallback_fn: G,
) where
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
    G: Fn(&mut criterion::Bencher<'_>),
{
    match primary_fn() {
        Ok(()) => {
            // Primary benchmark succeeded
        }
        Err(e) => {
            eprintln!("Warning: {} - using fallback: {}", id, e);
            group.bench_function(id, fallback_fn);
        }
    }
}
