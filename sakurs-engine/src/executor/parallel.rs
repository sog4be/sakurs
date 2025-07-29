//! Parallel execution strategy

use crate::{
    chunker::ChunkManager,
    config::ChunkPolicy,
    error::{EngineError, Result},
    executor::{ExecutionMode, Executor},
};
use rayon::prelude::*;
use sakurs_delta_core::{emit_push, Boundary, DeltaScanner, DeltaVec, LanguageRules, PartialState};

/// Parallel multi-threaded executor
#[derive(Debug)]
pub struct ParallelExecutor {
    chunk_manager: ChunkManager,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(chunk_policy: ChunkPolicy) -> Self {
        Self {
            chunk_manager: ChunkManager::new(chunk_policy),
        }
    }

    /// Run the three-phase parallel algorithm
    fn process_parallel<R: LanguageRules>(&self, text: &str, rules: &R) -> Result<Vec<Boundary>> {
        // Phase 1: Chunk the text
        let chunks = self.chunk_manager.chunk_text(text)?;
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        // Phase 2: Scan chunks in parallel
        let partial_states: Vec<PartialState> = chunks
            .par_iter()
            .map(|chunk| -> Result<PartialState> {
                let mut boundaries = Vec::new();
                let mut scanner = DeltaScanner::new(rules).map_err(EngineError::Core)?;

                for ch in chunk.text.chars() {
                    scanner
                        .step(ch, &mut emit_push(&mut boundaries))
                        .map_err(EngineError::Core)?;
                }

                let mut state = scanner.finish();
                state.boundaries = boundaries;
                Ok(state)
            })
            .collect::<Result<Vec<_>>>()?;

        // Phase 3: Compute prefix sums
        let prefix_deltas = compute_prefix_sum(&partial_states)?;

        // Phase 4: Reduce with depth information
        let boundaries = reduce_with_depths(&chunks, &partial_states, &prefix_deltas, rules)?;

        Ok(boundaries)
    }
}

impl Executor for ParallelExecutor {
    fn process<R: LanguageRules>(&self, text: &str, rules: &R) -> Result<Vec<Boundary>> {
        self.process_parallel(text, rules)
    }

    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Parallel
    }
}

/// Compute prefix sum of delta vectors
fn compute_prefix_sum(states: &[PartialState]) -> Result<Vec<DeltaVec>> {
    if states.is_empty() {
        return Ok(Vec::new());
    }

    let enclosure_count = states[0].deltas.len;
    let mut prefix_sums = Vec::with_capacity(states.len() + 1);

    // Start with identity
    prefix_sums.push(DeltaVec::new(enclosure_count)?);

    // Compute cumulative sums
    for state in states {
        let last = &prefix_sums[prefix_sums.len() - 1];
        let combined = last.combine(&state.deltas)?;
        prefix_sums.push(combined);
    }

    Ok(prefix_sums)
}

/// Reduce boundaries with depth information
fn reduce_with_depths<R: LanguageRules>(
    chunks: &[crate::chunker::TextChunk],
    states: &[PartialState],
    prefix_deltas: &[DeltaVec],
    rules: &R,
) -> Result<Vec<Boundary>> {
    let mut all_boundaries = Vec::new();

    // Process each chunk's boundaries
    for (i, (chunk, state)) in chunks.iter().zip(states.iter()).enumerate() {
        let chunk_start_deltas = &prefix_deltas[i];

        // Check each boundary candidate
        for boundary in &state.boundaries {
            // Calculate depth at this boundary
            let mut at_depth_zero = true;

            for j in 0..rules.max_enclosure_pairs() {
                if let Some((net, _)) = chunk_start_deltas.get(j) {
                    if net != 0 {
                        at_depth_zero = false;
                        break;
                    }
                }
            }

            // Only keep boundaries at depth 0
            if at_depth_zero {
                all_boundaries.push(Boundary {
                    byte_offset: chunk.start + boundary.byte_offset,
                    char_offset: boundary.char_offset, // Will need recalculation
                    kind: boundary.kind,
                });
            }
        }
    }

    // Handle cross-chunk abbreviations
    handle_cross_chunk_abbreviations(&mut all_boundaries, chunks, states);

    Ok(all_boundaries)
}

/// Handle abbreviations that span chunk boundaries
fn handle_cross_chunk_abbreviations(
    boundaries: &mut Vec<Boundary>,
    chunks: &[crate::chunker::TextChunk],
    states: &[PartialState],
) {
    // Check adjacent chunks for dangling dot + head alpha pattern
    for i in 0..states.len() - 1 {
        if states[i].dangling_dot && states[i + 1].head_alpha {
            // Find and remove the false boundary at chunk boundary
            let boundary_pos = chunks[i].start + chunks[i].text.len();
            boundaries.retain(|b| b.byte_offset != boundary_pos);
        }
    }

    // Sort boundaries by offset
    boundaries.sort_by_key(|b| b.byte_offset);
}
