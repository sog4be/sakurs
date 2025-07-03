//! Benchmark utilities and metrics for sakurs
//!
//! This crate provides common functionality for benchmarking sakurs,
//! including accuracy metrics, test data generation, and comparison utilities.

pub mod constants;
pub mod data;
pub mod metrics;
pub mod utils;

pub use data::TestData;
pub use metrics::{calculate_accuracy_metrics, AccuracyMetrics};
pub use utils::{
    calculate_complete_metrics, create_default_processor, create_processor_with_config,
    extract_boundaries,
};
