//! Test abbreviation handling in API layer

use sakurs_api::{process_text, Config};

#[test]
fn test_api_abbreviations() {
    // Test simple case
    let output = process_text("Dr. Smith").unwrap();
    eprintln!("Dr. Smith -> {} boundaries", output.boundaries.len());
    for (i, b) in output.boundaries.iter().enumerate() {
        eprintln!("  [{}] offset={}, kind={}", i, b.byte_offset, b.kind);
    }

    // Dr. should not create a boundary
    assert_eq!(output.boundaries.len(), 0, "Dr. should be an abbreviation");

    // Test with real sentence
    let output2 = process_text("Dr. Smith went home.").unwrap();
    eprintln!(
        "\nDr. Smith went home. -> {} boundaries",
        output2.boundaries.len()
    );
    for (i, b) in output2.boundaries.iter().enumerate() {
        eprintln!("  [{}] offset={}, kind={}", i, b.byte_offset, b.kind);
    }

    // Should have one boundary at the end
    assert_eq!(output2.boundaries.len(), 1);
    assert_eq!(output2.boundaries[0].byte_offset, 20); // After "home."
}

#[test]
fn test_api_usa() {
    let output = process_text("U.S.A.").unwrap();
    eprintln!("U.S.A. -> {} boundaries", output.boundaries.len());
    for (i, b) in output.boundaries.iter().enumerate() {
        eprintln!("  [{}] offset={}, kind={}", i, b.byte_offset, b.kind);
    }

    // All dots should be abbreviations
    assert_eq!(
        output.boundaries.len(),
        0,
        "U.S.A. should not have boundaries"
    );
}

#[test]
fn test_api_with_config() {
    let config = Config::builder()
        .language("en")
        .unwrap()
        .build_processor()
        .unwrap();

    let output = config.process_text("Dr. Smith").unwrap();
    eprintln!(
        "Config processor: Dr. Smith -> {} boundaries",
        output.boundaries.len()
    );

    assert_eq!(output.boundaries.len(), 0);
}
