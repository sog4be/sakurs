/// Represents the execution mode for text processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Single-threaded sequential processing
    Sequential,
    /// Multi-threaded parallel processing with optional thread count
    Parallel { threads: Option<usize> },
    /// Adaptive mode that automatically selects the best strategy
    Adaptive,
}

impl ExecutionMode {
    /// Determines the actual number of threads to use based on the mode and text size
    pub fn determine_thread_count(&self, text_len: usize) -> usize {
        match self {
            ExecutionMode::Sequential => 1,
            ExecutionMode::Parallel { threads: Some(n) } => *n,
            ExecutionMode::Parallel { threads: None } | ExecutionMode::Adaptive => {
                Self::calculate_optimal_threads(text_len)
            }
        }
    }

    /// Calculates the optimal number of threads based on text size
    /// This preserves the existing heuristics from UnifiedProcessor
    fn calculate_optimal_threads(text_len: usize) -> usize {
        const MIN_CHUNK_SIZE: usize = 256 * 1024; // 256KB per thread

        if text_len < MIN_CHUNK_SIZE {
            1
        } else {
            let available_parallelism = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);

            let size_based_threads = (text_len / MIN_CHUNK_SIZE).max(1);
            size_based_threads.min(available_parallelism)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_mode() {
        let mode = ExecutionMode::Sequential;
        assert_eq!(mode.determine_thread_count(1_000_000), 1);
    }

    #[test]
    fn test_parallel_mode_with_threads() {
        let mode = ExecutionMode::Parallel { threads: Some(4) };
        assert_eq!(mode.determine_thread_count(1_000_000), 4);
    }

    #[test]
    fn test_adaptive_mode_small_text() {
        let mode = ExecutionMode::Adaptive;
        // Small text should use single thread
        assert_eq!(mode.determine_thread_count(100_000), 1);
    }

    #[test]
    fn test_adaptive_mode_large_text() {
        let mode = ExecutionMode::Adaptive;
        // Large text should use multiple threads
        let thread_count = mode.determine_thread_count(10_000_000);
        assert!(thread_count > 1);
    }
}
