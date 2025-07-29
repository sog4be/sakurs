//! Basic tests for sakurs-delta-core

use sakurs_delta_core::*;

#[test]
fn test_boundary_creation() {
    let boundary = Boundary::new(10, 5, BoundaryKind::Strong);
    assert_eq!(boundary.byte_offset, 10);
    assert_eq!(boundary.char_offset, 5);
    assert_eq!(boundary.kind, BoundaryKind::Strong);
}

#[test]
fn test_class_from_char() {
    assert_eq!(Class::from_char('a'), Class::Alpha);
    assert_eq!(Class::from_char('5'), Class::Digit);
    assert_eq!(Class::from_char('.'), Class::Dot);
    assert_eq!(Class::from_char('!'), Class::Terminator);
    assert_eq!(Class::from_char('('), Class::Open);
    assert_eq!(Class::from_char(')'), Class::Close);
    assert_eq!(Class::from_char(' '), Class::Space);
    assert_eq!(Class::from_char('★'), Class::Other);
}

#[test]
fn test_delta_vec_operations() {
    let mut delta1 = DeltaVec::new(3).unwrap();
    delta1.set(0, 1, 0).unwrap();
    delta1.set(1, -1, -1).unwrap();

    let mut delta2 = DeltaVec::new(3).unwrap();
    delta2.set(0, 2, 0).unwrap();
    delta2.set(1, 1, 0).unwrap();

    let combined = delta1.combine(&delta2).unwrap();
    assert_eq!(combined.get(0), Some((3, 0)));
    assert_eq!(combined.get(1), Some((0, -1)));
}

#[test]
fn test_partial_state_creation() {
    let state = PartialState::new(5).unwrap();
    assert_eq!(state.boundaries.len(), 0);
    assert!(!state.dangling_dot);
    assert!(!state.head_alpha);
}

// Mock language rules for testing
struct TestRules;

impl LanguageRules for TestRules {
    fn classify_char(&self, ch: char) -> Class {
        Class::from_char(ch)
    }

    fn is_abbreviation(&self, _text: &str, _dot_pos: usize) -> bool {
        false
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

    fn max_enclosure_pairs(&self) -> usize {
        1
    }
}

#[test]
fn test_delta_scanner_basic() {
    let rules = TestRules;
    let mut scanner = DeltaScanner::new(&rules).unwrap();
    let mut boundaries = Vec::new();

    // Process a simple sentence
    for ch in "Hello.".chars() {
        scanner.step(ch, &mut emit_push(&mut boundaries)).unwrap();
    }

    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].byte_offset, 6); // After the period
}

#[test]
fn test_delta_scanner_with_parentheses() {
    let rules = TestRules;
    let mut scanner = DeltaScanner::new(&rules).unwrap();
    let mut boundaries = Vec::new();

    // Process text with parentheses
    for ch in "(Hello.) World.".chars() {
        scanner.step(ch, &mut emit_push(&mut boundaries)).unwrap();
    }

    // Should detect only the boundary outside parentheses (inside is suppressed)
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].byte_offset, 15); // After "World."
}
