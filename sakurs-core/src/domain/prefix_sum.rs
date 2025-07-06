//! Parallel prefix-sum implementation for the Δ-Stack Monoid algorithm.
//!
//! This module implements the parallel prefix-sum computation that calculates
//! the cumulative state at the beginning of each chunk, enabling independent
//! boundary candidate evaluation in the reduce phase.

use crate::domain::state::{DeltaEntry, PartialState};
use crate::domain::types::DeltaVec;

/// Represents the cumulative state at the start of a chunk.
#[derive(Debug, Clone)]
pub struct ChunkStartState {
    /// Cumulative delta values at chunk start
    pub cumulative_deltas: DeltaVec,
    /// Global offset of this chunk
    pub global_offset: usize,
}

/// Computes parallel prefix-sum of partial states.
///
/// This implements the classic parallel prefix-sum algorithm that computes
/// cumulative states in O(log n) parallel time with O(n) work.
pub struct PrefixSumComputer;

impl PrefixSumComputer {
    /// Computes the cumulative state at the start of each chunk.
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

        // Sequential implementation for small inputs
        if n <= 4 {
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

            return result;
        }

        // Parallel implementation using work-efficient algorithm
        Self::parallel_prefix_sum(states, &mut result);

        result
    }

    /// Parallel prefix-sum using the work-efficient algorithm.
    fn parallel_prefix_sum(states: &[PartialState], result: &mut [ChunkStartState]) {
        let n = states.len();
        let enclosure_count = states[0].deltas.len();

        // Up-sweep phase (reduction)
        let tree_depth = (n as f64).log2().ceil() as usize;
        let mut tree =
            vec![DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; enclosure_count]); n];
        let mut offsets = vec![0usize; n];

        // Initialize leaf nodes
        for (i, state) in states.iter().enumerate() {
            tree[i] = state.deltas.to_vec().into();
            offsets[i] = state.chunk_length;
        }

        // Up-sweep
        for level in 0..tree_depth {
            let stride = 1 << (level + 1);
            let half_stride = 1 << level;

            // Process this level sequentially to avoid borrow checker issues
            for i in (half_stride..n).step_by(stride) {
                let left_idx = i - half_stride;
                let right_idx = i;

                // Combine deltas
                let left_deltas = tree[left_idx].clone();
                let right_deltas = tree[right_idx].clone();
                for j in 0..enclosure_count {
                    tree[i][j] = DeltaEntry {
                        net: left_deltas[j].net + right_deltas[j].net,
                        min: left_deltas[j]
                            .min
                            .min(left_deltas[j].net + right_deltas[j].min),
                    };
                }

                // Combine offsets
                offsets[i] = offsets[left_idx] + offsets[right_idx];
            }
        }

        // Down-sweep phase
        if n > 0 {
            tree[n - 1] = DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; enclosure_count]);
            offsets[n - 1] = 0;
        }

        for level in (0..tree_depth).rev() {
            let stride = 1 << (level + 1);
            let half_stride = 1 << level;

            // Process this level sequentially
            for i in (half_stride..n).step_by(stride) {
                let left_idx = i - half_stride;
                let right_idx = i;

                // Save current value
                let temp_deltas = tree[right_idx].clone();
                let temp_offset = offsets[right_idx];

                // Update right child
                tree[right_idx] = tree[left_idx].clone();
                offsets[right_idx] = offsets[left_idx];

                // Update left child
                for (j, temp) in temp_deltas.iter().enumerate().take(enclosure_count) {
                    let old = tree[left_idx][j].clone();
                    tree[left_idx][j] = DeltaEntry {
                        net: old.net + temp.net,
                        min: old.min.min(old.net + temp.min),
                    };
                }
                offsets[left_idx] += temp_offset;
            }
        }

        // Copy results
        for (i, chunk_start) in result.iter_mut().enumerate() {
            chunk_start.cumulative_deltas = tree[i].clone();
            chunk_start.global_offset = offsets[i];
        }
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
    use crate::domain::state::BoundaryCandidate;
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
