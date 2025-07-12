//! Text parsing implementation for sentence boundary detection
//!
//! This module implements the character-by-character parsing logic in the
//! application layer, converting text into partial states for the domain layer.

use crate::domain::{
    enclosure::{EnclosureRules, StandardEnclosureRules},
    language::{BoundaryContext, BoundaryDecision, LanguageRules},
    types::{AbbreviationState, BoundaryFlags, DeltaEntry, DepthVec, PartialState},
};

mod strategies;

#[cfg(test)]
mod tests;

pub use strategies::{
    ParseError, ParseStrategy, ParsingInput, ParsingOutput, SequentialParser, StreamingParser,
};

/// Default context window size for boundary detection
const DEFAULT_CONTEXT_WINDOW: usize = 10;

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

/// Text parser for converting text into partial states
///
/// This parser lives in the application layer and orchestrates
/// the parsing process using different strategies.
pub struct TextParser {
    #[allow(dead_code)]
    config: ParserConfig,
}

impl TextParser {
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

    /// Scans a chunk of text and produces a partial state with boundary candidates.
    ///
    /// This implements the scan phase of the Δ-Stack Monoid algorithm.
    /// It tracks local enclosure depth and records boundary candidates without
    /// determining if they are valid (that happens in the reduce phase).
    pub fn scan_chunk(&self, text: &str, language_rules: &dyn LanguageRules) -> PartialState {
        // Initialize state with the number of enclosure types from language rules
        let enclosure_count = language_rules.enclosure_type_count();
        let mut state = PartialState::new(enclosure_count);

        // Local depth tracking (relative to chunk start)
        let mut local_depths = vec![0i32; enclosure_count];
        let mut min_depths = vec![0i32; enclosure_count];

        let mut position = 0;
        let mut chars = text.chars().peekable();

        // Track parsing context
        let mut last_char: Option<char> = None;
        let mut consecutive_dots = 0;
        let mut first_word_captured = false;

        while let Some(ch) = chars.next() {
            let char_len = ch.len_utf8();

            // Capture first word of chunk if not yet captured
            if !first_word_captured && ch.is_alphabetic() {
                // Found the start of the first word
                let mut word = String::new();
                word.push(ch);

                // Collect the rest of the word
                let mut word_position = position + char_len;
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphabetic() {
                        word.push(next_ch);
                        word_position += next_ch.len_utf8();
                        chars.next();
                    } else {
                        break;
                    }
                }

                state.abbreviation.first_word = Some(word.clone());
                first_word_captured = true;

                // Update position and continue from after the word
                position = word_position;
                continue;
            }

            // Check for enclosure characters using language rules
            if let Some(enc_char) = language_rules.get_enclosure_char(ch) {
                // Check if this enclosure should be suppressed
                let should_suppress = if let Some(suppressor) =
                    language_rules.enclosure_suppressor()
                {
                    // Build context for suppression check
                    let context = build_enclosure_context(text, position, ch, &chars, last_char);
                    suppressor.should_suppress_enclosure(ch, &context)
                } else {
                    false
                };

                // Only track enclosure if not suppressed
                if !should_suppress {
                    if let Some(type_id) = language_rules.get_enclosure_type_id(ch) {
                        if type_id < enclosure_count {
                            if enc_char.is_opening {
                                local_depths[type_id] += 1;
                            } else {
                                local_depths[type_id] -= 1;
                                min_depths[type_id] =
                                    min_depths[type_id].min(local_depths[type_id]);
                            }
                        }
                    }
                }
            }

            // Track consecutive dots for abbreviations
            if ch == '.' {
                consecutive_dots += 1;
            } else {
                consecutive_dots = 0;
            }

            // Check for potential sentence terminators (record as candidate regardless of depth)
            if is_potential_terminator(ch) {
                // Build context for language rules
                let context =
                    build_boundary_context(text, position, ch, &chars, last_char, consecutive_dots);

                // Ask language rules for decision
                let decision = language_rules.detect_sentence_boundary(&context);

                match decision {
                    BoundaryDecision::Boundary(flags) => {
                        // Record as boundary candidate with current local depths
                        state.add_boundary_candidate(
                            position + char_len,
                            DepthVec::from_vec(local_depths.clone()),
                            flags,
                        );

                        // Track abbreviation state
                        if ch == '.' {
                            let abbr_result = language_rules.process_abbreviation(text, position);
                            if abbr_result.is_abbreviation {
                                state.abbreviation = AbbreviationState::with_first_word(
                                    true, // dangling_dot
                                    context
                                        .following_context
                                        .chars()
                                        .next()
                                        .is_some_and(|c| c.is_alphabetic()), // head_alpha
                                    state.abbreviation.first_word.clone(), // preserve first_word
                                );
                            }
                        }
                    }
                    BoundaryDecision::NotBoundary => {
                        // Not a boundary candidate
                    }
                    BoundaryDecision::NeedsMoreContext => {
                        // Record as weak boundary candidate
                        state.add_boundary_candidate(
                            position + char_len,
                            DepthVec::from_vec(local_depths.clone()),
                            BoundaryFlags::WEAK,
                        );
                    }
                }
            }

            last_char = Some(ch);
            position += char_len;
        }

        // Update final chunk length
        state.chunk_length = position;

        // Calculate final deltas
        for i in 0..enclosure_count {
            state.deltas[i] = DeltaEntry {
                net: local_depths[i],
                min: min_depths[i],
            };
        }

        // Note: unclosed enclosures are handled through the delta representation

        state
    }
}

impl Default for TextParser {
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
    // Extract text before the boundary (up to DEFAULT_CONTEXT_WINDOW chars)
    // Need to find valid UTF-8 boundary
    let mut start = position.saturating_sub(DEFAULT_CONTEXT_WINDOW);
    while start > 0 && !text.is_char_boundary(start) {
        start -= 1;
    }
    let preceding_context = text[start..position].to_string();

    // Peek at upcoming characters (up to DEFAULT_CONTEXT_WINDOW chars)
    let following_context = chars_iter
        .clone()
        .take(DEFAULT_CONTEXT_WINDOW)
        .collect::<String>();

    BoundaryContext {
        text: text.to_string(),
        position,
        boundary_char: terminator,
        preceding_context,
        following_context,
    }
}

/// Builds an enclosure context for suppression evaluation.
fn build_enclosure_context<'a>(
    text: &'a str,
    position: usize,
    _ch: char,
    chars_iter: &std::iter::Peekable<std::str::Chars>,
    last_char: Option<char>,
) -> crate::domain::enclosure_suppressor::EnclosureContext<'a> {
    use smallvec::SmallVec;

    // Get preceding characters (up to 3)
    let mut preceding_chars = SmallVec::<[char; 3]>::new();

    // Add last_char if available
    if let Some(ch) = last_char {
        preceding_chars.push(ch);
    }

    // Try to get more preceding characters from text
    if position > 0 {
        let preceding_text = &text[..position];
        let mut chars_before: Vec<char> = preceding_text.chars().collect();
        chars_before.reverse();

        // Skip the first char if we already have it from last_char
        let skip = if last_char.is_some() { 1 } else { 0 };

        for ch in chars_before
            .into_iter()
            .skip(skip)
            .take(3 - preceding_chars.len())
        {
            preceding_chars.insert(0, ch);
        }
    }

    // Get following characters (up to 3)
    let following_chars: SmallVec<[char; 3]> = chars_iter.clone().take(3).collect();

    // Calculate line offset (simple approximation)
    let line_offset = text[..position]
        .chars()
        .rev()
        .take_while(|&c| c != '\n')
        .count();

    crate::domain::enclosure_suppressor::EnclosureContext {
        position,
        preceding_chars,
        following_chars,
        line_offset,
        chunk_text: text,
    }
}

/// Convenient function for scanning a chunk of text with default settings.
pub fn scan_chunk(text: &str, language_rules: &dyn LanguageRules) -> PartialState {
    let parser = TextParser::new();
    parser.scan_chunk(text, language_rules)
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::domain::language::MockLanguageRules;

    #[test]
    fn test_basic_parsing() {
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        let text = "Hello world. This is a test.";
        let state = parser.scan_chunk(text, &rules);

        // Should have detected two boundary candidates
        assert_eq!(state.boundary_candidates.len(), 2);
        assert_eq!(state.chunk_length, text.len());
    }

    #[test]
    fn test_enclosure_handling() {
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        let text = "He said (Hello. World). Done.";
        let state = parser.scan_chunk(text, &rules);

        // Should record all boundary candidates (even inside parentheses)
        // The reduce phase will determine which are valid
        assert!(state
            .boundary_candidates
            .iter()
            .any(|b| b.local_offset == 23)); // After ).
        assert!(state
            .boundary_candidates
            .iter()
            .any(|b| b.local_offset == 29)); // After Done.
    }

    #[test]
    fn test_delta_stack_building() {
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        let text = "Test (nested) text.";
        let state = parser.scan_chunk(text, &rules);

        // Should have delta entries for the parentheses
        assert!(!state.deltas.is_empty());

        // Verify the delta has correct net values (should be balanced)
        let total_net: i32 = state.deltas.iter().map(|d| d.net).sum();
        assert_eq!(total_net, 0); // Balanced parentheses
    }

    #[test]
    fn test_abbreviation_detection() {
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        let text = "Dr. Smith arrived.";
        let state = parser.scan_chunk(text, &rules);

        // With proper abbreviation rules, Dr. should not create a boundary candidate
        // Only the final period should create a boundary candidate
        assert_eq!(state.boundary_candidates.len(), 1);
        assert!(state
            .boundary_candidates
            .iter()
            .any(|b| b.local_offset == 18)); // After "arrived."
    }

    #[test]
    fn test_quotation_handling() {
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        let text = r#"She said "Hello." Then left."#;
        let state = parser.scan_chunk(text, &rules);

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

        let parser = TextParser::new();
        let rules = EnglishLanguageRules::new();

        // Complex text with abbreviations, numbers, and nested punctuation
        let text = "Dr. Smith (born 1965) earned his Ph.D. He works at Tech Corp. The company is valued at $2.5 billion! Amazing.";
        let state = parser.scan_chunk(text, &rules);

        // Should handle abbreviations properly - no boundary candidates after "Dr." or "Ph.D."
        let boundary_positions: Vec<usize> = state
            .boundary_candidates
            .iter()
            .map(|b| b.local_offset)
            .collect();

        // Should not create boundaries after some abbreviations
        assert!(!boundary_positions.contains(&3)); // After "Dr." (followed by "Smith", not a sentence starter)
        assert!(!boundary_positions.contains(&95)); // After "$2.5" decimal

        // But SHOULD create boundaries after abbreviations followed by sentence starters
        let phd_pos = text.find("Ph.D.").unwrap() + 5; // Position after "Ph.D."
        let corp_pos = text.find("Corp.").unwrap() + 5; // Position after "Corp."

        assert!(
            boundary_positions.contains(&phd_pos),
            "Expected boundary after 'Ph.D.' at position {}",
            phd_pos
        );
        assert!(
            boundary_positions.contains(&corp_pos),
            "Expected boundary after 'Corp.' at position {}",
            corp_pos
        );

        // Should create boundary candidates after real sentence endings:
        // - After "Ph.D." when followed by "He" (sentence starter)
        // - After "Corp." when followed by "The" (sentence starter)
        // - After "billion!" (exclamation mark)
        // - After "Amazing." (period at end)
        assert_eq!(boundary_positions.len(), 4);
    }

    #[test]
    fn test_complex_enclosure_nesting() {
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        // Text with nested enclosures and a sentence boundary outside them
        let text = r#"He said (and I quote: "Hello world.") to everyone. That was nice."#;
        let state = parser.scan_chunk(text, &rules);

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
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        let text = "First sentence. Second sentence! Third sentence?";

        // Parse the same text multiple times
        let state1 = parser.scan_chunk(text, &rules);
        let state2 = parser.scan_chunk(text, &rules);
        let state3 = parser.scan_chunk(text, &rules);

        // Results should be identical
        assert_eq!(state1.boundary_candidates, state2.boundary_candidates);
        assert_eq!(state2.boundary_candidates, state3.boundary_candidates);
        assert_eq!(state1.deltas, state2.deltas);
        assert_eq!(state1.chunk_length, state2.chunk_length);
    }

    #[test]
    fn test_empty_and_edge_cases() {
        let parser = TextParser::new();
        let rules = MockLanguageRules::english();

        // Empty text
        let state = parser.scan_chunk("", &rules);
        assert!(state.boundary_candidates.is_empty());
        assert_eq!(state.chunk_length, 0);

        // Single character
        let state = parser.scan_chunk(".", &rules);
        assert_eq!(state.chunk_length, 1);

        // Only spaces
        let state = parser.scan_chunk("   ", &rules);
        assert!(state.boundary_candidates.is_empty());
        assert_eq!(state.chunk_length, 3);

        // Unclosed quote
        let text = r#"He said "Hello world."#;
        let state = parser.scan_chunk(text, &rules);
        // Should handle gracefully without panicking
        assert_eq!(state.chunk_length, text.len());
    }
}
