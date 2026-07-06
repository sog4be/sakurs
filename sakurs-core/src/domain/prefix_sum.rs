//! Prefix-sum computation for the Δ-Stack Monoid algorithm.
//!
//! This module computes the cumulative state at the beginning of each chunk,
//! enabling independent boundary candidate evaluation in the reduce phase.
//! The scan is sequential: chunk counts are small (text size / chunk size),
//! so an O(n) loop over per-chunk deltas is negligible next to the scan
//! phase and much easier to verify than a tree-based parallel scan.

use crate::application::chunking::TextChunk;
use crate::domain::types::{DeltaEntry, DeltaVec, PartialState};

/// Represents the cumulative state at the start of a chunk.
#[derive(Debug, Clone)]
pub struct ChunkStartState {
    /// Cumulative delta values at chunk start
    pub cumulative_deltas: DeltaVec,
    /// Global offset of this chunk
    pub global_offset: usize,
}

/// Computes prefix sums of partial states.
pub struct PrefixSumComputer;

impl PrefixSumComputer {
    /// Computes the cumulative state at the start of each chunk.
    ///
    /// This version assumes chunks are contiguous without overlap,
    /// using the chunk_length field to calculate offsets.
    /// Used for sequential processing where chunks don't overlap.
    ///
    /// # Arguments
    /// * `states` - Partial states from the scan phase
    ///
    /// # Returns
    /// Vector of cumulative states, one for each input chunk
    pub fn compute_prefix_sum(states: &[PartialState]) -> Vec<ChunkStartState> {
        if states.is_empty() {
            return Vec::new();
        }

        let n = states.len();
        let mut result = vec![
            ChunkStartState {
                cumulative_deltas: DeltaVec::from_vec(vec![
                    DeltaEntry { net: 0, min: 0 };
                    states[0].deltas.len()
                ]),
                global_offset: 0,
            };
            n
        ];

        // Simple sequential scan. The number of chunks is small (text size /
        // chunk size, typically well under a thousand), so an O(n) scan is
        // both faster and easier to verify than a tree-based parallel scan.
        // The previous Blelloch-style implementation produced incorrect
        // cumulative deltas for some chunk counts, silently corrupting
        // boundary decisions for entire chunks.

        let mut cumulative_offset = 0;
        let mut cumulative_deltas =
            DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; states[0].deltas.len()]);

        for (i, state) in states.iter().enumerate() {
            result[i] = ChunkStartState {
                cumulative_deltas: cumulative_deltas.clone(),
                global_offset: cumulative_offset,
            };

            // Update cumulative values for next iteration
            cumulative_offset += state.chunk_length;
            for (j, delta) in state.deltas.iter().enumerate() {
                let old_net = cumulative_deltas[j].net;
                cumulative_deltas[j].net += delta.net;
                cumulative_deltas[j].min = cumulative_deltas[j].min.min(old_net + delta.min);
            }
        }

        result
    }

    /// Computes the cumulative state with proper chunk offset handling.
    ///
    /// This version uses the actual chunk positions from the original text
    /// to correctly handle overlapping chunks in parallel processing.
    /// Required when chunks may overlap (e.g., in parallel strategy).
    ///
    /// # Arguments
    /// * `states` - Partial states from the scan phase
    /// * `chunks` - Original text chunks with position information
    ///
    /// # Returns
    /// Vector of cumulative states with correct global offsets
    pub fn compute_prefix_sum_with_overlap(
        states: &[PartialState],
        chunks: &[TextChunk],
    ) -> Vec<ChunkStartState> {
        if states.is_empty() || chunks.is_empty() {
            return Vec::new();
        }

        assert_eq!(
            states.len(),
            chunks.len(),
            "States and chunks must have the same length"
        );

        let n = states.len();
        let mut result = vec![
            ChunkStartState {
                cumulative_deltas: DeltaVec::from_vec(vec![
                    DeltaEntry { net: 0, min: 0 };
                    states[0].deltas.len()
                ]),
                global_offset: 0,
            };
            n
        ];

        // Simple sequential scan (see compute_prefix_sum for rationale).

        let mut cumulative_deltas =
            DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; states[0].deltas.len()]);

        for (i, (state, chunk)) in states.iter().zip(chunks.iter()).enumerate() {
            result[i] = ChunkStartState {
                cumulative_deltas: cumulative_deltas.clone(),
                global_offset: chunk.start_offset,
            };

            // Update cumulative deltas for next iteration
            for (j, delta) in state.deltas.iter().enumerate() {
                let old_net = cumulative_deltas[j].net;
                cumulative_deltas[j].net += delta.net;
                cumulative_deltas[j].min = cumulative_deltas[j].min.min(old_net + delta.min);
            }
        }

        result
    }

    /// Applies cumulative state to boundary candidates in a chunk.
    ///
    /// # Arguments
    /// * `state` - Partial state with boundary candidates
    /// * `chunk_start` - Cumulative state at chunk start
    ///
    /// # Returns
    /// Updated partial state with globally-adjusted boundary candidates
    pub fn apply_cumulative_state(
        state: &PartialState,
        chunk_start: &ChunkStartState,
    ) -> PartialState {
        let mut result = state.clone();

        // Update boundary candidates with global offsets
        for candidate in &mut result.boundary_candidates {
            candidate.local_offset += chunk_start.global_offset;

            // Adjust local depths to global depths
            for (i, local_depth) in candidate.local_depths.iter_mut().enumerate() {
                *local_depth += chunk_start.cumulative_deltas[i].net;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::BoundaryCandidate;
    use crate::domain::types::{BoundaryVec, DepthVec};
    use crate::domain::BoundaryFlags;

    #[test]
    fn test_empty_prefix_sum() {
        let states: Vec<PartialState> = vec![];
        let result = PrefixSumComputer::compute_prefix_sum(&states);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_chunk_prefix_sum() {
        let state = PartialState {
            boundary_candidates: BoundaryVec::new(),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }]),
            abbreviation: Default::default(),
            chunk_length: 100,
        };

        let result = PrefixSumComputer::compute_prefix_sum(&[state]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].global_offset, 0);
        assert_eq!(result[0].cumulative_deltas[0].net, 0);
    }

    #[test]
    fn test_multiple_chunks_prefix_sum() {
        let states = vec![
            PartialState {
                boundary_candidates: BoundaryVec::new(),
                deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }]),
                abbreviation: Default::default(),
                chunk_length: 100,
            },
            PartialState {
                boundary_candidates: BoundaryVec::new(),
                deltas: DeltaVec::from_vec(vec![DeltaEntry { net: -1, min: -1 }]),
                abbreviation: Default::default(),
                chunk_length: 150,
            },
            PartialState {
                boundary_candidates: BoundaryVec::new(),
                deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 2, min: 0 }]),
                abbreviation: Default::default(),
                chunk_length: 200,
            },
        ];

        let result = PrefixSumComputer::compute_prefix_sum(&states);

        assert_eq!(result.len(), 3);

        // First chunk starts at beginning
        assert_eq!(result[0].global_offset, 0);
        assert_eq!(result[0].cumulative_deltas[0].net, 0);

        // Second chunk starts after first
        assert_eq!(result[1].global_offset, 100);
        assert_eq!(result[1].cumulative_deltas[0].net, 1);

        // Third chunk starts after first two
        assert_eq!(result[2].global_offset, 250);
        assert_eq!(result[2].cumulative_deltas[0].net, 0); // 1 + (-1) = 0
    }

    #[test]
    fn test_apply_cumulative_state() {
        let state = PartialState {
            boundary_candidates: BoundaryVec::from_vec(vec![BoundaryCandidate {
                local_offset: 10,
                local_depths: DepthVec::from_vec(vec![1]),
                flags: BoundaryFlags::WEAK,
            }]),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }]),
            abbreviation: Default::default(),
            chunk_length: 100,
        };

        let chunk_start = ChunkStartState {
            cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 2, min: 0 }]),
            global_offset: 200,
        };

        let result = PrefixSumComputer::apply_cumulative_state(&state, &chunk_start);

        assert_eq!(result.boundary_candidates[0].local_offset, 210); // 10 + 200
        assert_eq!(result.boundary_candidates[0].local_depths[0], 3); // 1 + 2
    }
}
