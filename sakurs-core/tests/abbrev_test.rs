//! Test abbreviation detection

use sakurs_core::language::{BoundaryDecision, BoundaryStrength, DotRole, EnclosureInfo};
use sakurs_core::{run, Class, LanguageRules};

#[derive(Debug)]
struct TestRules;

impl LanguageRules for TestRules {
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

    fn boundary_decision(&self, text: &str, pos: usize) -> BoundaryDecision {
        if pos == 0 || pos > text.len() {
            return BoundaryDecision::Reject;
        }

        // Check if it's a dot at pos-1
        let term_char = text.chars().nth(text[..pos].chars().count() - 1);
        if let Some('.') = term_char {
            // Check for abbreviations
            if self.is_abbreviation(text, pos - 1) {
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
        let before = &text[..dot_pos];

        // Check single letter abbreviations
        if before.len() == 1 && before.chars().all(|c| c.is_ascii_alphabetic()) {
            return true;
        }

        matches!(before, "Dr" | "Mr" | "U.S" | "U.S.A")
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
fn test_dr_abbreviation() {
    let rules = TestRules;

    // Test Dr. Smith
    let text = "Dr. Smith";
    let boundaries = run(text, &rules).unwrap();
    println!("Text: {}", text);
    println!("Boundaries: {:?}", boundaries);
    assert_eq!(boundaries.len(), 0, "Dr. should not create a boundary");
}

#[test]
fn test_abbrev_match() {
    let rules = TestRules;

    assert!(rules.abbrev_match("Dr"), "Dr should be an abbreviation");
    assert!(rules.abbrev_match("U"), "U should be an abbreviation");
    assert!(
        !rules.abbrev_match("Hello"),
        "Hello should not be an abbreviation"
    );
}
