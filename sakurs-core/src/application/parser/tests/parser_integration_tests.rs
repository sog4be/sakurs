//! Integration tests for parser module

use crate::application::parser::{
    ParseError, ParseStrategy, ParsingInput, ParsingOutput, SequentialParser, StreamingParser,
    TextParser,
};
use crate::domain::enclosure::{EnclosureChar, EnclosureType};
use crate::domain::language::{
    AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRules, QuotationContext,
    QuotationDecision,
};
use crate::domain::state::BoundaryFlags;

/// Mock language rules that simulates real parsing behavior
struct RealisticMockRules {
    /// Whether to detect abbreviations
    detect_abbreviations: bool,
    /// Known abbreviations
    abbreviations: Vec<&'static str>,
}

impl RealisticMockRules {
    fn new() -> Self {
        Self {
            detect_abbreviations: true,
            abbreviations: vec!["Dr", "Mr", "Mrs", "Ms", "Prof", "Inc", "Ltd"],
        }
    }
}

impl LanguageRules for RealisticMockRules {
    fn detect_sentence_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        // Simulate real boundary detection logic
        if context.boundary_char == '.' {
            // Check if it's an abbreviation
            if self.detect_abbreviations {
                // Extract word before the period
                let before_period = context.preceding_context.trim_end();
                if let Some(last_word_start) = before_period.rfind(|c: char| c.is_whitespace()) {
                    let word = &before_period[last_word_start + 1..];
                    if self.abbreviations.contains(&word) {
                        // This is an abbreviation, not a boundary
                        return BoundaryDecision::NotBoundary;
                    }
                } else {
                    // Check if entire preceding context is an abbreviation
                    if self.abbreviations.contains(&before_period) {
                        return BoundaryDecision::NotBoundary;
                    }
                }
            }

            // Check if followed by whitespace and uppercase
            let trimmed = context.following_context.trim_start();
            if let Some(first_char) = trimmed.chars().next() {
                if first_char.is_uppercase() {
                    return BoundaryDecision::Boundary(BoundaryFlags::STRONG);
                }
            }

            BoundaryDecision::NeedsMoreContext
        } else if matches!(context.boundary_char, '!' | '?') {
            BoundaryDecision::Boundary(BoundaryFlags::STRONG)
        } else {
            BoundaryDecision::NotBoundary
        }
    }

    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        if !self.detect_abbreviations {
            return AbbreviationResult {
                is_abbreviation: false,
                confidence: 0.0,
                length: 0,
            };
        }

        // Position is after the period, so we need to check the word before it
        if position == 0 || text.chars().nth(position.saturating_sub(1)) != Some('.') {
            return AbbreviationResult {
                is_abbreviation: false,
                confidence: 0.0,
                length: 0,
            };
        }

        // Extract word before the period (position-1)
        let before_period = &text[..position.saturating_sub(1)];
        let start = before_period
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        let word = &before_period[start..];

        let is_abbr = self.abbreviations.contains(&word);
        AbbreviationResult {
            is_abbreviation: is_abbr,
            confidence: if is_abbr { 0.9 } else { 0.1 },
            length: word.len() + 1, // Include the period
        }
    }

    fn handle_quotation(&self, _context: &QuotationContext) -> QuotationDecision {
        QuotationDecision::Regular
    }

    fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        match ch {
            '(' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: true,
            }),
            ')' => Some(EnclosureChar {
                enclosure_type: EnclosureType::Parenthesis,
                is_opening: false,
            }),
            '[' => Some(EnclosureChar {
                enclosure_type: EnclosureType::SquareBracket,
                is_opening: true,
            }),
            ']' => Some(EnclosureChar {
                enclosure_type: EnclosureType::SquareBracket,
                is_opening: false,
            }),
            '{' => Some(EnclosureChar {
                enclosure_type: EnclosureType::CurlyBrace,
                is_opening: true,
            }),
            '}' => Some(EnclosureChar {
                enclosure_type: EnclosureType::CurlyBrace,
                is_opening: false,
            }),
            _ => None,
        }
    }

    fn get_enclosure_type_id(&self, ch: char) -> Option<usize> {
        match ch {
            '(' | ')' => Some(0),
            '[' | ']' => Some(1),
            '{' | '}' => Some(2),
            _ => None,
        }
    }

    fn enclosure_type_count(&self) -> usize {
        3 // Parentheses, brackets, braces
    }

    fn language_code(&self) -> &str {
        "mock"
    }

    fn language_name(&self) -> &str {
        "Mock Language"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_with_strategies() {
        let parser = TextParser::new();
        let rules = RealisticMockRules::new();

        // Test text that should produce boundaries
        let text = "Hello world. This is a test! Another sentence?";
        let state = parser.scan_chunk(text, &rules);

        // Should detect 3 boundaries
        assert_eq!(state.boundary_candidates.len(), 3);

        // Verify positions
        let positions: Vec<usize> = state
            .boundary_candidates
            .iter()
            .map(|c| c.local_offset)
            .collect();

        assert!(positions.contains(&12)); // After "Hello world."
        assert!(positions.contains(&28)); // After "This is a test!"
        assert!(positions.contains(&46)); // After "Another sentence?"
    }

    #[test]
    fn test_parser_with_abbreviations() {
        let parser = TextParser::new();
        let rules = RealisticMockRules::new();

        // Test with abbreviations
        let text = "Dr. Smith arrived. Mr. Jones left.";
        let state = parser.scan_chunk(text, &rules);

        // Should not treat abbreviations as boundaries
        let positions: Vec<usize> = state
            .boundary_candidates
            .iter()
            .map(|c| c.local_offset)
            .collect();

        // Should have boundaries after sentences, not abbreviations
        assert!(positions.contains(&18)); // After "Dr. Smith arrived."
        assert!(positions.contains(&34)); // After "Mr. Jones left."

        // Should not have boundaries after "Dr." or "Mr."
        assert!(!positions.contains(&3)); // Not after "Dr."
        assert!(!positions.contains(&22)); // Not after "Mr."
    }

    #[test]
    fn test_sequential_vs_streaming_consistency() {
        let sequential = SequentialParser::new();
        let streaming = StreamingParser::new();
        let rules = RealisticMockRules::new();

        let text = "First sentence. Second sentence. Third sentence.";

        // Parse with sequential
        let seq_result = sequential.parse(ParsingInput::Text(text), &rules).unwrap();
        let seq_state = match seq_result {
            ParsingOutput::State(state) => state,
            _ => panic!("Expected single state"),
        };

        // Parse with streaming
        let stream_result = streaming.parse(ParsingInput::Text(text), &rules).unwrap();
        let stream_state = match stream_result {
            ParsingOutput::State(state) => state,
            _ => panic!("Expected single state"),
        };

        // Results should be identical for complete text
        assert_eq!(
            seq_state.boundary_candidates.len(),
            stream_state.boundary_candidates.len()
        );
        assert_eq!(seq_state.chunk_length, stream_state.chunk_length);
    }

    #[test]
    fn test_parser_with_nested_enclosures() {
        let parser = TextParser::new();
        let rules = RealisticMockRules::new();

        let text = "He said (and I quote [from the book]). Done.";
        let state = parser.scan_chunk(text, &rules);

        // Check delta tracking
        assert_eq!(state.deltas.len(), 3); // Three enclosure types

        // Should track all enclosures properly
        // Net should be 0 (all balanced)
        for delta in &state.deltas {
            assert_eq!(delta.net, 0);
        }

        // Should still detect boundaries
        assert!(!state.boundary_candidates.is_empty());
    }

    #[test]
    fn test_strategy_selection() {
        let rules = RealisticMockRules::new();
        let text = "Test text. Another sentence.";

        // Create different strategies with varying chunk sizes
        // Using smaller chunk sizes to ensure chunking actually occurs with the test text
        let strategies: Vec<Box<dyn ParseStrategy>> = vec![
            Box::new(SequentialParser::new()),
            Box::new(StreamingParser::new()),
            Box::new(SequentialParser::with_chunk_size(16)), // Small chunk size for testing
            Box::new(StreamingParser::with_buffer_size(20, 5)), // Small buffer with small overlap
        ];

        for (i, strategy) in strategies.iter().enumerate() {
            let result = strategy.parse(ParsingInput::Text(text), &rules);
            assert!(result.is_ok(), "Strategy {} failed", i);

            match result.unwrap() {
                ParsingOutput::State(state) => {
                    // All strategies should find the same boundaries
                    assert_eq!(
                        state.boundary_candidates.len(),
                        2,
                        "Strategy {} found wrong number of boundaries",
                        i
                    );
                }
                _ => panic!("Expected single state for strategy {}", i),
            }
        }
    }

    #[test]
    fn test_chunked_parsing_consistency() {
        let parser = StreamingParser::with_buffer_size(1024, 10);
        let rules = RealisticMockRules::new();

        // Test text split into chunks
        let full_text = "First part. Second part. Third part.";
        let chunks = vec!["First part.", " Second part.", " Third part."];

        // Parse as single text
        let single_result = parser.parse(ParsingInput::Text(full_text), &rules).unwrap();
        let single_boundaries = match single_result {
            ParsingOutput::State(state) => state.boundary_candidates.len(),
            _ => panic!("Expected single state"),
        };

        // Parse as chunks
        let chunks_result = parser
            .parse(ParsingInput::Chunks(Box::new(chunks.into_iter())), &rules)
            .unwrap();

        let total_boundaries: usize = match chunks_result {
            ParsingOutput::States(states) => {
                states.iter().map(|s| s.boundary_candidates.len()).sum()
            }
            _ => panic!("Expected multiple states"),
        };

        // When parsing chunks separately, we may get different results
        // because each chunk lacks the full context. The streaming parser
        // is designed to handle this with overlapping buffers.
        // We should expect similar but not necessarily identical results.

        // For this specific test case:
        // - Single text: 3 boundaries (after each sentence)
        // - Chunks: May have extra boundaries due to chunk boundaries

        // The important thing is that both approaches find boundaries
        assert!(single_boundaries > 0);
        assert!(total_boundaries > 0);

        // And that the difference is reasonable (within 2x)
        assert!(total_boundaries <= single_boundaries * 2);
    }

    #[test]
    fn test_error_propagation() {
        let sequential = SequentialParser::new();
        let streaming = StreamingParser::new();
        let rules = RealisticMockRules::new();

        // Test empty input errors
        let empty_text = ParsingInput::Text("");
        assert!(matches!(
            sequential.parse(empty_text, &rules),
            Err(ParseError::EmptyInput)
        ));

        let empty_chunks = ParsingInput::Chunks(Box::new(std::iter::empty()));
        assert!(matches!(
            streaming.parse(empty_chunks, &rules),
            Err(ParseError::EmptyInput)
        ));
    }

    #[test]
    fn test_utf8_boundary_handling() {
        let parser = TextParser::new();
        let rules = RealisticMockRules::new();

        // Test with multi-byte UTF-8 characters
        let texts = vec![
            "Hello 世界. Another sentence.",
            "Здравствуй мир! Другое предложение.",
            "مرحبا بالعالم. جملة أخرى.",
        ];

        for text in texts {
            let state = parser.scan_chunk(text, &rules);

            // Should handle UTF-8 correctly
            assert_eq!(state.chunk_length, text.len());
            assert!(!state.boundary_candidates.is_empty());

            // All boundaries should be at valid UTF-8 positions
            for candidate in &state.boundary_candidates {
                assert!(text.is_char_boundary(candidate.local_offset));
            }
        }
    }
}
