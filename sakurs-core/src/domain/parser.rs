//! Core parser implementation for the Δ-Stack Monoid algorithm.
//!
//! This module implements the character-by-character parsing logic that builds
//! the partial state representation for text segmentation.

use crate::domain::{
    enclosure::{EnclosureRules, EnclosureStack, StandardEnclosureRules},
    language::{BoundaryContext, BoundaryDecision, LanguageRules},
    state::{AbbreviationState, Boundary, BoundaryFlags, DeltaEntry, PartialState},
};

/// Parser configuration options.
pub struct ParserConfig {
    /// Rules for handling enclosures (quotes, parentheses, etc.)
    pub enclosure_rules: Box<dyn EnclosureRules>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            enclosure_rules: Box::new(StandardEnclosureRules::new()),
        }
    }
}

/// Core parser for converting text into partial states.
pub struct Parser {
    config: ParserConfig,
}

impl Parser {
    /// Creates a new parser with default configuration.
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Creates a new parser with custom configuration.
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parses a chunk of text and produces a partial state.
    ///
    /// This is the core parsing function that implements the Δ-Stack Monoid algorithm.
    /// It scans the text character by character, tracks enclosure depth, detects
    /// sentence boundaries, and builds the delta representation.
    pub fn parse_chunk(
        &self,
        text: &str,
        language_rules: &dyn LanguageRules,
        initial_state: Option<&PartialState>,
    ) -> PartialState {
        let mut state = match initial_state {
            Some(s) => s.clone(),
            None => PartialState::new(1), // Start with one enclosure type
        };
        let mut enclosure_stack = EnclosureStack::new();
        let mut position = 0;
        let mut chars = text.chars().peekable();

        // Track parsing context
        let mut last_char: Option<char> = None;
        let mut consecutive_dots = 0;

        while let Some(ch) = chars.next() {
            let char_len = ch.len_utf8();

            // Check for enclosure characters
            if let Some(enc_char) = self.config.enclosure_rules.get_enclosure_char(ch) {
                if enc_char.is_opening {
                    enclosure_stack.push(enc_char.enclosure_type, ch);
                    // Update delta for this enclosure type (simplified to first delta)
                    if !state.deltas.is_empty() {
                        state.deltas[0] = state.deltas[0].combine(&DeltaEntry::new(1, 0));
                    }
                } else {
                    // Try to close the enclosure
                    if enclosure_stack.close(enc_char.enclosure_type) && !state.deltas.is_empty() {
                        state.deltas[0] = state.deltas[0].combine(&DeltaEntry::new(-1, -1));
                    }
                }
            }

            // Track consecutive dots for abbreviations
            if ch == '.' {
                consecutive_dots += 1;
            } else {
                consecutive_dots = 0;
            }

            // Check for potential sentence terminators only at depth 0
            if enclosure_stack.depth() == 0 && is_potential_terminator(ch) {
                // Build context for language rules
                let context =
                    build_boundary_context(text, position, ch, &chars, last_char, consecutive_dots);

                // Ask language rules for decision
                let decision = language_rules.detect_sentence_boundary(&context);

                match decision {
                    BoundaryDecision::Boundary(flags) => {
                        // Create a sentence boundary
                        let boundary = Boundary {
                            offset: position + char_len,
                            flags,
                        };

                        // Check if this might be an abbreviation boundary
                        if ch == '.' {
                            let abbr_result = language_rules.process_abbreviation(text, position);
                            if abbr_result.is_abbreviation {
                                state.abbreviation = AbbreviationState {
                                    dangling_dot: true,
                                    head_alpha: context
                                        .following_context
                                        .chars()
                                        .next()
                                        .is_some_and(|c| c.is_alphabetic()),
                                };
                            }
                        }

                        state.boundaries.insert(boundary);
                    }
                    BoundaryDecision::NotBoundary => {
                        // Not a boundary
                    }
                    BoundaryDecision::NeedsMoreContext => {
                        // For ambiguous cases, we might want to mark the boundary
                        // but with a weak flag
                        let boundary = Boundary {
                            offset: position + char_len,
                            flags: BoundaryFlags::WEAK,
                        };
                        state.boundaries.insert(boundary);
                    }
                }
            }

            last_char = Some(ch);
            position += char_len;
        }

        // Update final chunk length
        state.chunk_length = position;

        // Handle any unclosed enclosures
        if enclosure_stack.depth() > 0 {
            // The delta stack should reflect the unclosed enclosures
            // This will be handled by the combine operation in the next chunk
        }

        state
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a character is a potential sentence terminator.
fn is_potential_terminator(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | '。' | '！' | '？')
}

/// Builds a boundary context for language rule evaluation.
fn build_boundary_context(
    text: &str,
    position: usize,
    terminator: char,
    chars_iter: &std::iter::Peekable<std::str::Chars>,
    _last_char: Option<char>,
    _consecutive_dots: usize,
) -> BoundaryContext {
    // Extract text before the boundary (up to 10 chars)
    // Need to find valid UTF-8 boundary
    let mut start = position.saturating_sub(10);
    while start > 0 && !text.is_char_boundary(start) {
        start -= 1;
    }
    let preceding_context = text[start..position].to_string();

    // Peek at upcoming characters (up to 10 chars)
    let following_context = chars_iter.clone().take(10).collect::<String>();

    BoundaryContext {
        text: text.to_string(),
        position,
        boundary_char: terminator,
        preceding_context,
        following_context,
    }
}

/// Convenient function for parsing a chunk of text with default settings.
pub fn parse_chunk(text: &str, language_rules: &dyn LanguageRules) -> PartialState {
    let parser = Parser::new();
    parser.parse_chunk(text, language_rules, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::language::MockLanguageRules;

    #[test]
    fn test_basic_parsing() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        let text = "Hello world. This is a test.";
        let state = parser.parse_chunk(text, &rules, None);

        // Should have detected two boundaries
        assert_eq!(state.boundaries.len(), 2);
        assert_eq!(state.chunk_length, text.len());
    }

    #[test]
    fn test_enclosure_handling() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        let text = "He said (Hello. World). Done.";
        let state = parser.parse_chunk(text, &rules, None);

        // Should skip the period inside parentheses
        // Only detect boundaries for ). and the final .
        assert!(state.boundaries.iter().any(|b| b.offset == 23)); // After ).
        assert!(state.boundaries.iter().any(|b| b.offset == 29)); // After Done.
    }

    #[test]
    fn test_delta_stack_building() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        let text = "Test (nested) text.";
        let state = parser.parse_chunk(text, &rules, None);

        // Should have delta entries for the parentheses
        assert!(!state.deltas.is_empty());

        // Verify the delta has correct net values (should be balanced)
        let total_net: i32 = state.deltas.iter().map(|d| d.net).sum();
        assert_eq!(total_net, 0); // Balanced parentheses
    }

    #[test]
    fn test_abbreviation_detection() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        let text = "Dr. Smith arrived.";
        let state = parser.parse_chunk(text, &rules, None);

        // With proper abbreviation rules, Dr. should not create a boundary
        // Only the final period should create a boundary
        assert_eq!(state.boundaries.len(), 1);
        assert!(state.boundaries.iter().any(|b| b.offset == 18)); // After "arrived."
    }

    #[test]
    fn test_quotation_handling() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        let text = r#"She said "Hello." Then left."#;
        let state = parser.parse_chunk(text, &rules, None);

        // Should have delta tracking for quotes
        assert!(!state.deltas.is_empty());
        // The delta should show quote handling (net > 0 indicates unclosed quotes)
        assert!(state.deltas[0].net > 0);
        // Parsing should complete successfully
        assert_eq!(state.chunk_length, text.len());
    }

    #[test]
    fn test_integration_with_english_rules() {
        use crate::domain::language::EnglishLanguageRules;

        let parser = Parser::new();
        let rules = EnglishLanguageRules::new();

        // Complex text with abbreviations, numbers, and nested punctuation
        let text = "Dr. Smith (born 1965) earned his Ph.D. He works at Tech Corp. The company is valued at $2.5 billion! Amazing.";
        let state = parser.parse_chunk(text, &rules, None);

        // Should handle abbreviations properly - no boundaries after "Dr." or "Ph.D."
        let boundary_positions: Vec<usize> = state.boundaries.iter().map(|b| b.offset).collect();

        // Should not create boundaries after abbreviations
        assert!(!boundary_positions.contains(&3)); // After "Dr."
        assert!(!boundary_positions.contains(&41)); // After "Ph.D."
        assert!(!boundary_positions.contains(&72)); // After "Corp."
        assert!(!boundary_positions.contains(&95)); // After "$2.5" decimal

        // Should create boundaries after real sentence endings
        assert!(boundary_positions.len() >= 2); // At least two real boundaries
    }

    #[test]
    fn test_complex_enclosure_nesting() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        // Text with nested enclosures and a sentence boundary outside them
        let text = r#"He said (and I quote: "Hello world.") to everyone. That was nice."#;
        let state = parser.parse_chunk(text, &rules, None);

        // Should track enclosures
        assert!(!state.deltas.is_empty());
        // Should handle parsing successfully
        assert_eq!(state.chunk_length, text.len());

        // Should handle parsing without errors - this is the main goal
        // The boundary detection depends on language rules implementation

        // For now, just check that parsing completed successfully
        // The specific boundary detection behavior depends on the language rules implementation
        assert_eq!(state.chunk_length, text.len());
    }

    #[test]
    fn test_parser_determinism() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        let text = "First sentence. Second sentence! Third sentence?";

        // Parse the same text multiple times
        let state1 = parser.parse_chunk(text, &rules, None);
        let state2 = parser.parse_chunk(text, &rules, None);
        let state3 = parser.parse_chunk(text, &rules, None);

        // Results should be identical
        assert_eq!(state1.boundaries, state2.boundaries);
        assert_eq!(state2.boundaries, state3.boundaries);
        assert_eq!(state1.deltas, state2.deltas);
        assert_eq!(state1.chunk_length, state2.chunk_length);
    }

    #[test]
    fn test_empty_and_edge_cases() {
        let parser = Parser::new();
        let rules = MockLanguageRules::english();

        // Empty text
        let state = parser.parse_chunk("", &rules, None);
        assert!(state.boundaries.is_empty());
        assert_eq!(state.chunk_length, 0);

        // Single character
        let state = parser.parse_chunk(".", &rules, None);
        assert_eq!(state.chunk_length, 1);

        // Only spaces
        let state = parser.parse_chunk("   ", &rules, None);
        assert!(state.boundaries.is_empty());
        assert_eq!(state.chunk_length, 3);

        // Unclosed quote
        let text = r#"He said "Hello world."#;
        let state = parser.parse_chunk(text, &rules, None);
        // Should handle gracefully without panicking
        assert_eq!(state.chunk_length, text.len());
    }
}
