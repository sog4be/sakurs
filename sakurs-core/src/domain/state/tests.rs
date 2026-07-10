//! Property and unit tests for the partial-state monoid.
//!
//! The tests instantiate the state model with a *toy* scanner and a
//! hash-based judge. The toy scanner honors the same contract as the real
//! one (items ≥ k characters from both chunk edges are decided inline, the
//! rest go to pending; head/tail context buffers cover 2k characters), and
//! the hash judge is a pure function of the exact window content, so any
//! discrepancy in window reconstruction across chunkings flips verdicts and
//! fails the equivalence assertions.
//!
//! Toy language: `.` `!` `?` terminate; `(`/`)` are an asymmetric enclosure
//! (net index 0) with a *suppressible* closer; `"` is a suppressible
//! symmetric enclosure (parity bit 0).

use super::*;
use crate::domain::types::{BoundaryFlags, DepthVec};
use proptest::prelude::*;
use smallvec::smallvec;
use std::cell::Cell;

const SYM_QUOTE: EnclosureSlot = EnclosureSlot::Sym { bit: 0 };
const ASYM_CLOSE: EnclosureSlot = EnclosureSlot::Asym {
    index: 0,
    delta: -1,
};

/// Toy scanner following the real scanner's contract.
fn toy_scan<J: Judge>(text: &str, judge: &J) -> PartialState {
    let mut state = PartialState::identity();
    state.deltas.push(0);
    state.chunk_len = text.len();
    state.head_ctx = ContextBuf::head_of(text);
    state.tail_ctx = ContextBuf::tail_of(text);

    let total_chars = text.chars().count();
    let mut depth: i32 = 0;
    let mut parity: u32 = 0;
    let mut char_idx = 0usize;

    for (i, ch) in text.char_indices() {
        // Characters strictly before / from (inclusive) this character —
        // matching PartialState::chars_before / chars_after semantics.
        let before = char_idx;
        let after = total_chars - char_idx;
        char_idx += 1;

        match ch {
            '(' => depth += 1, // not suppressible: counted unconditionally
            ')' | '"' => {
                // A suppressible enclosure character is decided inline when
                // its ±k window is inside the chunk, and becomes a pending
                // enclosure otherwise (its effect stays excluded until
                // resolution).
                let slot = if ch == ')' { ASYM_CLOSE } else { SYM_QUOTE };
                if before >= WINDOW_CHARS && after >= WINDOW_CHARS {
                    let (window, pos) = window_around(text, i, WINDOW_CHARS);
                    if !judge.suppress_enclosure(window, pos, ch) {
                        apply(slot, &mut depth, &mut parity);
                    }
                } else {
                    state.pending_enc.push(PendingEnclosure {
                        local_offset: i,
                        ch,
                        slot,
                    });
                }
            }
            '.' | '!' | '?' => {
                let offset = i + ch.len_utf8();
                let kind = TerminatorKind::Char(ch);
                let depths: DepthVec = smallvec![depth];
                // For the candidate the reference point is the offset after
                // the terminator: `before + 1` characters precede it.
                if before + 1 >= WINDOW_CHARS && after > WINDOW_CHARS {
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

    state.deltas[0] = depth;
    state.parity = parity;
    state
}

fn apply(slot: EnclosureSlot, depth: &mut i32, parity: &mut u32) {
    match slot {
        EnclosureSlot::Asym { delta, .. } => *depth += i32::from(delta),
        EnclosureSlot::Sym { bit } => *parity ^= 1 << bit,
    }
}

/// A pure judge whose verdicts depend on every byte of the window, the
/// position, and the item identity — maximally sensitive to any
/// window-reconstruction error.
struct HashJudge;

fn window_hash(window: &str, pos: usize, salt: u64) -> u64 {
    let mut h: u64 = salt;
    for &b in window.as_bytes() {
        h = h.wrapping_mul(1_099_511_628_211).wrapping_add(u64::from(b));
    }
    h ^ (pos as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

impl Judge for HashJudge {
    fn judge(&self, window: &str, pos_in_window: usize, kind: TerminatorKind) -> Judgment {
        let salt = match kind {
            TerminatorKind::Char(c) => c as u64,
            TerminatorKind::AfterClosers(c) => 0x1000 + c as u64,
        };
        match window_hash(window, pos_in_window, salt) % 3 {
            0 => Judgment::NotBoundary,
            1 => Judgment::Boundary(BoundaryFlags::WEAK),
            _ => Judgment::Boundary(BoundaryFlags::STRONG),
        }
    }

    fn suppress_enclosure(&self, window: &str, pos_in_window: usize, ch: char) -> bool {
        window_hash(window, pos_in_window, 0x300 + ch as u64) % 2 == 0
    }
}

/// Maps arbitrary indices to sorted cut positions on character boundaries.
pub(crate) fn char_boundary_cuts(text: &str, ixs: &[prop::sample::Index]) -> Vec<usize> {
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

pub(crate) fn segments<'a>(text: &'a str, cuts: &[usize]) -> Vec<&'a str> {
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

    fn suppress_enclosure(&self, _window: &str, _pos_in_window: usize, _ch: char) -> bool {
        panic!("no suppressible characters in this test's input")
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

/// Deterministic judge for the enclosure-resolution unit tests: `"` is a
/// contraction (suppressed) iff both immediate neighbors are alphabetic;
/// `)` is never suppressed; every candidate is a strong boundary.
struct ScriptJudge;

impl Judge for ScriptJudge {
    fn judge(&self, _window: &str, _pos_in_window: usize, _kind: TerminatorKind) -> Judgment {
        Judgment::Boundary(BoundaryFlags::STRONG)
    }

    fn suppress_enclosure(&self, window: &str, pos_in_window: usize, ch: char) -> bool {
        if ch != '"' {
            return false;
        }
        let before = window[..pos_in_window].chars().next_back();
        let after = window[pos_in_window..].chars().nth(1);
        matches!((before, after), (Some(b), Some(a)) if b.is_alphabetic() && a.is_alphabetic())
    }
}

#[test]
fn resolved_enclosure_toggles_later_candidates() {
    // `"` after `(`: not a contraction, so it is a real quote whose parity
    // toggle applies to the candidate after it once resolved at the edges.
    let judge = ScriptJudge;
    let left = toy_scan("(\"", &judge);
    assert_eq!(
        left.deltas[0], 1,
        "the unsuppressible '(' counts immediately"
    );
    assert_eq!(left.parity, 0, "the pending '\"' stays excluded");
    assert_eq!(left.pending_enc.len(), 1);

    let combined = left.combine_with(&toy_scan("xy.", &judge), &judge);
    assert_eq!(combined.pending_enc.len(), 1, "still no context to decide");
    assert_eq!(combined.pending.len(), 1);
    assert_eq!(combined.pending[0].local_offset, 5);
    assert_eq!(
        combined.pending[0].local_depths[0], 1,
        "depth rebased by left's net"
    );

    let resolved = combined.resolve_edges(&judge);
    assert_eq!(
        resolved.parity, 1,
        "the resolved quote toggles the state parity"
    );
    assert_eq!(resolved.deltas[0], 1);
    assert_eq!(resolved.boundaries.len(), 1);
    let b = &resolved.boundaries[0];
    assert_eq!(b.local_offset, 5);
    assert_eq!(b.local_depths[0], 1);
    assert_eq!(
        b.local_parity, 1,
        "candidate parity includes the resolved toggle"
    );
    assert!(resolved.pending_enc.is_empty());
}

#[test]
fn suppressed_enclosure_leaves_no_trace() {
    // `"` between alphabetic characters is a contraction under ScriptJudge:
    // resolving it must not toggle parity anywhere.
    let judge = ScriptJudge;
    let resolved = toy_scan("ab\"cd.", &judge).resolve_edges(&judge);
    assert_eq!(resolved.parity, 0);
    assert_eq!(resolved.boundaries.len(), 1);
    assert_eq!(resolved.boundaries[0].local_parity, 0);
    assert!(resolved.pending_enc.is_empty());
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
