//! Integration tests for quote suppression functionality

use sakurs_core::domain::{
    language::EnglishLanguageRules,
    prefix_sum::{ChunkStartState, PrefixSumComputer},
    quote_suppression::{
        QuoteSuppressionConfig, QuoteSuppressionContext, QuoteSuppressor, SuppressionDecision,
    },
    reduce_v2::BoundaryReducerV2,
    state::{BoundaryCandidate, DeltaEntry, PartialState},
    types::{BoundaryVec, DeltaVec, DepthVec},
    BoundaryFlags,
};
use std::sync::Arc;

#[test]
fn test_basic_quote_suppression() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let reducer = BoundaryReducerV2::new(rules.clone());

    // Text: He said "Hello. How are you?" and left.
    //                  ^-- This boundary should be suppressed
    let candidates = vec![
        BoundaryCandidate {
            local_offset: 15,                                      // After "Hello."
            local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]), // Inside double quotes
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 30,                                      // After "you?"
            local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]), // Outside quotes
            flags: BoundaryFlags::STRONG,
        },
    ];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // Should suppress the boundary inside quotes
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].offset, 30);
}

#[test]
fn test_nested_quote_handling() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let mut config = QuoteSuppressionConfig::default();
    config.suppress_in_double_quotes = false; // Allow boundaries in quotes
    config.max_nesting_level = 2;

    let reducer = BoundaryReducerV2::with_config(rules, config);

    // Text with nested quotes
    let candidates = vec![
        BoundaryCandidate {
            local_offset: 20,
            local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]), // Single level
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 40,
            local_depths: DepthVec::from_vec(vec![2, 0, 0, 0, 0]), // Nested quotes
            flags: BoundaryFlags::STRONG,
        },
    ];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // First boundary should be kept as STRONG
    // Second boundary should be weakened due to nesting
    assert_eq!(boundaries.len(), 2);
    assert_eq!(boundaries[0].flags, BoundaryFlags::STRONG);
    assert_eq!(boundaries[1].flags, BoundaryFlags::WEAK);
}

#[test]
fn test_parenthetical_boundaries() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let reducer = BoundaryReducerV2::new(rules);

    // Text: The result (see Table 1. for details) was significant.
    //                               ^-- Strong boundary should be kept
    let candidates = vec![
        BoundaryCandidate {
            local_offset: 25,                                      // After "Table 1."
            local_depths: DepthVec::from_vec(vec![0, 0, 1, 0, 0]), // Inside parentheses
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 25, // Same position, weak boundary
            local_depths: DepthVec::from_vec(vec![0, 0, 1, 0, 0]), // Inside parentheses
            flags: BoundaryFlags::WEAK,
        },
    ];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // Should keep strong boundary but suppress weak boundary in parentheses
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].flags, BoundaryFlags::STRONG);
}

#[test]
fn test_cross_chunk_quote_suppression() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let reducer = BoundaryReducerV2::new(rules);

    // Simulate two chunks where quote opens in first chunk
    let states = vec![
        PartialState {
            boundary_candidates: BoundaryVec::from_vec(vec![]),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }; 5]), // Opens quote
            abbreviation: Default::default(),
            chunk_length: 50,
        },
        PartialState {
            boundary_candidates: BoundaryVec::from_vec(vec![BoundaryCandidate {
                local_offset: 10,                                       // Boundary in second chunk
                local_depths: DepthVec::from_vec(vec![-1, 0, 0, 0, 0]), // Closes quote
                flags: BoundaryFlags::STRONG,
            }]),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: -1, min: -1 }; 5]),
            abbreviation: Default::default(),
            chunk_length: 50,
        },
    ];

    let chunk_starts = PrefixSumComputer::compute_prefix_sum(&states);
    let boundaries = reducer.reduce_all(&states, &chunk_starts);

    // The boundary should be suppressed because it's inside quotes
    // (global depth = 1 + (-1) = 0 at the boundary position, but before closing quote)
    assert_eq!(boundaries.len(), 1);
}

#[test]
fn test_custom_suppression_configuration() {
    let rules = Arc::new(EnglishLanguageRules::new());

    // Test with single quotes not suppressed
    let mut config = QuoteSuppressionConfig::default();
    config.suppress_in_single_quotes = false;

    let reducer = BoundaryReducerV2::with_config(rules, config);

    let candidates = vec![BoundaryCandidate {
        local_offset: 15,
        local_depths: DepthVec::from_vec(vec![0, 1, 0, 0, 0]), // Inside single quotes
        flags: BoundaryFlags::STRONG,
    }];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // Should keep the boundary since single quote suppression is disabled
    assert_eq!(boundaries.len(), 1);
}

#[test]
fn test_suppression_decision_types() {
    let config = QuoteSuppressionConfig::default();
    let rules = EnglishLanguageRules::new();

    // Test suppress decision
    let candidate = BoundaryCandidate {
        local_offset: 10,
        local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]),
        flags: BoundaryFlags::STRONG,
    };

    let context = QuoteSuppressionContext {
        candidate: &candidate,
        language_rules: &rules,
        enclosure_depths: &[1, 0, 0, 0, 0],
        config: &config,
    };

    let decision = QuoteSuppressor::evaluate(context);
    assert!(matches!(decision, SuppressionDecision::Suppress { .. }));

    // Test keep decision
    let candidate2 = BoundaryCandidate {
        local_offset: 10,
        local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
        flags: BoundaryFlags::STRONG,
    };

    let context2 = QuoteSuppressionContext {
        candidate: &candidate2,
        language_rules: &rules,
        enclosure_depths: &[0, 0, 0, 0, 0],
        config: &config,
    };

    let decision2 = QuoteSuppressor::evaluate(context2);
    assert_eq!(decision2, SuppressionDecision::Keep);
}
