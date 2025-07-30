use sakurs_core::{emit_push, DeltaScanner};

#[test]
fn test_symmetric_quote_handling() {
    // Test with a simple sentence containing symmetric quotes
    let text = r#"He said "Hello.""#;

    let rules = sakurs_core::language::get_rules("en").unwrap();
    let mut scanner = DeltaScanner::new(rules.as_ref()).unwrap();
    let mut boundaries = Vec::new();

    println!("Testing: {:?}", text);

    for (i, ch) in text.char_indices() {
        println!("\nChar {}: '{}'", i, ch);

        if let Some(info) = rules.enclosure_info(ch) {
            println!("  Enclosure info: {:?}", info);

            // Check what get_enclosure_pair returns
            if let Some((id, is_opening)) = rules.get_enclosure_pair(ch) {
                println!("  get_enclosure_pair: id={}, is_opening={}", id, is_opening);
            }
        }

        scanner.step(ch, &mut emit_push(&mut boundaries)).unwrap();
    }

    println!("\nBoundaries found: {:?}", boundaries);

    // There should be one boundary at the end
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].byte_offset, text.len());
}
