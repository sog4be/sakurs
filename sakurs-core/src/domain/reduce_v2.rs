//! Enhanced reduce phase with language-aware quote suppression
//!
//! This module provides an enhanced boundary reducer that integrates
//! the quote suppression logic for more intelligent boundary evaluation.

use crate::domain::{
    language::LanguageRules,
    prefix_sum::ChunkStartState,
    quote_suppression::{
        QuoteSuppressionConfig, QuoteSuppressionContext, QuoteSuppressor, SuppressionDecision,
    },
    state::{Boundary, BoundaryCandidate, PartialState},
};
use rayon::prelude::*;
use std::sync::Arc;

/// Enhanced boundary reducer with quote suppression support
pub struct BoundaryReducerV2 {
    /// Language rules for context
    language_rules: Arc<dyn LanguageRules>,
    /// Quote suppression configuration
    suppression_config: QuoteSuppressionConfig,
}

impl BoundaryReducerV2 {
    /// Create a new enhanced reducer
    pub fn new(language_rules: Arc<dyn LanguageRules>) -> Self {
        Self {
            language_rules,
            suppression_config: QuoteSuppressionConfig::default(),
        }
    }

    /// Create with custom suppression configuration
    pub fn with_config(
        language_rules: Arc<dyn LanguageRules>,
        suppression_config: QuoteSuppressionConfig,
    ) -> Self {
        Self {
            language_rules,
            suppression_config,
        }
    }

    /// Evaluate boundary candidates with language-aware quote suppression
    pub fn evaluate_candidates(
        &self,
        candidates: &[BoundaryCandidate],
        chunk_start: &ChunkStartState,
    ) -> Vec<Boundary> {
        candidates
            .iter()
            .filter_map(|candidate| {
                // Calculate global depths
                let global_depths: Vec<i32> = candidate
                    .local_depths
                    .iter()
                    .enumerate()
                    .map(|(i, &local_depth)| chunk_start.cumulative_deltas[i].net + local_depth)
                    .collect();

                // Create suppression context
                let context = QuoteSuppressionContext {
                    candidate,
                    language_rules: self.language_rules.as_ref(),
                    enclosure_depths: &global_depths,
                    config: &self.suppression_config,
                };

                // Evaluate suppression decision
                match QuoteSuppressor::evaluate(context) {
                    SuppressionDecision::Keep => {
                        // Keep the boundary as-is
                        Some(Boundary {
                            offset: chunk_start.global_offset + candidate.local_offset,
                            flags: candidate.flags,
                        })
                    }
                    SuppressionDecision::Weaken { new_flags } => {
                        // Keep but weaken the boundary
                        Some(Boundary {
                            offset: chunk_start.global_offset + candidate.local_offset,
                            flags: new_flags,
                        })
                    }
                    SuppressionDecision::Suppress { .. } => {
                        // Suppress this boundary
                        None
                    }
                }
            })
            .collect()
    }

    /// Reduce all chunks in parallel with enhanced suppression
    pub fn reduce_all(
        &self,
        states: &[PartialState],
        chunk_starts: &[ChunkStartState],
    ) -> Vec<Boundary> {
        assert_eq!(states.len(), chunk_starts.len());

        // Process each chunk in parallel
        let mut boundaries: Vec<Boundary> = states
            .par_iter()
            .zip(chunk_starts.par_iter())
            .flat_map(|(state, chunk_start)| {
                self.evaluate_candidates(&state.boundary_candidates, chunk_start)
            })
            .collect();

        // Sort by offset and deduplicate
        boundaries.sort_by_key(|b| b.offset);
        boundaries.dedup_by_key(|b| b.offset);

        boundaries
    }

    /// Reduce a single partial state (for sequential processing)
    pub fn reduce_single(&self, state: &PartialState) -> Vec<Boundary> {
        let chunk_start = ChunkStartState {
            cumulative_deltas: crate::domain::types::DeltaVec::from_vec(vec![
                crate::domain::state::DeltaEntry {
                    net: 0,
                    min: 0
                };
                state.deltas.len()
            ]),
            global_offset: 0,
        };

        self.evaluate_candidates(&state.boundary_candidates, &chunk_start)
    }

    /// Configure quote suppression behavior
    pub fn set_suppression_config(&mut self, config: QuoteSuppressionConfig) {
        self.suppression_config = config;
    }

    /// Get current suppression configuration
    pub fn suppression_config(&self) -> &QuoteSuppressionConfig {
        &self.suppression_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        language::MockLanguageRules, state::DeltaEntry, types::DepthVec, BoundaryFlags,
    };

    #[test]
    fn test_enhanced_quote_suppression() {
        let rules = Arc::new(MockLanguageRules::english());
        let reducer = BoundaryReducerV2::new(rules);

        let candidates = vec![
            BoundaryCandidate {
                local_offset: 10,
                local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]), // Outside quotes
                flags: BoundaryFlags::STRONG,
            },
            BoundaryCandidate {
                local_offset: 25,
                local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]), // Inside double quotes
                flags: BoundaryFlags::STRONG,
            },
        ];

        let chunk_start = ChunkStartState {
            cumulative_deltas: crate::domain::types::DeltaVec::from_vec(vec![
                DeltaEntry {
                    net: 0,
                    min: 0
                };
                5
            ]),
            global_offset: 0,
        };

        let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

        // Should keep the first boundary (outside quotes) but suppress the second
        assert_eq!(boundaries.len(), 1);
        assert_eq!(boundaries[0].offset, 10);
    }

    #[test]
    fn test_custom_suppression_config() {
        let rules = Arc::new(MockLanguageRules::english());
        let mut config = QuoteSuppressionConfig::default();
        config.suppress_in_double_quotes = false; // Don't suppress in double quotes

        let reducer = BoundaryReducerV2::with_config(rules, config);

        let candidates = vec![BoundaryCandidate {
            local_offset: 25,
            local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]), // Inside double quotes
            flags: BoundaryFlags::STRONG,
        }];

        let chunk_start = ChunkStartState {
            cumulative_deltas: crate::domain::types::DeltaVec::from_vec(vec![
                DeltaEntry {
                    net: 0,
                    min: 0
                };
                5
            ]),
            global_offset: 0,
        };

        let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

        // Should keep the boundary since suppression is disabled
        assert_eq!(boundaries.len(), 1);
    }

    #[test]
    fn test_weak_boundary_suppression_in_parentheses() {
        let rules = Arc::new(MockLanguageRules::english());
        let reducer = BoundaryReducerV2::new(rules);

        let candidates = vec![
            BoundaryCandidate {
                local_offset: 10,
                local_depths: DepthVec::from_vec(vec![0, 0, 1, 0, 0]), // Inside parentheses
                flags: BoundaryFlags::WEAK,
            },
            BoundaryCandidate {
                local_offset: 25,
                local_depths: DepthVec::from_vec(vec![0, 0, 1, 0, 0]), // Inside parentheses
                flags: BoundaryFlags::STRONG,
            },
        ];

        let chunk_start = ChunkStartState {
            cumulative_deltas: crate::domain::types::DeltaVec::from_vec(vec![
                DeltaEntry {
                    net: 0,
                    min: 0
                };
                5
            ]),
            global_offset: 0,
        };

        let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

        // Should suppress weak boundary but keep strong boundary
        assert_eq!(boundaries.len(), 1);
        assert_eq!(boundaries[0].offset, 25);
    }
}
