//! The scan phase: one pass over a chunk, producing a [`PartialState`].
//!
//! The scanner never decides anything context-dependent itself (see
//! `docs/DELTA_STACK_ALGORITHM.md`, "Design Principle: Deferred Judgment"):
//! interior items are decided inline by the [`Judge`] oracles on borrowed
//! windows, and items within [`WINDOW_CHARS`] of a chunk edge are recorded as
//! pending. Per-character work is a table lookup plus depth/parity updates.

use super::candidate::{
    Candidate, EnclosureSlot, Judge, Judgment, PendingCandidate, PendingEnclosure, TerminatorKind,
};
use super::compiled::CompiledRules;
use super::context::{window_around, ContextBuf, WINDOW_CHARS};
use super::PartialState;
use crate::domain::types::DepthVec;

/// Scans one chunk into a partial state.
pub(crate) fn scan_chunk(text: &str, rules: &CompiledRules) -> PartialState {
    let mut state = PartialState::identity();
    state.deltas.resize(rules.asym_type_count(), 0);
    state.chunk_len = text.len();
    state.head_ctx = ContextBuf::head_of(text);
    state.tail_ctx = ContextBuf::tail_of(text);

    let total_chars = text.chars().count();
    let mut depths: DepthVec = state.deltas.clone();
    let mut parity: u32 = 0;

    for (char_idx, (i, ch)) in text.char_indices().enumerate() {
        // Characters strictly before / from (inclusive) this character —
        // the same quantities PartialState::chars_before/chars_after report.
        let before = char_idx;
        let after = total_chars - char_idx;

        let class = rules.classify(ch);

        if let Some(enc) = class.enclosure {
            if !enc.suppressible {
                apply_slot(enc.slot, &mut depths, &mut parity);
            } else if before >= WINDOW_CHARS && after >= WINDOW_CHARS {
                let (window, pos) = window_around(text, i, WINDOW_CHARS);
                if !rules.suppress_enclosure(window, pos, ch) {
                    apply_slot(enc.slot, &mut depths, &mut parity);
                }
            } else {
                state.pending_enc.push(PendingEnclosure {
                    local_offset: i,
                    ch,
                    slot: enc.slot,
                });
            }
        }

        if class.terminator {
            let offset = i + ch.len_utf8();
            let kind = TerminatorKind::Char(ch);
            // The candidate's reference point is the offset after the
            // terminator: `before + 1` characters precede it and `after - 1`
            // follow it.
            if before + 1 >= WINDOW_CHARS && after > WINDOW_CHARS {
                let (window, pos) = window_around(text, offset, WINDOW_CHARS);
                if let Judgment::Boundary(flags) = rules.judge(window, pos, kind) {
                    state.boundaries.push(Candidate {
                        local_offset: offset,
                        local_depths: depths.clone(),
                        local_parity: parity,
                        flags,
                    });
                }
            } else {
                state.pending.push(PendingCandidate {
                    local_offset: offset,
                    local_depths: depths.clone(),
                    local_parity: parity,
                    kind,
                });
            }
        }
    }

    state.deltas = depths;
    state.parity = parity;
    state
}

fn apply_slot(slot: EnclosureSlot, depths: &mut DepthVec, parity: &mut u32) {
    match slot {
        EnclosureSlot::Asym { index, delta } => {
            let i = index as usize;
            debug_assert!(i < depths.len(), "slot index within compiled type count");
            depths[i] += i32::from(delta);
        }
        EnclosureSlot::Sym { bit } => *parity ^= 1 << bit,
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{char_boundary_cuts, segments};
    use super::*;
    use crate::domain::language::ConfigurableLanguageRules;
    use crate::{DeltaStackProcessor, ExecutionMode, ProcessorConfig};
    use proptest::prelude::*;
    use std::sync::{Arc, LazyLock};

    static EN: LazyLock<CompiledRules> =
        LazyLock::new(|| CompiledRules::from_code("en").expect("en config compiles"));
    static JA: LazyLock<CompiledRules> =
        LazyLock::new(|| CompiledRules::from_code("ja").expect("ja config compiles"));

    /// Extracts final boundaries from a fully-combined v2 state: candidates
    /// outside every enclosure (zero cumulative prefix for a single state).
    fn v2_boundaries(state: &PartialState) -> Vec<usize> {
        state
            .boundaries
            .iter()
            .filter(|c| c.local_parity == 0 && c.local_depths.iter().all(|&d| d <= 0))
            .map(|c| c.local_offset)
            .collect()
    }

    fn legacy_boundaries(text: &str, code: &str) -> Vec<usize> {
        let rules = Arc::new(ConfigurableLanguageRules::from_code(code).unwrap());
        let processor = DeltaStackProcessor::new(ProcessorConfig::default(), rules);
        processor
            .process(text, ExecutionMode::Sequential)
            .unwrap()
            .boundaries
    }

    /// Single-chunk v2 output must equal the legacy sequential pipeline —
    /// the porting-fidelity check for `judge`/`suppress_enclosure`.
    #[test]
    fn single_chunk_matches_legacy_pipeline() {
        let en_texts = [
            "Dr. Smith went to Washington. He arrived at 3.5 p.m. and left.",
            "She said \"Hello world.\" Then (after a pause) she left! Really?!",
            "Wait... what happened? The U.S. economy grew. That's John's book.",
            "Don't stop. \"It's fine,\" he said. Lists use 1) markers.",
            "Short. Even shorter! End",
        ];
        for text in en_texts {
            let state = scan_chunk(text, &EN).resolve_edges(&*EN);
            assert_eq!(
                v2_boundaries(&state),
                legacy_boundaries(text, "en"),
                "en mismatch for {text:?}"
            );
        }

        let ja_texts = [
            "彼は「こんにちは」と言った。今日は晴れ!明日は?",
            "彼は『引用「入れ子」だ』と言った。終わり。",
            "これはテストです。値は3.5です。すごい!?",
            "「囲まれた文。」の外。",
        ];
        for text in ja_texts {
            let state = scan_chunk(text, &JA).resolve_edges(&*JA);
            assert_eq!(
                v2_boundaries(&state),
                legacy_boundaries(text, "ja"),
                "ja mismatch for {text:?}"
            );
        }
    }

    fn en_soup() -> impl Strategy<Value = String> {
        let token = prop::sample::select(vec![
            "the", "He", "said", "Dr.", "U.S.", "Mr.", "3.5", "approx", "Smith", "don't", ". ",
            "! ", "? ", "... ", "?! ", "(", ")", "\"", "'", ", ", " ", "\n", "word.", "A",
        ]);
        proptest::collection::vec(token, 0..60).prop_map(|v| v.concat())
    }

    fn ja_soup() -> impl Strategy<Value = String> {
        let token = prop::sample::select(vec![
            "彼は",
            "言った",
            "。",
            "!",
            "?",
            "!?",
            "「",
            "」",
            "『",
            "』",
            "(",
            ")",
            "…",
            "今日",
            "3.5",
            " ",
            "\n",
            "。」",
            "です",
        ]);
        proptest::collection::vec(token, 0..60).prop_map(|v| v.concat())
    }

    proptest! {
        /// Sequential equivalence with the real English rules: arbitrary
        /// chunkings produce the identical state.
        #[test]
        fn en_chunked_equals_single_chunk(
            text in en_soup(),
            ixs in proptest::collection::vec(any::<prop::sample::Index>(), 0..5),
        ) {
            let whole = scan_chunk(&text, &EN).resolve_edges(&*EN);
            let cuts = char_boundary_cuts(&text, &ixs);
            let mut acc = PartialState::identity();
            for seg in segments(&text, &cuts) {
                acc = acc.combine_with(&scan_chunk(seg, &EN), &*EN);
            }
            let chunked = acc.resolve_edges(&*EN);
            prop_assert_eq!(&whole.boundaries, &chunked.boundaries);
            prop_assert_eq!(&whole.deltas, &chunked.deltas);
            prop_assert_eq!(whole.parity, chunked.parity);
        }

        /// Sequential equivalence with the real Japanese rules.
        #[test]
        fn ja_chunked_equals_single_chunk(
            text in ja_soup(),
            ixs in proptest::collection::vec(any::<prop::sample::Index>(), 0..5),
        ) {
            let whole = scan_chunk(&text, &JA).resolve_edges(&*JA);
            let cuts = char_boundary_cuts(&text, &ixs);
            let mut acc = PartialState::identity();
            for seg in segments(&text, &cuts) {
                acc = acc.combine_with(&scan_chunk(seg, &JA), &*JA);
            }
            let chunked = acc.resolve_edges(&*JA);
            prop_assert_eq!(&whole.boundaries, &chunked.boundaries);
            prop_assert_eq!(&whole.deltas, &chunked.deltas);
            prop_assert_eq!(whole.parity, chunked.parity);
        }
    }
}
