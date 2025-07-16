//! Debug test for configurable language rules

use sakurs_core::{Input, SentenceProcessor};

#[test]
fn debug_abbreviation_detection() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "Dr. Smith works at Apple Inc. and lives on Main St. in the city.";
    let result = processor.process(Input::from_text(text)).unwrap();
    
    println!("Text: {}", text);
    println!("Found {} boundaries", result.boundaries.len());
    for (i, boundary) in result.boundaries.iter().enumerate() {
        println!("  Boundary {}: offset={}, char={}", 
                 i, boundary.offset, text.chars().nth(boundary.char_offset.saturating_sub(1)).unwrap_or(' '));
        // Print the sentence
        let start = if i == 0 { 0 } else { result.boundaries[i-1].offset };
        println!("    Sentence: '{}'", &text[start..boundary.offset]);
    }
}

#[test]
fn debug_basic_sentences() {
    let processor = SentenceProcessor::with_language("en").unwrap();
    let text = "Hello world. This is a test.";
    let result = processor.process(Input::from_text(text)).unwrap();
    
    println!("\nText: {}", text);
    println!("Found {} boundaries", result.boundaries.len());
    for (i, boundary) in result.boundaries.iter().enumerate() {
        let char_at_boundary = if boundary.offset > 0 {
            text.chars().nth(boundary.offset - 1).unwrap_or(' ')
        } else {
            ' '
        };
        println!("  Boundary {}: offset={}, char='{}', confidence={}", 
                 i, boundary.offset, char_at_boundary, boundary.confidence);
        // Print the sentence
        let start = if i == 0 { 0 } else { result.boundaries[i-1].offset };
        println!("    Sentence: '{}'", &text[start..boundary.offset]);
    }
}