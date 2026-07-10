//! Golden Rules Set (GRS) harness.
//!
//! Runs the Golden Rules from pragmatic_segmenter (see the TOML data files in
//! `tests/golden_rules/` for source and license) against the bundled language
//! configurations. Sentences are produced the same way the adapters do: the
//! text is split at the reported boundaries, each piece is trimmed, and empty
//! pieces are dropped.
//!
//! The set of failing rule ids is pinned per language. The test fails when
//! - a currently-passing rule regresses, or
//! - a pinned-failing rule starts passing (ratchet: update the pin so the
//!   improvement is locked in).
//!
//! Run with `--nocapture` to see the per-rule report and pass rate.

use sakurs_core::{Input, SentenceProcessor};
use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenRules {
    rules: Vec<Rule>,
}

#[derive(Deserialize)]
struct Rule {
    id: u32,
    name: String,
    input: String,
    expected: Vec<String>,
    /// The expected output is not an exact substring of the input, so a pure
    /// boundary detector cannot produce it. Kept in the set (and the score)
    /// for comparability with published GRS results.
    #[serde(default)]
    requires_text_modification: bool,
}

/// Splits `text` at the reported boundaries the way the CLI and Python
/// adapters do: trim each piece, drop empty pieces.
fn split(processor: &SentenceProcessor, text: &str) -> Vec<String> {
    let output = processor
        .process(Input::from_text(text))
        .expect("processing should not fail");
    let mut sentences = Vec::new();
    let mut start = 0;
    for boundary in &output.boundaries {
        let piece = text[start..boundary.offset].trim();
        if !piece.is_empty() {
            sentences.push(piece.to_string());
        }
        start = boundary.offset;
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        sentences.push(tail.to_string());
    }
    sentences
}

fn run_golden_rules(language: &str, data: &str, pinned_failures: &[u32]) {
    let golden: GoldenRules = toml::from_str(data).expect("golden rules TOML should parse");
    let processor =
        SentenceProcessor::with_language(language).expect("bundled language should load");

    let mut failed = Vec::new();
    for rule in &golden.rules {
        let actual = split(&processor, &rule.input);
        if actual == rule.expected {
            println!("PASS {language} #{:>2} {}", rule.id, rule.name);
        } else {
            failed.push(rule.id);
            let note = if rule.requires_text_modification {
                " [requires text modification]"
            } else {
                ""
            };
            println!(
                "FAIL {language} #{:>2} {}{note}\n  input:    {:?}\n  expected: {:?}\n  actual:   {:?}",
                rule.id, rule.name, rule.input, rule.expected, actual
            );
        }
    }

    let total = golden.rules.len();
    let passed = total - failed.len();
    println!(
        "golden rules ({language}): {passed}/{total} passed ({:.2}%), failing: {failed:?}",
        passed as f64 / total as f64 * 100.0
    );

    let regressions: Vec<u32> = failed
        .iter()
        .copied()
        .filter(|id| !pinned_failures.contains(id))
        .collect();
    assert!(
        regressions.is_empty(),
        "rules regressed (previously passing, now failing): {regressions:?}"
    );

    let newly_passing: Vec<u32> = pinned_failures
        .iter()
        .copied()
        .filter(|id| !failed.contains(id))
        .collect();
    assert!(
        newly_passing.is_empty(),
        "rules now pass — remove them from the pinned failure list to lock in \
         the improvement: {newly_passing:?}"
    );
}

/// Pinned failing rules for English. Keep sorted; shrink as fixes land.
///
/// Baseline at harness introduction (v0.2.0 rules): 25/52 passing.
const ENGLISH_PINNED_FAILURES: &[u32] = &[
    4,  // one-letter initials (Jonas E. Smith)
    5,  // one-letter lower abbreviation (p. 55)
    10, // Mt. not in abbreviation list
    12, // Jr. followed by 's
    18, // a.m./P.M. followed by capitalized non-starter
    22, // email addresses
    23, // web addresses
    26, // quote-final terminator (boundary after closing quote)
    27, // !! treated as two boundaries
    28, // ?? treated as two boundaries
    31, // list: 1.) without item-final period
    32, // list: 1.) with item-final period
    33, // list: 1) without item-final period
    35, // list: 1. without item-final period
    36, // list: 1. with item-final period
    37, // list: bullet + number
    38, // list: hyphen + number
    39, // list: alphabetical
    41, // requires text modification (newline removal)
    42, // newline-terminated lowercase list items
    43, // N°. and digit-run coordinates
    44, // Yahoo! mid-sentence (terminator followed by lowercase)
    45, // I. as initial vs boundary
    46, // spaced ellipsis inside curly quotes
    48, // spaced ellipsis ". . . ."
    50, // spaced ellipsis as non-boundary
    51, // 4-dot spaced ellipsis
];

/// Pinned failing rules for Japanese. Keep sorted; shrink as fixes land.
///
/// Baseline at harness introduction (v0.2.0 rules): 4/5 passing.
const JAPANESE_PINNED_FAILURES: &[u32] = &[
    5, // requires text modification (newline removal)
];

#[test]
fn english_golden_rules() {
    run_golden_rules(
        "en",
        include_str!("golden_rules/english.toml"),
        ENGLISH_PINNED_FAILURES,
    );
}

#[test]
fn japanese_golden_rules() {
    run_golden_rules(
        "ja",
        include_str!("golden_rules/japanese.toml"),
        JAPANESE_PINNED_FAILURES,
    );
}
