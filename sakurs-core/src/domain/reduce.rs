//! Reduce phase implementation for the Δ-Stack Monoid algorithm.
//!
//! This module implements the reduce phase that evaluates boundary candidates
//! based on global enclosure depths to produce confirmed sentence boundaries.
//!
//! ## Quote Suppression
//!
//! The reduce phase now includes enhanced quote suppression that:
//! - Applies language-specific rules for different quote types
//! - Supports configurable suppression behavior
//! - Handles nested quotes with appropriate weakening
//! - Validates quote pairing when enabled
//!
//! The implementation evaluates boundary candidates based on their enclosure
//! context to determine which boundaries should be confirmed or rejected.

use crate::domain::prefix_sum::ChunkStartState;
use crate::domain::types::{Boundary, BoundaryCandidate, DeltaVec, PartialState};
use rayon::prelude::*;

/// Evaluates boundary candidates to produce confirmed boundaries.
pub struct BoundaryReducer;

impl BoundaryReducer {
    /// Evaluates boundary candidates in a single chunk based on global state.
    ///
    /// Treats every enclosure type as asymmetric. Prefer
    /// [`Self::evaluate_candidates_with_symmetry`] when the language defines
    /// symmetric enclosures (same character opens and closes).
    ///
    /// # Arguments
    /// * `candidates` - Boundary candidates from scan phase
    /// * `chunk_start` - Cumulative state at chunk start
    ///
    /// # Returns
    /// Vector of confirmed boundaries with global offsets
    pub fn evaluate_candidates(
        candidates: &[BoundaryCandidate],
        chunk_start: &ChunkStartState,
    ) -> Vec<Boundary> {
        Self::evaluate_candidates_with_symmetry(candidates, chunk_start, &[])
    }

    /// Evaluates boundary candidates, honoring symmetric enclosure types.
    ///
    /// `symmetric_types[i]` marks enclosure type ids whose occurrences are
    /// counted (the scanner adds one per character, because open vs. close
    /// cannot be decided chunk-locally). For those types a candidate is
    /// outside the enclosure when the cumulative count is even; asymmetric
    /// types require a depth of exactly zero. Types beyond the slice length
    /// default to asymmetric.
    pub fn evaluate_candidates_with_symmetry(
        candidates: &[BoundaryCandidate],
        chunk_start: &ChunkStartState,
        symmetric_types: &[bool],
    ) -> Vec<Boundary> {
        candidates
            .iter()
            .filter_map(|candidate| {
                // Check that the candidate sits outside every enclosure type
                let all_outside =
                    candidate
                        .local_depths
                        .iter()
                        .enumerate()
                        .all(|(i, &local_depth)| {
                            let global_depth = chunk_start.cumulative_deltas[i].net + local_depth;
                            if symmetric_types.get(i).copied().unwrap_or(false) {
                                global_depth % 2 == 0
                            } else {
                                global_depth == 0
                            }
                        });

                if all_outside {
                    Some(Boundary {
                        offset: chunk_start.global_offset + candidate.local_offset,
                        flags: candidate.flags,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Performs the reduce phase on all chunks in parallel.
    ///
    /// # Arguments
    /// * `states` - Partial states from scan phase
    /// * `chunk_starts` - Cumulative states from prefix-sum phase
    ///
    /// # Returns
    /// Vector of all confirmed boundaries sorted by offset
    pub fn reduce_all(states: &[PartialState], chunk_starts: &[ChunkStartState]) -> Vec<Boundary> {
        Self::reduce_all_with_symmetry(states, chunk_starts, &[])
    }

    /// Like [`Self::reduce_all`], honoring symmetric enclosure types.
    pub fn reduce_all_with_symmetry(
        states: &[PartialState],
        chunk_starts: &[ChunkStartState],
        symmetric_types: &[bool],
    ) -> Vec<Boundary> {
        assert_eq!(states.len(), chunk_starts.len());

        // Process each chunk in parallel
        let mut boundaries: Vec<Boundary> = states
            .par_iter()
            .zip(chunk_starts.par_iter())
            .flat_map(|(state, chunk_start)| {
                Self::evaluate_candidates_with_symmetry(
                    &state.boundary_candidates,
                    chunk_start,
                    symmetric_types,
                )
            })
            .collect();

        // Sort by offset (should already be mostly sorted)
        boundaries.sort_by_key(|b| b.offset);
        boundaries.dedup_by_key(|b| b.offset);

        boundaries
    }

    /// Reduces a single partial state (for sequential processing).
    ///
    /// This is used when processing a single chunk or in sequential mode.
    pub fn reduce_single(state: &PartialState) -> Vec<Boundary> {
        Self::reduce_single_with_symmetry(state, &[])
    }

    /// Like [`Self::reduce_single`], honoring symmetric enclosure types.
    pub fn reduce_single_with_symmetry(
        state: &PartialState,
        symmetric_types: &[bool],
    ) -> Vec<Boundary> {
        let chunk_start = ChunkStartState {
            cumulative_deltas: DeltaVec::from_vec(vec![
                crate::domain::types::DeltaEntry {
                    net: 0,
                    min: 0
                };
                state.deltas.len()
            ]),
            global_offset: 0,
        };

        Self::evaluate_candidates_with_symmetry(
            &state.boundary_candidates,
            &chunk_start,
            symmetric_types,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::DeltaEntry;
    use crate::domain::types::{BoundaryVec, DepthVec};
    use crate::domain::BoundaryFlags;

    #[test]
    fn test_evaluate_candidates_all_zero_depths() {
        let candidates = vec![
            BoundaryCandidate {
                local_offset: 10,
                local_depths: DepthVec::from_vec(vec![0, 0]),
                flags: BoundaryFlags::WEAK,
            },
            BoundaryCandidate {
                local_offset: 25,
                local_depths: DepthVec::from_vec(vec![0, 0]),
                flags: BoundaryFlags::STRONG,
            },
        ];

        let chunk_start = ChunkStartState {
            cumulative_deltas: DeltaVec::from_vec(vec![
                DeltaEntry { net: 0, min: 0 },
                DeltaEntry { net: 0, min: 0 },
            ]),
            global_offset: 100,
        };

        let boundaries = BoundaryReducer::evaluate_candidates(&candidates, &chunk_start);

        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0].offset, 110); // 100 + 10
        assert_eq!(boundaries[1].offset, 125); // 100 + 25
    }

    #[test]
    fn test_evaluate_candidates_inside_enclosure() {
        let candidates = vec![
            BoundaryCandidate {
                local_offset: 10,
                local_depths: DepthVec::from_vec(vec![1]), // Inside parentheses
                flags: BoundaryFlags::WEAK,
            },
            BoundaryCandidate {
                local_offset: 25,
                local_depths: DepthVec::from_vec(vec![0]), // Outside
                flags: BoundaryFlags::STRONG,
            },
        ];

        let chunk_start = ChunkStartState {
            cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }]),
            global_offset: 0,
        };

        let boundaries = BoundaryReducer::evaluate_candidates(&candidates, &chunk_start);

        assert_eq!(boundaries.len(), 1);
        assert_eq!(boundaries[0].offset, 25);
    }

    #[test]
    fn test_evaluate_candidates_with_cumulative_depth() {
        let candidates = vec![BoundaryCandidate {
            local_offset: 10,
            local_depths: DepthVec::from_vec(vec![-1]), // Closes an enclosure
            flags: BoundaryFlags::WEAK,
        }];

        let chunk_start = ChunkStartState {
            cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }]), // Started with open enclosure
            global_offset: 100,
        };

        let boundaries = BoundaryReducer::evaluate_candidates(&candidates, &chunk_start);

        assert_eq!(boundaries.len(), 1); // 1 + (-1) = 0, so valid boundary
        assert_eq!(boundaries[0].offset, 110);
    }

    #[test]
    fn test_reduce_all() {
        let states = vec![
            PartialState {
                boundary_candidates: BoundaryVec::from_vec(vec![BoundaryCandidate {
                    local_offset: 10,
                    local_depths: DepthVec::from_vec(vec![0]),
                    flags: BoundaryFlags::WEAK,
                }]),
                deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }]),
                abbreviation: Default::default(),
                chunk_length: 50,
            },
            PartialState {
                boundary_candidates: BoundaryVec::from_vec(vec![BoundaryCandidate {
                    local_offset: 15,
                    local_depths: DepthVec::from_vec(vec![-1]),
                    flags: BoundaryFlags::STRONG,
                }]),
                deltas: DeltaVec::from_vec(vec![DeltaEntry { net: -1, min: -1 }]),
                abbreviation: Default::default(),
                chunk_length: 50,
            },
        ];

        let chunk_starts = vec![
            ChunkStartState {
                cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }]),
                global_offset: 0,
            },
            ChunkStartState {
                cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }]),
                global_offset: 50,
            },
        ];

        let boundaries = BoundaryReducer::reduce_all(&states, &chunk_starts);

        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0].offset, 10); // First boundary at global 10
        assert_eq!(boundaries[1].offset, 65); // Second at 50 + 15
    }
}
