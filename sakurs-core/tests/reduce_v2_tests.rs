//! Additional tests for BoundaryReducerV2
use sakurs_core::domain::{
    language::EnglishLanguageRules,
    prefix_sum::{ChunkStartState, PrefixSumComputer},
    quote_suppression::QuoteSuppressionConfig,
    reduce_v2::BoundaryReducerV2,
    state::{BoundaryCandidate, DeltaEntry, PartialState},
    types::{BoundaryVec, DeltaVec, DepthVec},
    BoundaryFlags,
};
use std::sync::Arc;

#[test]
fn test_reducer_creation_and_configuration() {
    let rules = Arc::new(EnglishLanguageRules::new());

    // Test default creation
    let reducer1 = BoundaryReducerV2::new(rules.clone());

    // Test with custom config
    let config = QuoteSuppressionConfig {
        suppress_in_double_quotes: false,
        suppress_in_single_quotes: true,
        validate_pairing: false,
        max_nesting_level: 3,
    };
    let reducer2 = BoundaryReducerV2::with_config(rules.clone(), config);

    // Both should be created successfully
    let empty_states = vec![];
    let chunk_starts = vec![];
    let boundaries1 = reducer1.reduce_all(&empty_states, &chunk_starts);
    let boundaries2 = reducer2.reduce_all(&empty_states, &chunk_starts);
    assert_eq!(boundaries1.len(), 0);
    assert_eq!(boundaries2.len(), 0);
}

#[test]
fn test_filter_weak_boundaries() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let reducer = BoundaryReducerV2::new(rules);

    // Create candidates with mixed weak and strong boundaries
    let candidates = vec![
        BoundaryCandidate {
            local_offset: 10,
            local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
            flags: BoundaryFlags::WEAK,
        },
        BoundaryCandidate {
            local_offset: 20,
            local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 30,
            local_depths: DepthVec::from_vec(vec![0, 0, 1, 0, 0]), // Inside parentheses
            flags: BoundaryFlags::WEAK,
        },
    ];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // Should keep strong boundary and weak boundary not in enclosure
    assert_eq!(boundaries.len(), 2);
    assert_eq!(boundaries[0].offset, 10);
    assert_eq!(boundaries[1].offset, 20);
}

#[test]
fn test_deeply_nested_enclosures() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let config = QuoteSuppressionConfig {
        max_nesting_level: 2,
        ..Default::default()
    };
    let reducer = BoundaryReducerV2::with_config(rules, config);

    // Create candidates with different nesting levels
    let candidates = vec![
        BoundaryCandidate {
            local_offset: 10,
            local_depths: DepthVec::from_vec(vec![1, 0, 0, 0, 0]), // Level 1
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 20,
            local_depths: DepthVec::from_vec(vec![2, 0, 0, 0, 0]), // Level 2
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 30,
            local_depths: DepthVec::from_vec(vec![3, 0, 0, 0, 0]), // Level 3 (exceeds max)
            flags: BoundaryFlags::STRONG,
        },
    ];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // All boundaries inside quotes should be suppressed
    assert_eq!(boundaries.len(), 0);
}

#[test]
fn test_multiple_enclosure_types() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let reducer = BoundaryReducerV2::new(rules);

    // Test with multiple types of enclosures
    let candidates = vec![
        BoundaryCandidate {
            local_offset: 10,
            local_depths: DepthVec::from_vec(vec![1, 1, 0, 0, 0]), // Inside both double and single quotes
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 20,
            local_depths: DepthVec::from_vec(vec![0, 0, 1, 1, 0]), // Inside parentheses and brackets
            flags: BoundaryFlags::STRONG,
        },
        BoundaryCandidate {
            local_offset: 30,
            local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 1]), // Inside braces
            flags: BoundaryFlags::STRONG,
        },
    ];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // Quotes should be suppressed, parentheses/brackets/braces should be kept
    assert_eq!(boundaries.len(), 2);
    assert_eq!(boundaries[0].offset, 20);
    assert_eq!(boundaries[1].offset, 30);
}

#[test]
fn test_reduce_all_with_multiple_chunks() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let reducer = BoundaryReducerV2::new(rules);

    // Create multiple chunks with boundaries
    let states = vec![
        PartialState {
            boundary_candidates: BoundaryVec::from_vec(vec![BoundaryCandidate {
                local_offset: 15,
                local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
                flags: BoundaryFlags::STRONG,
            }]),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 1, min: 0 }; 5]),
            abbreviation: Default::default(),
            chunk_length: 20,
        },
        PartialState {
            boundary_candidates: BoundaryVec::from_vec(vec![
                BoundaryCandidate {
                    local_offset: 10,
                    local_depths: DepthVec::from_vec(vec![-1, 0, 0, 0, 0]),
                    flags: BoundaryFlags::STRONG,
                },
                BoundaryCandidate {
                    local_offset: 25,
                    local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
                    flags: BoundaryFlags::WEAK,
                },
            ]),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: -1, min: -1 }; 5]),
            abbreviation: Default::default(),
            chunk_length: 30,
        },
    ];

    let chunk_starts = PrefixSumComputer::compute_prefix_sum(&states);
    let boundaries = reducer.reduce_all(&states, &chunk_starts);

    // Should have boundaries from both chunks - the second boundary is suppressed due to quotes
    assert_eq!(boundaries.len(), 2);
    assert_eq!(boundaries[0].offset, 15);
    assert_eq!(boundaries[1].offset, 30); // 20 + 10 (second boundary, weak one is kept)
}

#[test]
fn test_empty_chunks_handling() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let reducer = BoundaryReducerV2::new(rules);

    // Mix empty and non-empty chunks
    let states = vec![
        PartialState {
            boundary_candidates: BoundaryVec::new(),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
            abbreviation: Default::default(),
            chunk_length: 10,
        },
        PartialState {
            boundary_candidates: BoundaryVec::from_vec(vec![BoundaryCandidate {
                local_offset: 5,
                local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
                flags: BoundaryFlags::STRONG,
            }]),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
            abbreviation: Default::default(),
            chunk_length: 10,
        },
        PartialState {
            boundary_candidates: BoundaryVec::new(),
            deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
            abbreviation: Default::default(),
            chunk_length: 10,
        },
    ];

    let chunk_starts = PrefixSumComputer::compute_prefix_sum(&states);
    let boundaries = reducer.reduce_all(&states, &chunk_starts);

    // Should only have the boundary from the middle chunk
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].offset, 15); // 10 + 5
}

#[test]
fn test_suppression_with_no_config() {
    let rules = Arc::new(EnglishLanguageRules::new());
    let config = QuoteSuppressionConfig {
        suppress_in_double_quotes: false,
        suppress_in_single_quotes: false,
        validate_pairing: false,
        max_nesting_level: 0,
    };
    let reducer = BoundaryReducerV2::with_config(rules, config);

    // Even boundaries inside quotes should be kept
    let candidates = vec![BoundaryCandidate {
        local_offset: 10,
        local_depths: DepthVec::from_vec(vec![1, 1, 0, 0, 0]),
        flags: BoundaryFlags::STRONG,
    }];

    let chunk_start = ChunkStartState {
        cumulative_deltas: DeltaVec::from_vec(vec![DeltaEntry { net: 0, min: 0 }; 5]),
        global_offset: 0,
    };

    let boundaries = reducer.evaluate_candidates(&candidates, &chunk_start);

    // Should keep the boundary
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].offset, 10);
}
