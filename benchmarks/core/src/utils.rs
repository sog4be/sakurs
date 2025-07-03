//! Common utilities for benchmarking
//!
//! This module provides shared functionality to reduce code duplication
//! across benchmark implementations.

use crate::metrics::{
    calculate_accuracy_metrics, calculate_pk_score, calculate_window_diff, AccuracyMetrics,
};
use sakurs_core::application::{ProcessorConfig, TextProcessor};
use sakurs_core::domain::language::EnglishLanguageRules;
use std::sync::Arc;

/// Create a default text processor with English language rules
pub fn create_default_processor() -> TextProcessor {
    TextProcessor::new(Arc::new(EnglishLanguageRules::new()))
}

/// Create a text processor with custom configuration
pub fn create_processor_with_config(config: ProcessorConfig) -> TextProcessor {
    TextProcessor::with_config(config, Arc::new(EnglishLanguageRules::new()))
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
pub fn extract_boundaries(output: &sakurs_core::application::ProcessingOutput) -> Vec<usize> {
    output.boundaries.iter().map(|b| b.offset).collect()
}
