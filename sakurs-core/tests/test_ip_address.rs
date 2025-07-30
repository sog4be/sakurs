use sakurs_core::run;

#[test]
fn test_ip_address_not_split() {
    let text = "The server at 192.168.1.1 is running.";

    let rules = sakurs_core::language::get_rules("en").unwrap();
    let boundaries = run(text, rules.as_ref()).unwrap();

    println!("Text: {:?}", text);
    println!("Boundaries: {:?}", boundaries);

    // Should have one boundary at the end
    assert_eq!(boundaries.len(), 1);
    assert_eq!(boundaries[0].byte_offset, text.len());
}

#[test]
fn test_ip_address_only() {
    let text = "192.168.1.1";

    let rules = sakurs_core::language::get_rules("en").unwrap();
    let boundaries = run(text, rules.as_ref()).unwrap();

    println!("Text: {:?}", text);
    println!("Boundaries: {:?}", boundaries);

    // Should have no boundaries - IP address doesn't end with punctuation
    assert_eq!(boundaries.len(), 0);
}
