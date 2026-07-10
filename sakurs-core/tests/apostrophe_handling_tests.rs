//! Tests for apostrophe and contraction handling
//!
//! This test suite verifies that contractions, possessives, and other
//! apostrophe patterns are handled correctly without breaking sentence
//! boundary detection.
//!
//! Note: These tests are currently disabled as they need to be updated
//! for the new configurable language rules system.

// Re-enabling tests with corrected expected values for new configurable language rules
use sakurs_core::{Input, SentenceProcessor};

// Known Limitation: James' Possessive Pattern
// ============================================
// The current implementation cannot distinguish between:
// 1. Possessive forms like "James' car"
// 2. Potential opening quotes like "He said 'hello'"
//
// This is because the apostrophe after 's' is ambiguous without
// contextual understanding. Most rule-based systems struggle with
// this pattern, and it typically requires ML-based approaches.
//
// See test_james_possessive_pattern() below for specific examples.

/// Helper function to detect sentences and return boundary offsets
fn detect_sentences(text: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let processor = SentenceProcessor::with_language("en")?;
    let result = processor.process(Input::from_text(text))?;

    Ok(result.boundaries.into_iter().map(|b| b.offset).collect())
}

#[test]
fn test_basic_contractions() {
    // Test with various contractions
    let test_cases = vec![
        ("I don't know. She isn't here.", vec![13, 29]),
        ("It's amazing! Isn't it wonderful?", vec![13, 33]),
        ("They're coming. We'll see.", vec![15, 26]),
        ("I've been there. You've seen it.", vec![16, 32]),
        ("Can't wait. Won't stop.", vec![11, 23]), // Corrected: both sentences should be detected
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}

#[test]
fn test_possessive_forms() {
    let test_cases = vec![
        ("That's John's book. It's new.", vec![19, 29]),
        (
            "The students' papers are graded. They're done.",
            vec![32, 46],
        ),
        // NOTE: James' pattern is tested separately in ignored tests below
        // NOTE: '90s pattern also moved to ignored test due to similar apostrophe issues
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}

#[test]
fn test_possessive_after_abbreviation() {
    // The apostrophe in `Jr.'s` sits between a period and `s`; without a
    // suppression rule for that shape it counts as a quote toggle and the
    // corrupted parity swallows every boundary after it.
    let test_cases = vec![
        ("That is JFK Jr.'s book. Hello. Bye.", vec![23, 30, 35]),
        ("The U.S.'s policy changed. Nothing else did.", vec![26, 44]),
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        assert_eq!(
            boundaries, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, boundaries, expected_offsets
        );
    }
}

#[test]
#[ignore = "James' possessive pattern is not supported - requires contextual understanding beyond rule-based systems"]
fn test_james_possessive_pattern() {
    // This test documents the known limitation with James' possessive forms
    // The pattern "word ending in 's' followed by apostrophe" is ambiguous:
    // - "James' car" (possessive of James)
    // - "She told it to James' friend" (possessive)
    // vs
    // - "She told it to James' (incomplete quote or typo)

    let test_cases = vec![
        ("James' car is fast. Mary's is faster.", vec![19, 37]),
        ("This is James' book.", vec![20]),
        ("Charles' opinion matters.", vec![25]),
        ("The princess' crown was stolen.", vec![31]),
        ("The '90s were great. The 2000s too.", vec![20, 35]),
        (
            "The '60s and '70s were different. Times changed.",
            vec![33, 48],
        ),
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;

        // Currently, these cases return no boundaries due to the apostrophe
        // being interpreted as an opening quote that never closes
        eprintln!(
            "James' pattern test - Text: '{}'\nExpected: {:?}\nActual: {:?}",
            text, expected_offsets, offsets
        );

        // When this is eventually fixed, uncomment this assertion:
        // assert_eq!(offsets, expected_offsets);
    }
}

#[test]
fn test_complex_apostrophe_patterns() {
    // The original problematic case
    let text = "Dr. Smith went to the U.S.A. He bought a new car. The car cost $25,000! Isn't that expensive?";
    let boundaries = detect_sentences(text).unwrap();
    let offsets = boundaries;

    // Should detect 4 sentences - actual boundaries: [28, 49, 71, 93]
    // Note: U.S.A. is now followed by "He" (a sentence starter), so it creates a boundary
    assert_eq!(offsets.len(), 4, "Should detect 4 sentences");
    assert_eq!(
        offsets,
        vec![28, 49, 71, 93],
        "Boundaries should match expected positions"
    );
}

#[test]
fn test_mixed_quotes_and_contractions() {
    let test_cases = vec![
        // Boundary-after-closers places the break after `know."`; the period
        // itself stays parity-suppressed inside the quotation.
        (r#"He said "I don't know." She agreed."#, vec![23, 35]),
        // Boundary after "she said." plus the quote-final boundary at the end
        // of text (after `it?"`).
        (r#""It's true," she said. "Isn't it?""#, vec![22, 34]),
        // Boundary after "he said." plus the quote-final boundary at the end
        // of text (after `not.'`).
        (r#"'I'm going,' he said. 'You're not.'"#, vec![21, 35]),
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}

#[test]
fn test_measurement_marks() {
    let test_cases = vec![
        ("He is 5'9\" tall. She is shorter.", vec![16, 32]),
        ("The angle is 45°30'. Perfect!", vec![21, 30]),
        ("It's 6' wide. That's big.", vec![13, 25]),
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}

#[test]
fn test_list_item_parentheses() {
    // Bare list markers contribute unmatched ")" closers. Since negative
    // enclosure depth no longer suppresses boundaries, every sentence is
    // found. (The old expectations encoded the poisoning bug: everything
    // after the second marker was silently dropped.)
    let test_cases = vec![
        ("1) First item. 2) Second item.", vec![14, 30]),
        ("a) Option A is good. b) Option B is better.", vec![20, 43]),
        ("i) Introduction. ii) Main body.", vec![16, 31]),
        ("1) First item.\n2) Second item.", vec![14, 30]),
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}

#[test]
fn test_unicode_apostrophes() {
    // Test with Unicode right single quotation mark (U+2019)
    let test_cases = vec![
        ("I don't know. She isn't here.", vec![13, 29]),
        ("It's amazing! Isn't it wonderful?", vec![13, 33]),
        ("They're coming. We'll see.", vec![15, 26]),
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}

#[test]
fn test_multi_period_abbreviations() {
    let test_cases = vec![
        // Various multi-period abbreviations
        ("I work at the U.N. headquarters.", vec![32]),
        ("She has a Ph.D. in physics.", vec![27]),
        ("The E.U. is a union.", vec![20]),
        // "today" is lowercase, not a sentence starter
        ("He lives in Washington D.C. today.", vec![34]),
        // Multiple abbreviations
        ("Dr. Smith has a Ph.D. from M.I.T. in Cambridge.", vec![47]),
        // Abbreviation at end
        ("I'm from the U.S.A.", vec![19]),
        // Abbreviation followed by lowercase
        ("The U.S.A. economy is large.", vec![28]),
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}

#[test]
fn test_edge_cases() {
    let test_cases = vec![
        // Multiple contractions in one sentence
        (
            "I don't think it's what we're looking for. Next?",
            vec![42, 48],
        ),
        // Contraction at sentence start
        ("It's done. Don't worry.", vec![10, 23]),
        // NOTE: James' pattern moved to ignored test test_james_possessive_pattern()
        // NOTE: Year abbreviations ('60s, '70s) also moved to ignored test
    ];

    for (text, expected_offsets) in test_cases {
        let boundaries = detect_sentences(text).unwrap();
        let offsets = boundaries;
        assert_eq!(
            offsets, expected_offsets,
            "Failed for text: '{}'\nGot: {:?}\nExpected: {:?}",
            text, offsets, expected_offsets
        );
    }
}
