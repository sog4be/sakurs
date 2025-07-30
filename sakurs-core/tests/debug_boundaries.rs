//! Debug boundary generation

use sakurs_core::language::{BoundaryDecision, BoundaryStrength, DotRole, EnclosureInfo};
use sakurs_core::{run, Class, LanguageRules};

struct DebugRules;

impl LanguageRules for DebugRules {
    fn is_terminator_char(&self, ch: char) -> bool {
        matches!(ch, '.' | '!' | '?')
    }

    fn enclosure_info(&self, ch: char) -> Option<EnclosureInfo> {
        match ch {
            '(' => Some(EnclosureInfo {
                type_id: 0,
                delta: 1,
                symmetric: false,
            }),
            ')' => Some(EnclosureInfo {
                type_id: 0,
                delta: -1,
                symmetric: false,
            }),
            _ => None,
        }
    }

    fn dot_role(&self, _prev: Option<char>, _next: Option<char>) -> DotRole {
        DotRole::Ordinary
    }

    fn boundary_decision(
        &self,
        text: &str,
        pos: usize,
        terminator_char: char,
        _prev_char: Option<char>,
        _next_char: Option<char>,
    ) -> BoundaryDecision {
        if pos == 0 || pos > text.len() {
            return BoundaryDecision::Reject;
        }

        // Check if it's a dot terminator
        if terminator_char == '.' {
            // Check for abbreviations
            if self.is_abbreviation(text, pos - 1) {
                eprintln!(
                    "boundary_decision: rejecting at {} due to abbreviation",
                    pos
                );
                return BoundaryDecision::Reject;
            }
        }

        BoundaryDecision::Accept(BoundaryStrength::Strong)
    }
    fn classify_char(&self, ch: char) -> Class {
        Class::from_char(ch)
    }

    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool {
        if dot_pos == 0 {
            return false;
        }

        // Look for word boundary before the potential abbreviation
        let mut start = dot_pos;
        while start > 0 {
            let ch = text.chars().nth(start - 1).unwrap();
            if !ch.is_alphanumeric() && ch != '.' {
                break;
            }
            start -= 1;
        }

        // Extract the potential abbreviation
        let abbrev = &text[start..dot_pos];

        // Check known abbreviations
        let is_known = matches!(abbrev, "Dr" | "Mr" | "U" | "S" | "A" | "U.S" | "U.S.A");

        eprintln!(
            "is_abbreviation: '{}' at {} -> {} (abbrev='{}')",
            text, dot_pos, is_known, abbrev
        );

        is_known
    }

    fn abbrev_match(&self, abbrev: &str) -> bool {
        // Check single letter abbreviations
        if abbrev.len() == 1 && abbrev.chars().all(|c| c.is_ascii_alphabetic()) {
            eprintln!("abbrev_match: '{}' -> single letter", abbrev);
            return true;
        }

        let result = matches!(abbrev, "Dr" | "Mr" | "U" | "S" | "A");
        eprintln!("abbrev_match: '{}' -> {}", abbrev, result);
        result
    }

    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        match ch {
            '(' => Some((0, true)),
            ')' => Some((0, false)),
            _ => None,
        }
    }

    fn is_terminator(&self, ch: char) -> bool {
        matches!(ch, '.' | '!' | '?')
    }
}

#[test]
fn debug_dr_smith() {
    let rules = DebugRules;

    let text = "Dr. Smith";
    eprintln!("\n=== Testing: '{}' ===", text);
    let boundaries = run(text, &rules).unwrap();
    eprintln!("Boundaries: {:?}", boundaries);
    for (i, b) in boundaries.iter().enumerate() {
        eprintln!("  [{}] offset={}, kind={:?}", i, b.byte_offset, b.kind);
    }

    // Should have no boundaries (Dr. is abbreviation)
    assert_eq!(
        boundaries.len(),
        0,
        "Expected no boundaries for 'Dr. Smith'"
    );
}

#[test]
fn debug_usa() {
    let rules = DebugRules;

    let text = "U.S.A.";
    eprintln!("\n=== Testing: '{}' ===", text);
    let boundaries = run(text, &rules).unwrap();
    eprintln!("Boundaries: {:?}", boundaries);
    for (i, b) in boundaries.iter().enumerate() {
        eprintln!("  [{}] offset={}, kind={:?}", i, b.byte_offset, b.kind);
    }

    // All dots should be abbreviations
    assert_eq!(boundaries.len(), 0, "Expected no boundaries for 'U.S.A.'");
}

#[test]
fn debug_sentence_with_abbrev() {
    let rules = DebugRules;

    let text = "Dr. Smith went to the U.S.A. yesterday.";
    eprintln!("\n=== Testing: '{}' ===", text);
    let boundaries = run(text, &rules).unwrap();
    eprintln!("Boundaries: {:?}", boundaries);
    for (i, b) in boundaries.iter().enumerate() {
        eprintln!(
            "  [{}] offset={}, kind={:?}, text up to: '{}'",
            i,
            b.byte_offset,
            b.kind,
            &text[..b.byte_offset]
        );
    }

    // Should have only one boundary at the end
    assert_eq!(boundaries.len(), 1, "Expected one boundary at the end");
    assert_eq!(boundaries[0].byte_offset, text.len()); // After final period
}
