//! Regression tests for unbalanced-enclosure robustness.
//!
//! Two related bugs found while verifying against real texts (Aozora
//! Bunko's 吾輩は猫である, Project Gutenberg prose):
//!
//! 1. The line-start ")" / "）" list-item suppression rules matched kanji and
//!    kana as "alnum", so the closing paren of ordinary parentheticals like
//!    （例） near a line start was suppressed. That left the paren depth at
//!    +1 forever and silenced every following boundary — the novel yielded
//!    zero sentences.
//! 2. A closing delimiter without a matching opener (bare list markers like
//!    "1)", editorial artifacts) drove the depth negative, which the reduce
//!    phase treated as "inside an enclosure", again suppressing the rest of
//!    the document.
//!
//! Fixes: the suppression rules were removed, and negative asymmetric depth
//! no longer suppresses boundaries.

use sakurs_core::{Config, Input, SentenceProcessor};

fn boundaries(text: &str, lang: &str) -> Vec<usize> {
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

/// A parenthetical near the line start must not unbalance the depth: both
/// sentences end at depth zero.
#[test]
fn ja_parenthetical_near_line_start() {
    let text = "（例）吾輩は猫である。名前はまだ無い。";
    let b = boundaries(text, "ja");
    assert_eq!(b.len(), 2, "expected both sentences, got {b:?}");
}

/// Same shape mid-line but within the first ten characters of the line,
/// which used to trigger the list-item heuristic as well.
#[test]
fn ja_parenthetical_mid_line() {
    let text = "これは（例）だが吾輩は猫である。次の文だ。";
    let b = boundaries(text, "ja");
    assert_eq!(b.len(), 2, "expected both sentences, got {b:?}");
}

/// A bare list marker's unmatched "）" must not suppress the rest of the
/// document.
#[test]
fn ja_bare_list_marker_does_not_poison() {
    let text = "1）最初の項目である。次の文も切れる。";
    let b = boundaries(text, "ja");
    assert_eq!(b.len(), 2, "expected both sentences, got {b:?}");
}

/// English line-start parenthetical: both boundaries must survive.
#[test]
fn en_parenthetical_at_line_start() {
    let text = "(note) He arrived early. She left late.";
    let b = boundaries(text, "en");
    assert_eq!(b.len(), 2, "expected both sentences, got {b:?}");
}

/// English bare list marker: the unmatched ")" must not poison the rest.
#[test]
fn en_bare_list_marker_does_not_poison() {
    let text = "a) First item sentence. Second sentence still splits.";
    let b = boundaries(text, "en");
    assert_eq!(b.len(), 2, "expected both sentences, got {b:?}");
}

/// Balanced parentheses must still suppress boundaries inside them.
#[test]
fn balanced_parentheses_still_suppress() {
    let text = "He said (Hello. World). Done.";
    let b = boundaries(text, "en");
    // No boundary after "Hello." (inside parens); boundaries after ")." and
    // "Done." only.
    assert_eq!(b.len(), 2, "expected two boundaries, got {b:?}");
    assert!(
        !b.contains(&15),
        "boundary inside parentheses must stay suppressed: {b:?}"
    );
}
