use std::sync::Arc;

use rayon::prelude::*;

use crate::{
    application::{
        chunking::chunk_spans,
        config::{ProcessingError, ProcessingResult, ProcessorConfig},
    },
    domain::language::config::{get_language_config, LanguageConfig},
    domain::state::{scan_chunk, CompiledRules, PartialState},
};

use super::execution_mode::ExecutionMode;

/// Result of delta-stack processing with metadata
pub struct DeltaStackResult {
    pub boundaries: Vec<usize>,
    pub chunk_count: usize,
    pub thread_count: usize,
}

/// Core implementation of the Δ-Stack Monoid algorithm
///
/// Runs the three phases described in `docs/DELTA_STACK_ALGORITHM.md`:
/// 1. Scan: chunks are scanned in parallel into partial states
/// 2. Prefix: the states are folded left-to-right, accumulating depth/parity
///    and resolving pending items from neighboring context, then the text
///    edges are resolved
/// 3. Reduce: candidates outside every enclosure become boundaries
pub struct DeltaStackProcessor {
    rules: Arc<CompiledRules>,
    chunk_size: usize,
}

impl DeltaStackProcessor {
    /// Creates a processor for an embedded language code (e.g. "en", "ja").
    pub fn from_language_code(
        config: ProcessorConfig,
        code: &str,
    ) -> Result<Self, ProcessingError> {
        let language = get_language_config(code).map_err(|e| ProcessingError::InvalidConfig {
            reason: e.to_string(),
        })?;
        Self::from_language_config(config, language)
    }

    /// Creates a processor from a language configuration (embedded or
    /// loaded from an external TOML file).
    pub fn from_language_config(
        config: ProcessorConfig,
        language: &LanguageConfig,
    ) -> Result<Self, ProcessingError> {
        let rules =
            CompiledRules::from_config(language).map_err(|e| ProcessingError::InvalidConfig {
                reason: e.to_string(),
            })?;
        Ok(Self {
            rules: Arc::new(rules),
            chunk_size: config.chunk_size,
        })
    }

    /// Main processing method that executes the Δ-Stack Monoid algorithm
    pub fn process(&self, text: &str, mode: ExecutionMode) -> ProcessingResult<DeltaStackResult> {
        if text.is_empty() {
            return Ok(DeltaStackResult {
                boundaries: Vec::new(),
                chunk_count: 0,
                thread_count: 1,
            });
        }

        let chunks = chunk_spans(text, self.chunk_size);
        let chunk_count = chunks.len();

        // Phase 1: scan chunks into partial states (parallel when warranted).
        let thread_count = mode.determine_thread_count(text.len());
        let states: Vec<PartialState> = if thread_count > 1 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build()
                .map_err(|e| ProcessingError::InvalidConfig {
                    reason: format!("Failed to create thread pool: {e}"),
                })?;
            let rules = &self.rules;
            pool.install(|| {
                chunks
                    .par_iter()
                    .map(|chunk| scan_chunk(chunk, rules))
                    .collect()
            })
        } else {
            chunks
                .iter()
                .map(|chunk| scan_chunk(chunk, self.rules.as_ref()))
                .collect()
        };

        // Phase 2: prefix fold. Sequential over per-chunk states (the number
        // of chunks is small and each step only touches the new state's
        // items), then edge resolution with the knowledge that the text ends.
        let mut acc = PartialState::identity();
        for state in &states {
            acc.absorb(state, self.rules.as_ref());
        }
        let acc = acc.resolve_edges(self.rules.as_ref());

        // Phase 3: reduce. A candidate is a boundary iff it sits outside
        // every enclosure: clamped depth for asymmetric types, even parity
        // for symmetric types (offsets are text-global here, so the
        // cumulative prefix is zero).
        let boundaries: Vec<usize> = acc
            .boundaries
            .iter()
            .filter(|c| c.local_parity == 0 && c.local_depths.iter().all(|&d| d <= 0))
            .map(|c| c.local_offset)
            .collect();

        Ok(DeltaStackResult {
            boundaries,
            chunk_count,
            thread_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_processor() -> DeltaStackProcessor {
        DeltaStackProcessor::from_language_code(ProcessorConfig::default(), "en").unwrap()
    }

    #[test]
    fn test_empty_text() {
        let processor = create_test_processor();
        let result = processor.process("", ExecutionMode::Sequential).unwrap();
        assert!(result.boundaries.is_empty());
        assert_eq!(result.chunk_count, 0);
        assert_eq!(result.thread_count, 1);
    }

    #[test]
    fn test_single_sentence() {
        let processor = create_test_processor();
        let text = "This is a sentence.";
        let result = processor.process(text, ExecutionMode::Sequential).unwrap();
        assert_eq!(result.boundaries.len(), 1);
        assert_eq!(result.boundaries[0], 19); // Position after the period
        assert_eq!(result.chunk_count, 1);
        assert_eq!(result.thread_count, 1);
    }

    #[test]
    fn test_parallel_vs_sequential() {
        let processor = create_test_processor();
        let text = "First sentence. Second sentence. Third sentence.";

        let seq_result = processor.process(text, ExecutionMode::Sequential).unwrap();
        let par_result = processor
            .process(text, ExecutionMode::Parallel { threads: Some(2) })
            .unwrap();

        assert_eq!(seq_result.boundaries, par_result.boundaries);
        assert_eq!(seq_result.chunk_count, par_result.chunk_count);
        assert_eq!(seq_result.thread_count, 1);
        assert_eq!(par_result.thread_count, 2);
    }

    #[test]
    fn test_unknown_language_code() {
        let err = DeltaStackProcessor::from_language_code(ProcessorConfig::default(), "zz");
        assert!(err.is_err());
    }
}
