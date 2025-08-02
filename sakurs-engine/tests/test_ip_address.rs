use sakurs_engine::{Input, SentenceProcessorBuilder};

#[test]
fn test_ip_address_engine() {
    let processor = SentenceProcessorBuilder::new()
        .language("en")
        .build()
        .unwrap();

    let text = "192.168.1.1";

    let output = processor.process(Input::from_text(text)).unwrap();

    println!("Text: {:?}", text);
    println!("Boundaries found: {}", output.boundaries.len());
    for (i, b) in output.boundaries.iter().enumerate() {
        println!(
            "  [{}] byte_offset={}, text up to: '{}'",
            i,
            b.byte_offset,
            &text[..b.byte_offset]
        );
    }

    // IP address should not have any boundaries
    assert_eq!(output.boundaries.len(), 0);
}
