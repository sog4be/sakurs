//! Chunk-invariance property tests for the Δ-Stack Monoid algorithm.
//!
//! The algorithm's core guarantee (see docs/DELTA_STACK_ALGORITHM.md) is that
//! results are independent of how the text is chunked and how many threads are
//! used: processing with any chunk size must produce exactly the same boundary
//! set as processing the whole text as a single chunk.
//!
//! Several tests in this file currently fail and are marked `#[ignore]`: they
//! document known chunking bugs in v0.1.1. Run them explicitly with
//! `cargo test --test chunk_invariance -- --ignored` and remove the ignore
//! attributes as fixes land.

use proptest::prelude::*;
use sakurs_core::{Config, Input, SentenceProcessor};

/// Returns boundary byte offsets for the given configuration.
fn boundaries(
    text: &str,
    lang: &str,
    chunk_size: usize,
    overlap: usize,
    threads: usize,
) -> Vec<usize> {
    let config = Config::builder()
        .language(lang)
        .expect("language config should load")
        .chunk_size(chunk_size)
        .overlap_size(overlap.min(chunk_size.saturating_sub(1)))
        .threads(Some(threads))
        .parallel_threshold(0)
        .build()
        .expect("config should validate");
    let processor = SentenceProcessor::with_config(config).expect("processor should build");
    let output = processor
        .process(Input::from_text(text.to_string()))
        .expect("processing should succeed");
    output.boundaries.iter().map(|b| b.offset).collect()
}

/// Reference result: the whole text processed as a single chunk on one thread.
fn reference(text: &str, lang: &str) -> Vec<usize> {
    boundaries(text, lang, text.len() + 1024, 256, 1)
}

// ---------------------------------------------------------------------------
// Deterministic invariance sweeps
// ---------------------------------------------------------------------------

/// Plain English prose without quotes or abbreviations chunks cleanly today.
/// This is the passing baseline that guards the delta/prefix-sum plumbing.
#[test]
fn plain_english_is_chunk_invariant() {
    let unit = "The quick brown fox jumps over the lazy dog near the river bank. \
It was a sunny day and everyone was happy about the weather. ";
    let text = unit.repeat(100); // ~13KB, ends with a space
    let expected = reference(&text, "en");
    assert!(!expected.is_empty());
    for chunk_size in [1024, 2048, 4096, 8192] {
        for threads in [1, 2, 4] {
            let got = boundaries(&text, "en", chunk_size, 256.min(chunk_size / 2), threads);
            assert_eq!(
                got, expected,
                "boundaries diverged at chunk_size={chunk_size}, threads={threads}"
            );
        }
    }
}

/// English text containing quotes and parentheses must be chunk-invariant.
/// Fails on v0.1.1: enclosure deltas in overlap regions are double-counted by
/// the prefix sum, so entire chunks lose (or gain) boundaries.
#[test]
fn quoted_english_is_chunk_invariant() {
    let unit = "He said \"Hello there my friend.\" Then (quite slowly) he left the room. \
She replied \"I will see you tomorrow.\" The others (all of them) nodded in agreement. ";
    let text = unit.repeat(100); // ~16KB
    let expected = reference(&text, "en");
    assert!(!expected.is_empty());
    for chunk_size in [1024, 2048, 4096, 8192] {
        for threads in [1, 2] {
            let got = boundaries(&text, "en", chunk_size, 256.min(chunk_size / 2), threads);
            assert_eq!(
                got, expected,
                "boundaries diverged at chunk_size={chunk_size}, threads={threads}"
            );
        }
    }
}

/// Japanese text with 「」/『』 brackets must be chunk-invariant.
/// Fails on v0.1.1 for the same reason as the quoted-English sweep; with the
/// default 256KB chunks this manifests as losing every boundary after the
/// first chunk boundary on real-world sized documents.
#[test]
fn japanese_brackets_are_chunk_invariant() {
    let unit = "彼は「こんにちは」と言った。彼女は『それは素晴らしい』と答えた。\
今日はとても良い天気です。明日も晴れるでしょうか。皆で公園へ行きました。";
    let text = unit.repeat(80); // ~16KB
    let expected = reference(&text, "ja");
    assert!(!expected.is_empty());
    for chunk_size in [1024, 2048, 4096, 8192] {
        for threads in [1, 2] {
            let got = boundaries(&text, "ja", chunk_size, 256.min(chunk_size / 2), threads);
            assert_eq!(
                got, expected,
                "boundaries diverged at chunk_size={chunk_size}, threads={threads}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property tests over generated corpora
// ---------------------------------------------------------------------------

const EN_FRAGMENTS: &[&str] = &[
    "The quick brown fox jumps over the lazy dog. ",
    "It was a sunny day. ",
    "He said \"Hello there. It is me.\" and left. ",
    "She replied (rather quietly) that all was well. ",
    "Dr. Smith met Prof. Brown at the U.S. embassy. ",
    "The value is 3.14 exactly. ",
    "Wait... what happened next? ",
    "Amazing!? Truly amazing! ",
];

const JA_FRAGMENTS: &[&str] = &[
    "彼は「こんにちは」と言った。",
    "彼女は『それは素晴らしい』と答えた。",
    "今日はとても良い天気です。",
    "明日も晴れるでしょうか?",
    "皆で公園へ行きました!",
];

fn build_text(fragments: &[&str], indices: &[usize], trim_trailing: bool) -> String {
    let mut text: String = indices
        .iter()
        .map(|&i| fragments[i % fragments.len()])
        .collect();
    if trim_trailing {
        text.truncate(text.trim_end().len());
    }
    text
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    /// Any mix of English sentence shapes must be chunk-invariant.
    ///
    /// English boundary decisions need lookahead (next word after an
    /// abbreviation, decimal digits, ellipsis context); candidates whose
    /// lookahead crosses a chunk edge are carried as pending and judged when
    /// the neighboring chunk's context is available, so the decision is
    /// identical to the single-chunk run.
    #[test]
    fn generated_english_is_chunk_invariant(
        indices in prop::collection::vec(0usize..EN_FRAGMENTS.len(), 3..40),
        chunk_size in prop::sample::select(vec![64usize, 128, 256, 512, 1024, 4096]),
        overlap in prop::sample::select(vec![0usize, 16, 64, 256]),
        threads in prop::sample::select(vec![1usize, 2, 4]),
        trim_trailing in any::<bool>(),
    ) {
        let text = build_text(EN_FRAGMENTS, &indices, trim_trailing);
        let expected = reference(&text, "en");
        let got = boundaries(&text, "en", chunk_size, overlap, threads);
        prop_assert_eq!(
            got, expected,
            "boundaries diverged: chunk_size={}, overlap={}, threads={}, text={:?}",
            chunk_size, overlap, threads, text
        );
    }

    /// Any mix of Japanese sentence shapes must be chunk-invariant.
    #[test]
    fn generated_japanese_is_chunk_invariant(
        indices in prop::collection::vec(0usize..JA_FRAGMENTS.len(), 3..40),
        chunk_size in prop::sample::select(vec![64usize, 128, 256, 512, 1024, 4096]),
        overlap in prop::sample::select(vec![0usize, 16, 64, 256]),
        threads in prop::sample::select(vec![1usize, 2, 4]),
        trim_trailing in any::<bool>(),
    ) {
        let text = build_text(JA_FRAGMENTS, &indices, trim_trailing);
        let expected = reference(&text, "ja");
        let got = boundaries(&text, "ja", chunk_size, overlap, threads);
        prop_assert_eq!(
            got, expected,
            "boundaries diverged: chunk_size={}, overlap={}, threads={}, text={:?}",
            chunk_size, overlap, threads, text
        );
    }
}
