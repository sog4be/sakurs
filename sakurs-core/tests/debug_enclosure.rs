use sakurs_core::{Input, SentenceProcessor};

#[test]
fn debug_enclosure_failure() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "He said \"I don't know.\" She agreed.";
    let result = processor.process(Input::from_text(text)).unwrap();

    println!("Text: {}", text);
    println!("Found {} boundaries", result.boundaries.len());
    for (i, boundary) in result.boundaries.iter().enumerate() {
        let char_at_boundary = if boundary.offset > 0 {
            text.chars().nth(boundary.offset - 1).unwrap_or(' ')
        } else {
            ' '
        };
        println!(
            "  Boundary {}: offset={}, char='{}'",
            i, boundary.offset, char_at_boundary
        );
    }

    // Expected: vec![35] (only final boundary)
    let expected = vec![35];
    println!("Expected: {:?}", expected);
    println!(
        "Actual: {:?}",
        result
            .boundaries
            .iter()
            .map(|b| b.offset)
            .collect::<Vec<_>>()
    );

    // For now, we just want to see the debug output
    // assert_eq!(result.boundaries.iter().map(|b| b.offset).collect::<Vec<_>>(), expected);
}
