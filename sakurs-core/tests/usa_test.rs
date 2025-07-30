//! Test U.S.A. abbreviation handling

use sakurs_core::language::get_rules;

#[test]
fn test_usa_abbreviation() {
    let rules = get_rules("en").unwrap();

    // Test boundary decision at each dot position
    let text = "U.S.A.";

    // After first dot (position 2)
    let decision1 = rules.boundary_decision(text, 2);
    eprintln!("Decision at pos 2: {:?}", decision1);

    // After second dot (position 4)
    let decision2 = rules.boundary_decision(text, 4);
    eprintln!("Decision at pos 4: {:?}", decision2);

    // After third dot (position 6)
    let decision3 = rules.boundary_decision(text, 6);
    eprintln!("Decision at pos 6: {:?}", decision3);

    // All should be rejected (no boundaries)
    assert!(matches!(
        decision1,
        sakurs_core::language::BoundaryDecision::Reject
    ));
    assert!(matches!(
        decision2,
        sakurs_core::language::BoundaryDecision::Reject
    ));
    assert!(matches!(
        decision3,
        sakurs_core::language::BoundaryDecision::Reject
    ));
}

#[test]
fn test_home_period() {
    let rules = get_rules("en").unwrap();

    // Test "home." - should NOT be an abbreviation
    let text = "went home.";
    let decision = rules.boundary_decision(text, 10); // After the period
    eprintln!("'went home.' decision at pos 10: {:?}", decision);

    // Should be accepted as a boundary
    assert!(matches!(
        decision,
        sakurs_core::language::BoundaryDecision::Accept(_)
    ));
}
