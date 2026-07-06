//! Property and unit tests for the partial-state monoid.
//!
//! The tests instantiate the state model with a *toy* scanner and a
//! hash-based judge. The toy scanner honors the same contract as the real
//! one (candidates ≥ k characters from both chunk edges are judged inline,
//! the rest go to pending; head/tail context buffers cover 2k characters),
//! and the hash judge is a pure function of the exact window content, so any
//! discrepancy in window reconstruction across chunkings flips verdicts and
//! fails the equivalence assertions.

use super::*;
use crate::domain::types::{BoundaryFlags, DeltaEntry, DepthVec};
use proptest::prelude::*;
use smallvec::smallvec;
use std::cell::Cell;

/// Toy scanner: `.` `!` `?` are terminators, `(`/`)` an asymmetric enclosure
/// (type 0), `"` a symmetric enclosure (parity bit 0).
fn toy_scan<J: Judge>(text: &str, judge: &J) -> PartialState {
    let mut state = PartialState::identity();
    state.deltas.push(DeltaEntry::identity());
    state.chunk_len = text.len();
    state.head_ctx = ContextBuf::head_of(text);
    state.tail_ctx = ContextBuf::tail_of(text);

    let total_chars = text.chars().count();
    let mut depth: i32 = 0;
    let mut min_depth: i32 = 0;
    let mut parity: u32 = 0;
    let mut chars_seen = 0usize;

    for (i, ch) in text.char_indices() {
        chars_seen += 1;
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                min_depth = min_depth.min(depth);
            }
            '"' => parity ^= 1,
            '.' | '!' | '?' => {
                let offset = i + ch.len_utf8();
                let kind = TerminatorKind::Char(ch);
                let depths: DepthVec = smallvec![depth];
                if chars_seen >= WINDOW_CHARS && total_chars - chars_seen >= WINDOW_CHARS {
                    let (window, pos) = window_around(text, offset, WINDOW_CHARS);
                    if let Judgment::Boundary(flags) = judge.judge(window, pos, kind) {
                        state.boundaries.push(Candidate {
                            local_offset: offset,
                            local_depths: depths,
                            local_parity: parity,
                            flags,
                        });
                    }
                } else {
                    state.pending.push(PendingCandidate {
                        local_offset: offset,
                        local_depths: depths,
                        local_parity: parity,
                        kind,
                    });
                }
            }
            _ => {}
        }
    }

    state.deltas[0] = DeltaEntry {
        net: depth,
        min: min_depth,
    };
    state.parity = parity;
    state
}

/// A pure judge whose verdict depends on every byte of the window, the
/// candidate position, and the terminator kind — maximally sensitive to any
/// window-reconstruction error.
struct HashJudge;

impl Judge for HashJudge {
    fn judge(&self, window: &str, pos_in_window: usize, kind: TerminatorKind) -> Judgment {
        let mut h: u64 = match kind {
            TerminatorKind::Char(c) => c as u64,
            TerminatorKind::Pattern { len } => 0x100 + u64::from(len),
            TerminatorKind::Ellipsis { len } => 0x200 + u64::from(len),
        };
        for &b in window.as_bytes() {
            h = h.wrapping_mul(1_099_511_628_211).wrapping_add(u64::from(b));
        }
        h ^= (pos_in_window as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        match h % 3 {
            0 => Judgment::NotBoundary,
            1 => Judgment::Boundary(BoundaryFlags::WEAK),
            _ => Judgment::Boundary(BoundaryFlags::STRONG),
        }
    }
}

/// Maps arbitrary indices to sorted cut positions on character boundaries.
fn char_boundary_cuts(text: &str, ixs: &[prop::sample::Index]) -> Vec<usize> {
    let mut cuts: Vec<usize> = ixs
        .iter()
        .map(|ix| {
            let mut p = ix.index(text.len() + 1).min(text.len());
            while p < text.len() && !text.is_char_boundary(p) {
                p += 1;
            }
            p
        })
        .collect();
    cuts.sort_unstable();
    cuts
}

fn segments<'a>(text: &'a str, cuts: &[usize]) -> Vec<&'a str> {
    let mut segs = Vec::new();
    let mut prev = 0;
    for &c in cuts {
        segs.push(&text[prev..c]);
        prev = c;
    }
    segs.push(&text[prev..]);
    segs
}

fn text_strategy() -> impl Strategy<Value = String> {
    let ch = prop_oneof![
        4 => Just('a'),
        1 => Just('B'),
        2 => Just(' '),
        2 => Just('.'),
        1 => Just('!'),
        1 => Just('('),
        1 => Just(')'),
        1 => Just('"'),
        1 => Just('あ'),
        1 => Just('。'),
        1 => Just('\n'),
    ];
    proptest::collection::vec(ch, 0..300).prop_map(|v| v.into_iter().collect())
}

proptest! {
    /// The theorem in miniature: folding chunk states with ⊕ and resolving
    /// edges yields exactly the single-chunk result, for arbitrary texts and
    /// arbitrary chunkings (including empty and sub-window-sized chunks).
    #[test]
    fn chunked_processing_equals_single_chunk(
        text in text_strategy(),
        ixs in proptest::collection::vec(any::<prop::sample::Index>(), 0..6),
    ) {
        let judge = HashJudge;
        let whole = toy_scan(&text, &judge).resolve_edges(&judge);

        let cuts = char_boundary_cuts(&text, &ixs);
        let mut acc = PartialState::identity();
        for seg in segments(&text, &cuts) {
            acc = acc.combine_with(&toy_scan(seg, &judge), &judge);
        }
        let chunked = acc.resolve_edges(&judge);

        prop_assert_eq!(&whole.boundaries, &chunked.boundaries);
        prop_assert_eq!(&whole.deltas, &chunked.deltas);
        prop_assert_eq!(whole.parity, chunked.parity);
    }

    /// Associativity of ⊕ as full state equality, for arbitrary 3-way splits.
    #[test]
    fn combine_is_associative(
        text in text_strategy(),
        ixs in proptest::collection::vec(any::<prop::sample::Index>(), 2),
    ) {
        let judge = HashJudge;
        let cuts = char_boundary_cuts(&text, &ixs);
        let segs = segments(&text, &cuts);
        let s: Vec<PartialState> = segs.iter().map(|t| toy_scan(t, &judge)).collect();

        let left = s[0].combine_with(&s[1], &judge).combine_with(&s[2], &judge);
        let right = s[0].combine_with(&s[1].combine_with(&s[2], &judge), &judge);
        prop_assert_eq!(left, right);
    }

    /// The empty state is a two-sided identity.
    #[test]
    fn identity_laws(text in text_strategy()) {
        let judge = HashJudge;
        let s = toy_scan(&text, &judge);
        let id = PartialState::identity();
        prop_assert_eq!(id.combine_with(&s, &judge), s.clone());
        prop_assert_eq!(s.combine_with(&id, &judge), s);
    }
}

/// A judge that asserts it is invoked with exactly the expected window.
struct ExpectWindow<'a> {
    expected_window: &'a str,
    expected_pos: usize,
    hits: Cell<usize>,
}

impl Judge for ExpectWindow<'_> {
    fn judge(&self, window: &str, pos_in_window: usize, _kind: TerminatorKind) -> Judgment {
        assert_eq!(window, self.expected_window);
        assert_eq!(pos_in_window, self.expected_pos);
        self.hits.set(self.hits.get() + 1);
        Judgment::Boundary(BoundaryFlags::STRONG)
    }
}

#[test]
fn combine_reconstructs_the_exact_window() {
    // Candidate right at the edge of the left chunk: judged at combine, on a
    // window that must equal the corresponding substring of the full text.
    let full = format!("{}.{}", "a".repeat(40), "b".repeat(45));
    let candidate_offset = 41;
    let (expected_window, expected_pos) = window_around(&full, candidate_offset, WINDOW_CHARS);
    let judge = ExpectWindow {
        expected_window,
        expected_pos,
        hits: Cell::new(0),
    };

    let (l, r) = full.split_at(candidate_offset);
    let combined = toy_scan(l, &judge).combine_with(&toy_scan(r, &judge), &judge);

    assert_eq!(judge.hits.get(), 1, "the pending candidate is judged once");
    assert_eq!(combined.boundaries.len(), 1);
    assert_eq!(combined.boundaries[0].local_offset, candidate_offset);
    assert!(combined.pending.is_empty());
}

#[test]
fn resolve_edges_clips_the_window_at_text_bounds() {
    // A candidate near the start of a short text stays pending through the
    // scan and is resolved with a window clipped at both text edges.
    let text = "ab.cdefghij";
    let judge = ExpectWindow {
        expected_window: text, // k=32 ≫ |text|: the window is the whole text
        expected_pos: 3,
        hits: Cell::new(0),
    };
    let resolved = toy_scan(text, &judge).resolve_edges(&judge);
    assert_eq!(judge.hits.get(), 1);
    assert_eq!(resolved.boundaries.len(), 1);
    assert_eq!(resolved.boundaries[0].local_offset, 3);
}

#[test]
fn combine_rebases_offsets_depths_and_parity() {
    let judge = HashJudge;
    // Left chunk opens a paren and a quote: net depth +1, parity 1.
    let left = toy_scan("(\"", &judge);
    assert_eq!(left.deltas[0].net, 1);
    assert_eq!(left.parity, 1);

    // Right chunk has one candidate at depth 0 / parity 0 locally.
    let right = toy_scan("xy.", &judge);
    assert_eq!(right.pending.len(), 1);

    let combined = left.combine_with(&right, &judge);
    // Short text: the candidate stays pending, rebased to the combined origin.
    assert_eq!(combined.pending.len(), 1);
    let pc = &combined.pending[0];
    assert_eq!(pc.local_offset, 2 + 3);
    assert_eq!(pc.local_depths[0], 1, "depth rebased by left's net");
    assert_eq!(pc.local_parity, 1, "parity rebased by left's parity");
}

#[test]
fn empty_and_terminator_only_texts() {
    let judge = HashJudge;
    let empty = toy_scan("", &judge).resolve_edges(&judge);
    assert!(empty.boundaries.is_empty());
    assert_eq!(empty.chunk_len, 0);

    let dot = toy_scan(".", &judge).resolve_edges(&judge);
    // Whether "." is a boundary is the judge's call; the offset must be right
    // if it is confirmed.
    for b in &dot.boundaries {
        assert_eq!(b.local_offset, 1);
    }
}
