//! Regression tests for byte-offset window slicing around multi-byte
//! characters.
//!
//! The ellipsis exception check builds a ±20-byte regex window around the
//! candidate position. Before the fix this used raw byte arithmetic, so any
//! multi-byte character exactly 20 bytes before (or after) a terminator made
//! processing panic with "byte index N is not a char boundary" — trivially
//! triggered by real prose containing accented names (found while verifying
//! against Project Gutenberg's War and Peace).

use sakurs_core::{Config, Input, SentenceProcessor};

fn process(text: &str, lang: &str) -> Vec<usize> {
    let config = Config::builder()
        .language(lang)
        .expect("language config should load")
        .threads(Some(1))
        .build()
        .expect("config should validate");
    let processor = SentenceProcessor::with_config(config).expect("processor should build");
    let output = processor
        .process(Input::from_text(text.to_string()))
        .expect("processing should succeed");
    output.boundaries.iter().map(|b| b.offset).collect()
}

/// Multi-byte character exactly at the window's start offset: for the third
/// dot of the ellipsis, position - 20 lands inside 'ó'.
#[test]
fn multibyte_char_at_window_start_does_not_panic() {
    let text = format!("aaa\u{f3}{}...", "b".repeat(17));
    let boundaries = process(&text, "en");
    // Reaching here without a panic is the regression check; the ellipsis
    // itself may or may not be a boundary depending on context rules.
    let _ = boundaries;
}

/// Multi-byte character inside the window's end offset: position + 20 lands
/// inside a multi-byte character following the terminator.
#[test]
fn multibyte_char_at_window_end_does_not_panic() {
    for pad in 0..4usize {
        let text = format!(
            "Sentence one ends... {}\u{f3}\u{f3}\u{f3} tail.",
            "x".repeat(pad)
        );
        let _ = process(&text, "en");
    }
}

/// Sweep a multi-byte character across every offset near a terminator so all
/// window-boundary alignments are exercised (both sides, en and ja).
#[test]
fn multibyte_window_alignment_sweep() {
    for lang in ["en", "ja"] {
        for lead in 0..24usize {
            let text = format!(
                "{}\u{f3}{} wait... End.",
                "a".repeat(lead),
                "b".repeat(24 - lead)
            );
            let _ = process(&text, lang);
        }
    }
}
