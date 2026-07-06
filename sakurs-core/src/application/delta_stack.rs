use std::sync::Arc;

use rayon::prelude::*;

use crate::{
    application::{
        chunking::chunk_spans,
        config::{ProcessingError, ProcessingResult, ProcessorConfig},
    },
    domain::language::config::{get_language_config, LanguageConfig},
    domain::state::{
        adjust_for_toggles, rebase_candidate, scan_chunk, Candidate, CandidateVec, CompiledRules,
        PartialState, ToggleVec,
    },
    domain::types::DepthVec,
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

        let thread_count = mode.determine_thread_count(text.len());
        let pool = if thread_count > 1 {
            Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(thread_count)
                    .build()
                    .map_err(|e| ProcessingError::InvalidConfig {
                        reason: format!("Failed to create thread pool: {e}"),
                    })?,
            )
        } else {
            None
        };
        let rules = self.rules.as_ref();

        // Phase 1: scan chunks into partial states (parallel when warranted).
        let mut states: Vec<PartialState> = if let Some(pool) = &pool {
            pool.install(|| {
                chunks
                    .par_iter()
                    .map(|chunk| scan_chunk(chunk, rules))
                    .collect()
            })
        } else {
            chunks
                .iter()
                .map(|chunk| scan_chunk(chunk, rules))
                .collect()
        };

        // Phase 2: prefix fold over aggregates. The confirmed candidates are
        // taken out of the states first, so the sequential fold only touches
        // per-chunk totals, context buffers, and pending items; the bulk is
        // rebased and filtered in parallel below. Seam-resolved candidates
        // surface in the accumulator and are collected as extras; resolved
        // enclosure toggles are routed to every chunk whose prefix was
        // recorded before the toggle was known (later chunks receive them
        // through the cumulative prefix itself).
        let mut bulk: Vec<CandidateVec> = Vec::with_capacity(chunk_count);
        let mut chunk_starts: Vec<usize> = Vec::with_capacity(chunk_count);
        let mut prefix: Vec<(DepthVec, u32)> = Vec::with_capacity(chunk_count);
        let mut toggles_by_chunk: Vec<ToggleVec> = vec![ToggleVec::new(); chunk_count];
        let mut acc = PartialState::identity();
        for (i, state) in states.iter_mut().enumerate() {
            bulk.push(std::mem::take(&mut state.boundaries));
            chunk_starts.push(acc.chunk_len);
            prefix.push((acc.deltas.clone(), acc.parity));
            let toggles = acc.absorb(state, rules);
            assign_toggles(&mut toggles_by_chunk, &chunk_starts, toggles, i);
        }
        // Seam-resolved candidates stay in the accumulator through edge
        // resolution: that is where boundary-of-text enclosure toggles are
        // applied to them (a toggle resolved at a later step always sits
        // after every earlier-confirmed candidate, but BOF/EOF toggles do
        // not).
        let (mut acc, edge_toggles) = acc.resolve_edges_full(rules);
        assign_toggles(
            &mut toggles_by_chunk,
            &chunk_starts,
            edge_toggles,
            chunk_count - 1,
        );
        let extras: Vec<Candidate> = acc.boundaries.drain(..).collect();

        // Phase 3: reduce — rebase each chunk's candidates to text-global
        // coordinates, apply the toggles positioned before them, and keep
        // candidates outside every enclosure: clamped depth for asymmetric
        // types, even parity for symmetric types. Embarrassingly parallel.
        let reduce_chunk = |i: usize| -> Vec<usize> {
            let (deltas, parity) = &prefix[i];
            let toggles = &toggles_by_chunk[i];
            bulk[i]
                .iter()
                .filter_map(|c| {
                    let mut c = rebase_candidate(c, chunk_starts[i], deltas, *parity);
                    adjust_for_toggles(
                        &mut c.local_depths,
                        &mut c.local_parity,
                        c.local_offset,
                        toggles,
                    );
                    is_boundary(&c).then_some(c.local_offset)
                })
                .collect()
        };
        let per_chunk: Vec<Vec<usize>> = if let Some(pool) = &pool {
            pool.install(|| (0..chunk_count).into_par_iter().map(reduce_chunk).collect())
        } else {
            (0..chunk_count).map(reduce_chunk).collect()
        };

        // Merge: per-chunk results are globally ordered by construction; the
        // few seam/edge extras are merged in by offset.
        let mut extra_offsets: Vec<usize> = extras
            .iter()
            .filter(|c| is_boundary(c))
            .map(|c| c.local_offset)
            .collect();
        extra_offsets.sort_unstable();
        let total: usize = per_chunk.iter().map(Vec::len).sum::<usize>() + extra_offsets.len();
        let mut boundaries = Vec::with_capacity(total);
        let mut extras_iter = extra_offsets.into_iter().peekable();
        for chunk_offsets in per_chunk {
            for off in chunk_offsets {
                while extras_iter.peek().is_some_and(|&e| e < off) {
                    boundaries.push(extras_iter.next().unwrap());
                }
                boundaries.push(off);
            }
        }
        boundaries.extend(extras_iter);
        boundaries.dedup();

        Ok(DeltaStackResult {
            boundaries,
            chunk_count,
            thread_count,
        })
    }
}

/// A candidate is a sentence boundary iff it sits outside every enclosure.
fn is_boundary(c: &Candidate) -> bool {
    c.local_parity == 0 && c.local_depths.iter().all(|&d| d <= 0)
}

/// Routes resolved enclosure toggles to the chunks whose recorded prefix
/// predates them: from the chunk containing the toggle through chunk `upto`
/// (the step at which it resolved). Later chunks see the toggle through the
/// cumulative prefix instead.
fn assign_toggles(
    by_chunk: &mut [ToggleVec],
    chunk_starts: &[usize],
    toggles: ToggleVec,
    upto: usize,
) {
    for (q, slot) in toggles {
        let from = chunk_starts[..=upto].partition_point(|&s| s <= q) - 1;
        for list in &mut by_chunk[from..=upto] {
            list.push((q, slot));
        }
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
