//! Streaming execution strategy

use crate::{
    error::Result,
    executor::{ExecutionMode, Executor, ProcessingOutput, SequentialExecutor},
};
use sakurs_core::{Boundary, LanguageRules};
use std::io::Read;
use std::time::Instant;

/// Streaming executor for memory-efficient processing
#[derive(Debug)]
pub struct StreamingExecutor {
    #[allow(dead_code)]
    window_size: usize,
    #[allow(dead_code)]
    overlap: usize,
}

impl StreamingExecutor {
    /// Create a new streaming executor
    pub fn new(window_size: usize, overlap: usize) -> Self {
        Self {
            window_size,
            overlap,
        }
    }
}

impl Executor for StreamingExecutor {
    fn process_with_metadata<R: LanguageRules>(
        &self,
        text: &str,
        rules: &R,
    ) -> Result<ProcessingOutput> {
        let start_time = Instant::now();

        // For now, delegate to sequential executor
        // TODO: Implement true streaming with overlap windows
        let sequential = SequentialExecutor;
        let mut output = sequential.process_with_metadata(text, rules)?;

        // Update metadata to reflect streaming mode
        output.metadata.mode_used = ExecutionMode::Streaming;
        output.metadata.processing_time = start_time.elapsed();
        output.metadata.bytes_per_second =
            text.len() as f64 / output.metadata.processing_time.as_secs_f64();

        Ok(output)
    }

    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Streaming
    }
}

/// Process a reader in streaming fashion
pub fn process_reader<R: Read, L: LanguageRules>(
    reader: R,
    rules: &L,
    window_size: usize,
    overlap: usize,
) -> Result<Vec<Boundary>> {
    // TODO: Implement actual streaming from reader
    // For now, read all and process
    let mut buffer = String::new();
    let mut reader = reader;
    reader.read_to_string(&mut buffer)?;

    let executor = StreamingExecutor::new(window_size, overlap);
    executor.process(&buffer, rules)
}
