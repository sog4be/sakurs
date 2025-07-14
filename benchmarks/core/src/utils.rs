//! Common utilities for benchmarking
//!
//! This module provides shared functionality to reduce code duplication
//! across benchmark implementations.

use crate::metrics::{
    calculate_accuracy_metrics, calculate_pk_score, calculate_window_diff, AccuracyMetrics,
};
use sakurs_core::{Config, SentenceProcessor};

/// Create a default sentence processor with English language rules
pub fn create_default_processor() -> SentenceProcessor {
    SentenceProcessor::new()
}

/// Create a sentence processor with custom configuration
pub fn create_processor_with_config(
    chunk_size: usize,
    threads: Option<usize>,
) -> SentenceProcessor {
    let config = Config::builder()
        .language("en")
        .unwrap()
        .chunk_size(chunk_size)
        .threads(threads)
        .build()
        .unwrap();
    SentenceProcessor::with_config(config).unwrap()
}

/// Calculate complete accuracy metrics including F1, Pk, and WindowDiff
pub fn calculate_complete_metrics(
    predicted: &[usize],
    actual: &[usize],
    text_length: usize,
) -> AccuracyMetrics {
    let metrics = calculate_accuracy_metrics(predicted, actual);
    let pk = calculate_pk_score(predicted, actual, text_length, None);
    let wd = calculate_window_diff(predicted, actual, text_length, None);
    metrics.with_pk_score(pk).with_window_diff(wd)
}

/// Extract boundary positions from processing output
pub fn extract_boundaries(output: &sakurs_core::Output) -> Vec<usize> {
    output.boundaries.iter().map(|b| b.offset).collect()
}
