//! Regression tests for specific chunking bugs found in v0.1.1.
//!
//! Each test pins one concrete failure mode with a minimal reproduction.
//! Tests that currently fail are marked `#[ignore]`; run them with
//! `cargo test --test chunking_regressions -- --ignored` and remove the
//! ignore attribute when the corresponding fix lands.

use sakurs_core::{Config, Input, SentenceProcessor};

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

fn reference(text: &str, lang: &str) -> Vec<usize> {
    boundaries(text, lang, text.len() + 1024, 256, 1)
}

/// Bug: when the text ends exactly at a terminator, the final boundary is
/// dropped in multi-chunk mode. The reduce filter in
/// `DeltaStackProcessor::reduce_chunk` keeps only `offset < chunk.end_offset`,
/// which excludes a boundary sitting at the very end of the last chunk.
#[test]
#[ignore = "known v0.1.1 bug: final boundary dropped by reduce filter `offset < end_offset` (fix planned for v0.1.2)"]
fn final_boundary_survives_chunking() {
    let text = {
        let repeated = "This is a sentence. ".repeat(300); // ~6KB
        repeated.trim_end().to_string() // ends with '.'
    };
    let single = boundaries(&text, "en", text.len() + 1024, 256, 1);
    assert!(
        single.contains(&text.len()),
        "single-chunk run must report the final boundary"
    );

    let chunked = boundaries(&text, "en", 1024, 128, 1);
    assert!(
        chunked.contains(&text.len()),
        "multi-chunk run dropped the boundary at the end of the text"
    );
}

/// Bug: an abbreviation split across a chunk boundary ("Dr." at the end of one
/// chunk, "Smith" at the start of the next) produces a spurious boundary. The
/// cross-chunk abbreviation state (dangling_dot / head_alpha) is computed in
/// the scan phase but never consulted when boundaries are resolved.
#[test]
#[ignore = "known v0.1.1 bug: cross-chunk abbreviation state never reaches the reduce phase (fix planned for v0.2.0)"]
fn abbreviation_split_across_chunks() {
    let text =
        "Dr. Smith arrived early. He met Prof. Brown at the U.S. embassy today. They talked.";
    let expected = reference(text, "en");
    for chunk_size in [20, 30, 40, 50, 60] {
        let got = boundaries(text, "en", chunk_size, chunk_size / 4, 2);
        assert_eq!(
            got, expected,
            "boundaries diverged at chunk_size={chunk_size}"
        );
    }
}

/// Bug: a quotation split across a chunk boundary loses its enclosure context,
/// so terminators inside the quote are accepted as boundaries (and boundaries
/// after the quote can be lost).
#[test]
#[ignore = "known v0.1.1 bug: enclosure context lost at chunk boundaries (fix planned for v0.2.0)"]
fn quotation_split_across_chunks() {
    let text = "He said \"Hello there. It is me.\" Then he left quickly. She smiled at him.";
    let expected = reference(text, "en");
    for chunk_size in [20, 30, 40, 50] {
        let got = boundaries(text, "en", chunk_size, chunk_size / 4, 2);
        assert_eq!(
            got, expected,
            "boundaries diverged at chunk_size={chunk_size}"
        );
    }
}

/// Bug: abbreviation lookup mixes byte offsets and character indices
/// (`AbbreviationTrie::find_at_position` indexes a `Vec<char>` with a byte
/// offset; `process_abbreviation` calls `chars().nth(byte_offset)`). Any
/// multi-byte character before an abbreviation shifts the lookup window, so
/// the abbreviation is no longer recognized. This is a sequential bug: it does
/// not need chunking to trigger.
#[test]
#[ignore = "known v0.1.1 bug: byte/char index confusion in abbreviation lookup (fix planned for v0.1.2)"]
fn abbreviation_after_multibyte_text_is_recognized() {
    // Pure-ASCII control: "Dr." followed by a non-sentence-starter must not
    // produce a boundary, so exactly one boundary (end of text) is expected.
    let ascii = "Then Dr. Smith arrived.";
    let ascii_boundaries = reference(ascii, "en");
    assert_eq!(
        ascii_boundaries.len(),
        1,
        "control case: expected only the final boundary, got {ascii_boundaries:?}"
    );

    // Same sentence preceded by multi-byte characters. The boundary set must
    // have the same shape: exactly one boundary, at the end of the text.
    let multibyte = "Café note: Dr. Smith arrived.";
    let multibyte_boundaries = reference(multibyte, "en");
    assert_eq!(
        multibyte_boundaries.len(),
        1,
        "multi-byte prefix broke abbreviation detection, got {multibyte_boundaries:?}"
    );
    assert_eq!(multibyte_boundaries[0], multibyte.len());
}
