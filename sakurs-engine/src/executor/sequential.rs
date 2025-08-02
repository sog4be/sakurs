//! Sequential execution strategy

use crate::{
    error::Result,
    executor::{ExecutionMetrics, ExecutionMode, Executor, ProcessingOutput},
};
use sakurs_core::{emit_push, DeltaScanner, LanguageRules};
use std::time::Instant;

/// Sequential single-threaded executor
#[derive(Debug, Clone)]
pub struct SequentialExecutor;

impl Executor for SequentialExecutor {
    fn process_with_metadata<R: LanguageRules>(
        &self,
        text: &str,
        rules: &R,
    ) -> Result<ProcessingOutput> {
        let start_time = Instant::now();
        let mut boundaries = Vec::new();
        let mut scanner = DeltaScanner::with_text(rules, text)?;

        // Process each character
        for ch in text.chars() {
            scanner.step(ch, &mut emit_push(&mut boundaries))?;
        }

        let processing_time = start_time.elapsed();
        let bytes_processed = text.len();
        let bytes_per_second = bytes_processed as f64 / processing_time.as_secs_f64();

        let metadata = ExecutionMetrics {
            mode_used: ExecutionMode::Sequential,
            chunks_processed: 1,
            bytes_per_second,
            thread_efficiency: 1.0, // Sequential processing is 100% efficient
            processing_time,
            bytes_processed,
        };

        Ok(ProcessingOutput {
            boundaries,
            metadata,
        })
    }

    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Sequential
    }
}
