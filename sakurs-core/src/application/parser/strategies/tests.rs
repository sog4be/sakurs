//! Tests for parsing strategies

use super::*;
use crate::domain::language::{
    AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRules, QuotationContext,
    QuotationDecision,
};
use crate::domain::state::BoundaryFlags;
use crate::domain::EnclosureChar;

/// Mock language rules for testing
struct MockParserRules {
    /// Whether to always return boundary
    always_boundary: bool,
    /// Whether to return needs more context
    needs_context: bool,
}

impl MockParserRules {
    fn new() -> Self {
        Self {
            always_boundary: true,
            needs_context: false,
        }
    }

    fn with_needs_context() -> Self {
        Self {
            always_boundary: false,
            needs_context: true,
        }
    }

    fn never_boundary() -> Self {
        Self {
            always_boundary: false,
            needs_context: false,
        }
    }
}

impl LanguageRules for MockParserRules {
    fn detect_sentence_boundary(&self, _context: &BoundaryContext) -> BoundaryDecision {
        if self.needs_context {
            BoundaryDecision::NeedsMoreContext
        } else if self.always_boundary {
            BoundaryDecision::Boundary(BoundaryFlags::STRONG)
        } else {
            BoundaryDecision::NotBoundary
        }
    }

    fn process_abbreviation(&self, _text: &str, _position: usize) -> AbbreviationResult {
        AbbreviationResult {
            is_abbreviation: false,
            confidence: 0.0,
            length: 0,
        }
    }

    fn handle_quotation(&self, _context: &QuotationContext) -> QuotationDecision {
        QuotationDecision::Regular
    }

    fn get_enclosure_char(&self, _ch: char) -> Option<EnclosureChar> {
        None
    }

    fn get_enclosure_type_id(&self, _ch: char) -> Option<usize> {
        None
    }

    fn enclosure_type_count(&self) -> usize {
        1 // Minimal for testing
    }

    fn language_code(&self) -> &str {
        "test"
    }

    fn language_name(&self) -> &str {
        "Test Language"
    }
}

#[cfg(test)]
mod parse_error_tests {
    use super::*;

    #[test]
    fn test_parse_error_variants() {
        // Test all error variants can be created
        let errors = vec![
            ParseError::InvalidUtf8,
            ParseError::EmptyInput,
            ParseError::ParsingFailed("Test failure".to_string()),
        ];

        for error in &errors {
            // Test Display trait
            let display = format!("{}", error);
            assert!(!display.is_empty());

            // Test Debug trait
            let debug = format!("{:?}", error);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_parse_error_messages() {
        // Test specific error messages
        let utf8_err = ParseError::InvalidUtf8;
        assert_eq!(format!("{}", utf8_err), "Invalid UTF-8 in input");

        let empty_err = ParseError::EmptyInput;
        assert_eq!(format!("{}", empty_err), "Empty input");

        let parsing_err = ParseError::ParsingFailed("Custom error".to_string());
        assert_eq!(format!("{}", parsing_err), "Parsing failed: Custom error");
    }
}

#[cfg(test)]
mod sequential_parser_tests {
    use super::*;

    #[test]
    fn test_sequential_parser_creation() {
        // Test default creation
        let parser = SequentialParser::new();
        assert_eq!(parser.optimal_chunk_size(), 65536);
        assert!(!parser.supports_streaming());

        // Test with custom chunk size
        let parser = SequentialParser::with_chunk_size(1024);
        assert_eq!(parser.optimal_chunk_size(), 1024);

        // Test Default trait
        let parser = SequentialParser::default();
        assert_eq!(parser.optimal_chunk_size(), 65536);
    }

    #[test]
    fn test_sequential_parse_text() {
        let parser = SequentialParser::new();
        let rules = MockParserRules::new();

        // Test with text input
        let input = ParsingInput::Text("Hello world. This is a test.");
        let result = parser.parse(input, &rules).unwrap();

        match result {
            ParsingOutput::State(state) => {
                // Should have parsed the text
                assert!(state.chunk_length > 0);
            }
            ParsingOutput::States(_) => panic!("Expected single state for text input"),
        }
    }

    #[test]
    fn test_sequential_parse_empty_text() {
        let parser = SequentialParser::new();
        let rules = MockParserRules::new();

        // Test with empty text
        let input = ParsingInput::Text("");
        let result = parser.parse(input, &rules);

        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_sequential_parse_chunks() {
        let parser = SequentialParser::new();
        let rules = MockParserRules::new();

        // Test with chunk input
        let chunks = vec!["Hello world.", " This is", " a test."];
        let input = ParsingInput::Chunks(Box::new(chunks.into_iter()));
        let result = parser.parse(input, &rules).unwrap();

        match result {
            ParsingOutput::States(states) => {
                // Should have multiple states
                assert_eq!(states.len(), 3);
                // Each state should have content
                for state in &states {
                    assert!(state.chunk_length > 0);
                }
            }
            ParsingOutput::State(_) => panic!("Expected multiple states for chunk input"),
        }
    }

    #[test]
    fn test_sequential_parse_empty_chunks() {
        let parser = SequentialParser::new();
        let rules = MockParserRules::new();

        // Test with empty chunks
        let chunks: Vec<&str> = vec![];
        let input = ParsingInput::Chunks(Box::new(chunks.into_iter()));
        let result = parser.parse(input, &rules);

        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_sequential_parse_mixed_chunks() {
        let parser = SequentialParser::new();
        let rules = MockParserRules::new();

        // Test with some empty chunks
        let chunks = vec!["Hello", "", "world", "", "."];
        let input = ParsingInput::Chunks(Box::new(chunks.into_iter()));
        let result = parser.parse(input, &rules).unwrap();

        match result {
            ParsingOutput::States(states) => {
                // Should skip empty chunks
                assert_eq!(states.len(), 3);
            }
            _ => panic!("Expected states output"),
        }
    }

    #[test]
    fn test_sequential_parser_trait_impl() {
        let parser = SequentialParser::new();

        // Test trait methods
        assert!(!parser.supports_streaming());
        assert_eq!(parser.optimal_chunk_size(), 65536);

        // Test as trait object
        let parser_ref: &dyn ParseStrategy = &parser;
        assert!(!parser_ref.supports_streaming());
        assert_eq!(parser_ref.optimal_chunk_size(), 65536);
    }
}

#[cfg(test)]
mod streaming_parser_tests {
    use super::*;

    #[test]
    fn test_streaming_parser_creation() {
        // Test default creation
        let parser = StreamingParser::new();
        assert_eq!(parser.optimal_chunk_size(), 1_048_576);
        assert!(parser.supports_streaming());

        // Test with custom buffer size
        let parser = StreamingParser::with_buffer_size(4096, 128);
        assert_eq!(parser.optimal_chunk_size(), 4096);

        // Test Default trait
        let parser = StreamingParser::default();
        assert_eq!(parser.optimal_chunk_size(), 1_048_576);
    }

    #[test]
    fn test_streaming_parse_text() {
        let parser = StreamingParser::new();
        let rules = MockParserRules::new();

        // Test with text input
        let input = ParsingInput::Text("Hello world. This is a test.");
        let result = parser.parse(input, &rules).unwrap();

        match result {
            ParsingOutput::State(state) => {
                // Should have parsed the text
                assert!(state.chunk_length > 0);
            }
            ParsingOutput::States(_) => panic!("Expected single state for text input"),
        }
    }

    #[test]
    fn test_streaming_parse_empty_text() {
        let parser = StreamingParser::new();
        let rules = MockParserRules::new();

        // Test with empty text
        let input = ParsingInput::Text("");
        let result = parser.parse(input, &rules);

        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_streaming_parse_chunks_with_overlap() {
        let parser = StreamingParser::with_buffer_size(1024, 5);
        let rules = MockParserRules::new();

        // Test with chunks that should overlap
        let chunks = vec!["Hello world.", " This is", " a test."];
        let input = ParsingInput::Chunks(Box::new(chunks.into_iter()));
        let result = parser.parse(input, &rules).unwrap();

        match result {
            ParsingOutput::States(states) => {
                // Should have states for each chunk
                assert_eq!(states.len(), 3);
                // Each state should include overlap from previous
                // (except the first one)
                assert!(states[0].chunk_length > 0);
                assert!(states[1].chunk_length > " This is".len());
                assert!(states[2].chunk_length > " a test.".len());
            }
            _ => panic!("Expected states output"),
        }
    }

    #[test]
    fn test_streaming_parse_small_chunks() {
        let parser = StreamingParser::with_buffer_size(1024, 10);
        let rules = MockParserRules::new();

        // Test with chunks smaller than overlap size
        let chunks = vec!["Hi", ".", " ", "Bye", "."];
        let input = ParsingInput::Chunks(Box::new(chunks.into_iter()));
        let result = parser.parse(input, &rules).unwrap();

        match result {
            ParsingOutput::States(states) => {
                // Should handle small chunks correctly
                assert_eq!(states.len(), 5);
                // Small chunks should be preserved entirely as overlap
                for (i, state) in states.iter().enumerate() {
                    assert!(state.chunk_length > 0, "State {} has no content", i);
                }
            }
            _ => panic!("Expected states output"),
        }
    }

    #[test]
    fn test_streaming_parse_empty_chunks() {
        let parser = StreamingParser::new();
        let rules = MockParserRules::new();

        // Test with empty chunks
        let chunks: Vec<&str> = vec![];
        let input = ParsingInput::Chunks(Box::new(chunks.into_iter()));
        let result = parser.parse(input, &rules);

        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_streaming_parser_trait_impl() {
        let parser = StreamingParser::new();

        // Test trait methods
        assert!(parser.supports_streaming());
        assert_eq!(parser.optimal_chunk_size(), 1_048_576);

        // Test as trait object
        let parser_ref: &dyn ParseStrategy = &parser;
        assert!(parser_ref.supports_streaming());
        assert_eq!(parser_ref.optimal_chunk_size(), 1_048_576);
    }

    #[test]
    fn test_streaming_overlap_behavior() {
        let parser = StreamingParser::with_buffer_size(1024, 3);
        let rules = MockParserRules::new();

        // Test overlap preservation
        let chunks = vec!["12345", "67890", "ABCDE"];
        let input = ParsingInput::Chunks(Box::new(chunks.into_iter()));
        let result = parser.parse(input, &rules).unwrap();

        match result {
            ParsingOutput::States(states) => {
                assert_eq!(states.len(), 3);

                // First chunk: just "12345"
                assert_eq!(states[0].chunk_length, 5);

                // Second chunk: "345" (overlap) + "67890" = 8 chars
                assert_eq!(states[1].chunk_length, 8);

                // Third chunk: "890" (overlap) + "ABCDE" = 8 chars
                assert_eq!(states[2].chunk_length, 8);
            }
            _ => panic!("Expected states output"),
        }
    }
}

#[cfg(test)]
mod parse_strategy_tests {
    use super::*;

    #[test]
    fn test_parse_strategy_as_trait_object() {
        // Test that both parsers can be used as trait objects
        let parsers: Vec<Box<dyn ParseStrategy>> = vec![
            Box::new(SequentialParser::new()),
            Box::new(StreamingParser::new()),
        ];

        let rules = MockParserRules::new();
        let text = "Test text.";

        for parser in &parsers {
            // Test parsing
            let input = ParsingInput::Text(text);
            let result = parser.parse(input, &rules);
            assert!(result.is_ok());

            // Test trait methods
            let _ = parser.supports_streaming();
            let _ = parser.optimal_chunk_size();
        }
    }

    #[test]
    fn test_different_rule_behaviors() {
        let parser = SequentialParser::new();

        // Test with always boundary rules
        let rules = MockParserRules::new();
        let input = ParsingInput::Text("Test. Text.");
        let result = parser.parse(input, &rules).unwrap();
        match result {
            ParsingOutput::State(state) => {
                // Should have boundary candidates
                assert!(!state.boundary_candidates.is_empty());
            }
            _ => panic!("Expected state output"),
        }

        // Test with never boundary rules
        let rules = MockParserRules::never_boundary();
        let input = ParsingInput::Text("Test. Text.");
        let result = parser.parse(input, &rules).unwrap();
        match result {
            ParsingOutput::State(state) => {
                // Should have no boundary candidates
                assert!(state.boundary_candidates.is_empty());
            }
            _ => panic!("Expected state output"),
        }

        // Test with needs context rules
        let rules = MockParserRules::with_needs_context();
        let input = ParsingInput::Text("Test. Text.");
        let result = parser.parse(input, &rules).unwrap();
        match result {
            ParsingOutput::State(state) => {
                // Should have weak boundary candidates
                assert!(!state.boundary_candidates.is_empty());
                for candidate in &state.boundary_candidates {
                    assert!(candidate.flags.contains(BoundaryFlags::WEAK));
                }
            }
            _ => panic!("Expected state output"),
        }
    }

    #[test]
    fn test_send_sync_constraints() {
        // Verify that parsers implement Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<SequentialParser>();
        assert_send_sync::<StreamingParser>();

        // Verify trait objects are Send + Sync
        assert_send_sync::<Box<dyn ParseStrategy>>();
    }

    #[test]
    fn test_parsing_unicode_text() {
        let parser = SequentialParser::new();
        let rules = MockParserRules::new();

        // Test with Unicode text
        let texts = vec![
            "Hello 世界。",
            "Привет мир!",
            "مرحبا بالعالم.",
            "🌍 Hello world! 🌎",
        ];

        for text in texts {
            let input = ParsingInput::Text(text);
            let result = parser.parse(input, &rules);
            assert!(result.is_ok(), "Failed to parse: {}", text);

            match result.unwrap() {
                ParsingOutput::State(state) => {
                    assert_eq!(state.chunk_length, text.len());
                }
                _ => panic!("Expected state output"),
            }
        }
    }

    #[test]
    fn test_chunk_iterator_handling() {
        let parser = StreamingParser::new();
        let rules = MockParserRules::new();

        // Test with custom iterator
        let data = vec!["Part1", "Part2", "Part3"];
        let chunks = data.into_iter().filter(|s| !s.is_empty());

        let input = ParsingInput::Chunks(Box::new(chunks));
        let result = parser.parse(input, &rules);
        assert!(result.is_ok());

        match result.unwrap() {
            ParsingOutput::States(states) => {
                assert_eq!(states.len(), 3);
            }
            _ => panic!("Expected states output"),
        }
    }
}
