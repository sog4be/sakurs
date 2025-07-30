use sakurs_engine::SentenceProcessorBuilder;

#[test]
fn test_ip_address_engine() {
    let processor = SentenceProcessorBuilder::new()
        .language("en")
        .build()
        .unwrap();

    let text = "192.168.1.1";

    let boundaries = processor.process(text).unwrap();

    println!("Text: {:?}", text);
    println!("Boundaries found: {}", boundaries.len());
    for (i, b) in boundaries.iter().enumerate() {
        println!(
            "  [{}] byte_offset={}, text up to: '{}'",
            i,
            b.byte_offset,
            &text[..b.byte_offset]
        );
    }

    // IP address should not have any boundaries
    assert_eq!(boundaries.len(), 0);
}
