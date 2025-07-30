use sakurs_api::{process_with_processor, SentenceProcessor};

#[test]
fn test_ip_address_api() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "192.168.1.1";

    let result = process_with_processor(&processor, text).unwrap();

    println!("Text: {:?}", text);
    println!("Boundaries found: {}", result.boundaries.len());
    for (i, b) in result.boundaries.iter().enumerate() {
        println!(
            "  [{}] byte_offset={}, text up to: '{}'",
            i,
            b.byte_offset,
            &text[..b.byte_offset]
        );
    }

    // IP address should not have any boundaries
    assert_eq!(result.boundaries.len(), 0);
}
