//! Benchmark utilities and metrics for sakurs
//!
//! This crate provides common functionality for benchmarking sakurs,
//! including accuracy metrics, test data generation, and comparison utilities.

pub mod data;
pub mod metrics;

pub use data::TestData;
pub use metrics::{calculate_accuracy_metrics, AccuracyMetrics};
