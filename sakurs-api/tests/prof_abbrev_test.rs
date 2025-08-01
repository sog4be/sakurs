use sakurs_api::{Input, SentenceProcessor};

#[test]
fn test_prof_abbreviation() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Test case with "Prof." abbreviation
    let text = "I met Prof. Smith yesterday.";
    let output = processor.process(Input::from_text(text)).unwrap();

    // Should find one boundary at the end of the sentence
    assert_eq!(output.boundaries.len(), 1);
    assert_eq!(output.boundaries[0].byte_offset, 28); // After the final period
}

#[test]
fn test_corp_abbreviation() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Test case with "Corp." abbreviation
    let text = "She works at Apple Corp. in California.";
    let output = processor.process(Input::from_text(text)).unwrap();

    // Should find one boundary at the end of the sentence
    assert_eq!(output.boundaries.len(), 1);
    assert_eq!(output.boundaries[0].byte_offset, 39); // After the final period
}

#[test]
fn test_mixed_abbreviations() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Test with multiple abbreviations of different lengths
    let text = "Dr. Jones and Prof. Smith work at Tech Corp. together.";
    let output = processor.process(Input::from_text(text)).unwrap();

    // Should find one boundary at the end
    assert_eq!(output.boundaries.len(), 1);
    assert_eq!(output.boundaries[0].byte_offset, 54); // After the final period
}

#[test]
fn test_five_letter_abbreviation() {
    let processor = SentenceProcessor::with_language("en").unwrap();

    // Test edge case: 5-letter abbreviation (max window size)
    let text = "The Assoc. Director will arrive soon.";
    let output = processor.process(Input::from_text(text)).unwrap();

    // Should find one boundary at the end
    assert_eq!(output.boundaries.len(), 1);
    assert_eq!(output.boundaries[0].byte_offset, 37); // After the final period
}
