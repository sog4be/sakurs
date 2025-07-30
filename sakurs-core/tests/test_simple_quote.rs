use sakurs_core::run;

#[test]
fn test_simple_double_quote() {
    let text = r#"He said "Hello.""#;

    let rules = sakurs_core::language::get_rules("en").unwrap();
    let boundaries = run(text, rules.as_ref()).unwrap();

    println!("Text: {:?}", text);
    println!("Boundaries: {:?}", boundaries);

    // Should have one boundary at the end
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].byte_offset, text.len());
}

#[test]
fn test_quote_with_period_inside() {
    let text = r#"Parser ready (v2.3 passes.) now."#;

    let rules = sakurs_core::language::get_rules("en").unwrap();
    let boundaries = run(text, rules.as_ref()).unwrap();

    println!("Text: {:?}", text);
    println!("Boundaries: {:?}", boundaries);

    // Should have one boundary at the end
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].byte_offset, text.len());
}
