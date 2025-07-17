use sakurs_core::{Input, SentenceProcessor};

#[test]
fn test_symmetric_quote_basic() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "Simple test. Next sentence.";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should detect both boundaries
    assert_eq!(result.boundaries.len(), 2);
    assert_eq!(result.boundaries[0].offset, 12); // After "Simple test."
    assert_eq!(result.boundaries[1].offset, 27); // After "Next sentence."
}

#[test]
fn test_symmetric_quote_with_quotes() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "He said \"Hello.\" She agreed.";
    let result = processor.process(Input::from_text(text)).unwrap();

    // Should detect boundary only at the end (quote content should be suppressed)
    assert_eq!(result.boundaries.len(), 1);
    assert_eq!(result.boundaries[0].offset, 28); // After "She agreed."
}
