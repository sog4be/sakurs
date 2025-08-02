//! Adaptive execution dispatcher
//!
//! Automatically selects the optimal execution mode based on input characteristics
//! and system resources, as specified in DESIGN.md Section 3.2.

use crate::{
    config::EngineConfig,
    error::Result,
    executor::{
        auto_select, ExecutionMode, Executor, ProcessingOutput, SequentialExecutor,
        StreamingExecutor,
    },
};
use sakurs_core::LanguageRules;
use std::time::Instant;

#[cfg(feature = "parallel")]
use crate::executor::ParallelExecutor;

/// Adaptive execution dispatcher
///
/// Automatically selects between sequential and parallel execution modes
/// based on input size and available CPU resources.
pub struct AdaptiveDispatcher {
    sequential_executor: SequentialExecutor,
    #[cfg(feature = "parallel")]
    parallel_executor: ParallelExecutor,
    streaming_executor: StreamingExecutor,
    config: EngineConfig,
}

impl AdaptiveDispatcher {
    /// Create a new adaptive dispatcher with the given configuration
    pub fn new(config: EngineConfig) -> Self {
        Self {
            sequential_executor: SequentialExecutor,
            #[cfg(feature = "parallel")]
            parallel_executor: ParallelExecutor::new(config.chunk_policy),
            streaming_executor: StreamingExecutor::new(64 * 1024, 1024), // Default streaming config
            config,
        }
    }

    /// Select the optimal execution mode for the given input
    ///
    /// Implements the adaptive logic specified in DESIGN.md Section 6.1:
    /// - Sequential if bytes_per_core < 128KB AND input_size < 512KB
    /// - Parallel otherwise
    pub fn select_mode(&self, input_size: usize) -> ExecutionMode {
        auto_select(input_size, &self.config)
    }

    /// Process text with automatic mode selection
    pub fn process_adaptive<R: LanguageRules>(
        &self,
        text: &str,
        rules: &R,
    ) -> Result<ProcessingOutput> {
        let mode = if matches!(
            self.config.chunk_policy,
            crate::config::ChunkPolicy::Streaming { .. }
        ) {
            ExecutionMode::Streaming
        } else {
            self.select_mode(text.len())
        };

        self.process_with_mode(text, rules, mode)
    }

    /// Process text with the specified execution mode
    pub fn process_with_mode<R: LanguageRules>(
        &self,
        text: &str,
        rules: &R,
        mode: ExecutionMode,
    ) -> Result<ProcessingOutput> {
        let start_time = Instant::now();

        let result = match mode {
            ExecutionMode::Sequential => {
                self.sequential_executor.process_with_metadata(text, rules)
            }
            ExecutionMode::Streaming => self.streaming_executor.process_with_metadata(text, rules),
            ExecutionMode::Adaptive => {
                // Adaptive mode selects between Sequential and Parallel
                let selected_mode = self.select_mode(text.len());
                return self.process_with_mode(text, rules, selected_mode);
            }
            #[cfg(feature = "parallel")]
            ExecutionMode::Parallel => self.parallel_executor.process_with_metadata(text, rules),
            #[cfg(not(feature = "parallel"))]
            ExecutionMode::Parallel => {
                // Fallback to sequential if parallel is not available
                self.sequential_executor.process_with_metadata(text, rules)
            }
        };

        // Update timing information in the result
        result.map(|mut output| {
            output.metadata.processing_time = start_time.elapsed();
            output.metadata.bytes_processed = text.len();
            output.metadata.bytes_per_second =
                text.len() as f64 / output.metadata.processing_time.as_secs_f64();
            output
        })
    }
}

impl Executor for AdaptiveDispatcher {
    fn process_with_metadata<R: LanguageRules>(
        &self,
        text: &str,
        rules: &R,
    ) -> Result<ProcessingOutput> {
        self.process_adaptive(text, rules)
    }

    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Adaptive
    }
}
