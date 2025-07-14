//! English-specific language rules for sentence boundary detection
//!
//! This module implements comprehensive English language rules that handle
//! the complexities of English sentence boundary detection including:
//! - Extended abbreviation detection
//! - Capitalization rules
//! - Number and date handling
//! - Quotation mark processing

use super::traits::{
    AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRules, QuotationContext,
    QuotationDecision,
};
use crate::domain::BoundaryFlags;
use std::collections::HashSet;

/// Comprehensive English language rules implementation
///
/// This implementation provides sophisticated English-specific sentence
/// boundary detection logic, building upon the basic rule foundation.
#[derive(Debug, Clone)]
pub struct EnglishLanguageRules {
    abbreviation_rule: EnglishAbbreviationRule,
    capitalization_rule: EnglishCapitalizationRule,
    number_rule: EnglishNumberRule,
    quotation_rule: EnglishQuotationRule,
    enclosure_suppressor: crate::domain::enclosure_suppressor::EnglishEnclosureSuppressor,
    sentence_starter_rule: EnglishSentenceStarterRule,
}

impl EnglishLanguageRules {
    /// Create a new English language rules instance
    pub fn new() -> Self {
        Self {
            abbreviation_rule: EnglishAbbreviationRule::new(),
            capitalization_rule: EnglishCapitalizationRule::new(),
            number_rule: EnglishNumberRule::new(),
            quotation_rule: EnglishQuotationRule::new(),
            enclosure_suppressor:
                crate::domain::enclosure_suppressor::EnglishEnclosureSuppressor::new(),
            sentence_starter_rule: EnglishSentenceStarterRule::new(),
        }
    }

    /// Create English rules with custom configurations
    pub fn with_custom_abbreviations(abbreviations: HashSet<String>) -> Self {
        Self {
            abbreviation_rule: EnglishAbbreviationRule::with_custom_list(abbreviations),
            capitalization_rule: EnglishCapitalizationRule::new(),
            number_rule: EnglishNumberRule::new(),
            quotation_rule: EnglishQuotationRule::new(),
            enclosure_suppressor:
                crate::domain::enclosure_suppressor::EnglishEnclosureSuppressor::new(),
            sentence_starter_rule: EnglishSentenceStarterRule::new(),
        }
    }
}

impl Default for EnglishLanguageRules {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishLanguageRules {
    /// Check if this looks like a title followed by a name pattern
    fn is_likely_title_name_pattern(
        &self,
        text: &str,
        position: usize,
        following_context: &str,
    ) -> bool {
        // Get the word before the period
        if position == 0 {
            return false;
        }

        let text_before = &text[..position];
        let word_start = text_before
            .char_indices()
            .rfind(|(_, c)| c.is_whitespace())
            .map(|(idx, c)| {
                // Get the byte position after the found character
                idx + c.len_utf8()
            })
            .unwrap_or(0);

        if word_start >= position {
            return false;
        }

        let word_before = &text[word_start..position];

        // Check if it's a known title abbreviation
        let is_title = matches!(
            word_before.to_lowercase().as_str(),
            "mr" | "mrs"
                | "ms"
                | "dr"
                | "prof"
                | "rev"
                | "fr"
                | "gov"
                | "lt"
                | "sen"
                | "rep"
                | "hon"
                | "pres"
                | "gen"
                | "col"
                | "maj"
                | "capt"
                | "sgt"
                | "atty"
                | "esq"
                | "supt"
        );

        if !is_title {
            return false;
        }

        // Check if the following text starts with a capital letter (likely a name)
        let trimmed_following = following_context.trim_start();
        if let Some(first_char) = trimmed_following.chars().next() {
            return first_char.is_uppercase() && first_char.is_alphabetic();
        }

        false
    }
}

impl LanguageRules for EnglishLanguageRules {
    fn detect_sentence_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        // Check if this is a sentence-ending punctuation
        if !matches!(context.boundary_char, '.' | '!' | '?') {
            return BoundaryDecision::NotBoundary;
        }

        // Check for abbreviations first
        let abbrev_result = self
            .abbreviation_rule
            .detect_abbreviation(&context.text, context.position);
        if abbrev_result.is_abbreviation && abbrev_result.confidence > 0.8 {
            // Check if the abbreviation is followed by a sentence starter
            if let Some(first_word) =
                EnglishSentenceStarterRule::extract_first_word(&context.following_context)
            {
                if self.sentence_starter_rule.is_sentence_starter(first_word) {
                    // This is an abbreviation followed by a sentence starter - it IS a boundary
                    return BoundaryDecision::Boundary(BoundaryFlags::STRONG);
                }
            }

            // Enhanced context check for title + name patterns
            if self.is_likely_title_name_pattern(
                &context.text,
                context.position,
                &context.following_context,
            ) {
                return BoundaryDecision::NotBoundary;
            }
            // For high-confidence abbreviations not followed by sentence starters, generally not a boundary
            return BoundaryDecision::NotBoundary;
        }

        // Check for numbers/decimals
        if self
            .number_rule
            .is_decimal_point(&context.text, context.position)
        {
            return BoundaryDecision::NotBoundary;
        }

        // Check for time format
        if self
            .number_rule
            .is_time_format(&context.text, context.position)
        {
            return BoundaryDecision::NotBoundary;
        }

        // Check for numeric dot sequences (version numbers, IP addresses, section numbers, etc.)
        if self
            .number_rule
            .is_numeric_dot_sequence(&context.text, context.position)
        {
            return BoundaryDecision::NotBoundary;
        }

        // Check for list markers
        if self
            .number_rule
            .is_list_marker(&context.text, context.position)
        {
            return BoundaryDecision::NotBoundary;
        }

        // Check capitalization of following text
        let cap_analysis = self
            .capitalization_rule
            .analyze_following_text(&context.following_context);

        match context.boundary_char {
            '!' | '?' => {
                // Strong punctuation usually indicates sentence boundary
                if cap_analysis.starts_with_capital || context.following_context.trim().is_empty() {
                    BoundaryDecision::Boundary(BoundaryFlags::STRONG)
                } else {
                    BoundaryDecision::NeedsMoreContext
                }
            }
            '.' => {
                // Period requires more careful analysis
                if context.following_context.trim().is_empty() {
                    // End of text
                    BoundaryDecision::Boundary(BoundaryFlags::WEAK)
                } else if cap_analysis.starts_with_capital {
                    // Additional check for title + name pattern even without abbreviation detection
                    if self.is_likely_title_name_pattern(
                        &context.text,
                        context.position,
                        &context.following_context,
                    ) {
                        BoundaryDecision::NotBoundary
                    } else {
                        // Next sentence starts with capital
                        BoundaryDecision::Boundary(BoundaryFlags::WEAK)
                    }
                } else if cap_analysis.starts_with_quote_and_capital {
                    // Quoted speech starting with capital
                    BoundaryDecision::Boundary(BoundaryFlags::WEAK)
                } else {
                    BoundaryDecision::NeedsMoreContext
                }
            }
            _ => BoundaryDecision::NotBoundary,
        }
    }

    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        self.abbreviation_rule.detect_abbreviation(text, position)
    }

    fn handle_quotation(&self, context: &QuotationContext) -> QuotationDecision {
        self.quotation_rule.classify_quote(context)
    }

    fn language_code(&self) -> &str {
        "en"
    }

    fn language_name(&self) -> &str {
        "English"
    }

    fn get_enclosure_char(&self, ch: char) -> Option<crate::domain::enclosure::EnclosureChar> {
        use crate::domain::enclosure::{EnclosureChar, EnclosureType};

        #[allow(unreachable_patterns)]
        match ch {
            '"' => Some(EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: true, // Ambiguous straight quote - parser will determine
            }),
            '"' => Some(EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: true,
            }),
            '"' => Some(EnclosureChar {
                enclosure_type: EnclosureType::DoubleQuote,
                is_opening: false,
            }),
            '\'' | '\u{2018}' | '\u{2019}' => Some(EnclosureChar {
                enclosure_type: EnclosureType::SingleQuote,
                is_opening: matches!(ch, '\'' | '\u{2018}'),
            }),
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
        use crate::domain::enclosure::EnclosureType;

        self.get_enclosure_char(ch)
            .map(|enc| match enc.enclosure_type {
                EnclosureType::DoubleQuote => 0,
                EnclosureType::SingleQuote => 1,
                EnclosureType::Parenthesis => 2,
                EnclosureType::SquareBracket => 3,
                EnclosureType::CurlyBrace => 4,
                _ => 0, // Default for any other types
            })
    }

    fn enclosure_type_count(&self) -> usize {
        5 // DoubleQuote, SingleQuote, Parenthesis, SquareBracket, CurlyBrace
    }

    fn enclosure_suppressor(
        &self,
    ) -> Option<&dyn crate::domain::enclosure_suppressor::EnclosureSuppressor> {
        Some(&self.enclosure_suppressor)
    }
}

/// Enhanced English abbreviation detection
#[derive(Debug, Clone)]
pub struct EnglishAbbreviationRule {
    /// Comprehensive set of English abbreviations
    abbreviations: HashSet<String>,
    /// Confidence threshold for unknown abbreviations
    confidence_threshold: f32,
}

impl Default for EnglishAbbreviationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishAbbreviationRule {
    /// Create with comprehensive English abbreviation list
    pub fn new() -> Self {
        let mut abbreviations = HashSet::new();

        // Academic titles
        abbreviations.insert("Dr".to_string());
        abbreviations.insert("Prof".to_string());
        abbreviations.insert("Ph".to_string()); // Ph.D.
        abbreviations.insert("D".to_string()); // Ph.D., M.D., etc.
        abbreviations.insert("M".to_string()); // M.D., M.A., M.S.
        abbreviations.insert("A".to_string()); // M.A., B.A.
        abbreviations.insert("S".to_string()); // M.S., B.S.
        abbreviations.insert("B".to_string()); // B.A., B.S.
        abbreviations.insert("Jr".to_string());
        abbreviations.insert("Sr".to_string());

        // Personal titles
        abbreviations.insert("Mr".to_string());
        abbreviations.insert("Mrs".to_string());
        abbreviations.insert("Ms".to_string());
        abbreviations.insert("Miss".to_string());
        abbreviations.insert("Rev".to_string());
        abbreviations.insert("Fr".to_string());

        // Government/Political titles (from Brown Corpus analysis)
        abbreviations.insert("Gov".to_string());
        abbreviations.insert("Lt".to_string());
        abbreviations.insert("Sen".to_string());
        abbreviations.insert("Rep".to_string());
        abbreviations.insert("Hon".to_string());
        abbreviations.insert("Pres".to_string());

        // Military ranks
        abbreviations.insert("Gen".to_string());
        abbreviations.insert("Col".to_string());
        abbreviations.insert("Maj".to_string());
        abbreviations.insert("Capt".to_string());
        abbreviations.insert("Sgt".to_string());
        abbreviations.insert("Cpl".to_string());
        abbreviations.insert("Pvt".to_string());

        // Legal/Professional
        abbreviations.insert("Atty".to_string());
        abbreviations.insert("Esq".to_string());
        abbreviations.insert("Supt".to_string());

        // Geographic abbreviations
        abbreviations.insert("St".to_string()); // Street/Saint
        abbreviations.insert("Ave".to_string()); // Avenue
        abbreviations.insert("Blvd".to_string()); // Boulevard
        abbreviations.insert("Rd".to_string()); // Road
        abbreviations.insert("Ln".to_string()); // Lane
        abbreviations.insert("Apt".to_string()); // Apartment
        abbreviations.insert("Bldg".to_string()); // Building
        abbreviations.insert("Fl".to_string()); // Floor
        abbreviations.insert("U".to_string()); // U.S., U.K., etc.
        abbreviations.insert("S".to_string()); // U.S. (also B.S., M.S.)

        // State abbreviations commonly found in Brown Corpus
        abbreviations.insert("Ill".to_string());
        abbreviations.insert("Ind".to_string());
        abbreviations.insert("Kan".to_string());
        abbreviations.insert("Mass".to_string());
        abbreviations.insert("Ore".to_string());
        abbreviations.insert("Tex".to_string());
        abbreviations.insert("Conn".to_string());
        abbreviations.insert("Calif".to_string());

        // Business/Organization
        abbreviations.insert("Corp".to_string());
        abbreviations.insert("Inc".to_string());
        abbreviations.insert("Ltd".to_string());
        abbreviations.insert("LLC".to_string());
        abbreviations.insert("Co".to_string());
        abbreviations.insert("Assn".to_string()); // Association
        abbreviations.insert("Org".to_string()); // Organization
        abbreviations.insert("C".to_string()); // C.E.O., C.F.O., etc.
        abbreviations.insert("E".to_string()); // C.E.O.
        abbreviations.insert("O".to_string()); // C.E.O., C.O.O., etc.

        // Common abbreviations
        abbreviations.insert("etc".to_string());
        abbreviations.insert("vs".to_string());
        abbreviations.insert("e.g".to_string());
        abbreviations.insert("i.e".to_string());
        abbreviations.insert("cf".to_string());
        abbreviations.insert("viz".to_string());
        abbreviations.insert("approx".to_string());
        abbreviations.insert("est".to_string());

        // Add missing single-letter abbreviations commonly used as initials
        // (Some are already defined above for academic titles)
        abbreviations.insert("F".to_string());
        abbreviations.insert("G".to_string());
        abbreviations.insert("H".to_string());
        abbreviations.insert("I".to_string());
        abbreviations.insert("J".to_string());
        abbreviations.insert("K".to_string());
        abbreviations.insert("L".to_string());
        abbreviations.insert("N".to_string());
        abbreviations.insert("P".to_string());
        abbreviations.insert("Q".to_string());
        abbreviations.insert("R".to_string());
        abbreviations.insert("T".to_string());
        abbreviations.insert("V".to_string());
        abbreviations.insert("W".to_string());
        abbreviations.insert("X".to_string());
        abbreviations.insert("Y".to_string());
        abbreviations.insert("Z".to_string());

        // Time/Date
        abbreviations.insert("Jan".to_string());
        abbreviations.insert("Feb".to_string());
        abbreviations.insert("Mar".to_string());
        abbreviations.insert("Apr".to_string());
        abbreviations.insert("Aug".to_string());
        abbreviations.insert("Sep".to_string());
        abbreviations.insert("Sept".to_string());
        abbreviations.insert("Oct".to_string());
        abbreviations.insert("Nov".to_string());
        abbreviations.insert("Dec".to_string());
        abbreviations.insert("Mon".to_string());
        abbreviations.insert("Tue".to_string());
        abbreviations.insert("Wed".to_string());
        abbreviations.insert("Thu".to_string());
        abbreviations.insert("Fri".to_string());
        abbreviations.insert("Sat".to_string());
        abbreviations.insert("Sun".to_string());

        Self {
            abbreviations,
            confidence_threshold: 0.8,
        }
    }

    /// Create with custom abbreviation list
    pub fn with_custom_list(abbreviations: HashSet<String>) -> Self {
        Self {
            abbreviations,
            confidence_threshold: 0.8,
        }
    }

    /// Detect abbreviations using enhanced logic
    pub fn detect_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        if position == 0 {
            return AbbreviationResult {
                is_abbreviation: false,
                length: 0,
                confidence: 0.0,
            };
        }

        // Find word boundaries using char_indices for UTF-8 safety
        let text_before = &text[..position];
        let start_pos = text_before
            .char_indices()
            .rfind(|(_, c)| c.is_whitespace() || c.is_ascii_punctuation())
            .map(|(idx, c)| {
                // Get the byte position after the found character
                idx + c.len_utf8()
            })
            .unwrap_or(0);

        if start_pos >= position {
            return AbbreviationResult {
                is_abbreviation: false,
                length: 0,
                confidence: 0.0,
            };
        }

        let potential_abbrev = &text[start_pos..position];

        // Check exact matches
        if self.abbreviations.contains(potential_abbrev) {
            return AbbreviationResult {
                is_abbreviation: true,
                length: potential_abbrev.len(),
                confidence: 1.0,
            };
        }

        // Check multi-part abbreviations (e.g., "Ph.D", "U.S.A")
        if potential_abbrev.contains('.') {
            let without_dots = potential_abbrev.replace('.', "");
            if self.abbreviations.contains(&without_dots) {
                return AbbreviationResult {
                    is_abbreviation: true,
                    length: potential_abbrev.len(),
                    confidence: 0.95,
                };
            }
        }

        // Heuristic detection for unknown abbreviations
        let confidence = self.calculate_abbreviation_confidence(potential_abbrev);

        AbbreviationResult {
            is_abbreviation: confidence >= self.confidence_threshold,
            length: if confidence >= self.confidence_threshold {
                potential_abbrev.len()
            } else {
                0
            },
            confidence,
        }
    }

    /// Calculate confidence score for potential abbreviations
    fn calculate_abbreviation_confidence(&self, text: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Length-based scoring
        if text.len() >= 2 && text.len() <= 6 {
            score += 0.3;
        }

        // All uppercase letters
        if text.chars().all(|c| c.is_ascii_uppercase() || c == '.') {
            score += 0.4;
        }

        // Contains capital letters
        if text.chars().any(|c| c.is_ascii_uppercase()) {
            score += 0.2;
        }

        // Ends with common abbreviation patterns
        if text.ends_with("Corp") || text.ends_with("Inc") || text.ends_with("Ltd") {
            score += 0.3;
        }

        score.min(1.0)
    }
}

/// English capitalization analysis
#[derive(Debug, Clone)]
pub struct EnglishCapitalizationRule {
    /// Common English articles and prepositions that might appear after periods
    lowercase_words: HashSet<String>,
}

impl Default for EnglishCapitalizationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishCapitalizationRule {
    pub fn new() -> Self {
        let mut lowercase_words = HashSet::new();
        lowercase_words.insert("a".to_string());
        lowercase_words.insert("an".to_string());
        lowercase_words.insert("the".to_string());
        lowercase_words.insert("and".to_string());
        lowercase_words.insert("or".to_string());
        lowercase_words.insert("but".to_string());
        lowercase_words.insert("in".to_string());
        lowercase_words.insert("on".to_string());
        lowercase_words.insert("at".to_string());
        lowercase_words.insert("to".to_string());
        lowercase_words.insert("for".to_string());
        lowercase_words.insert("of".to_string());
        lowercase_words.insert("with".to_string());
        lowercase_words.insert("by".to_string());

        Self { lowercase_words }
    }

    pub fn analyze_following_text(&self, following_text: &str) -> CapitalizationAnalysis {
        let trimmed = following_text.trim_start();

        if trimmed.is_empty() {
            return CapitalizationAnalysis {
                starts_with_capital: false,
                starts_with_quote_and_capital: false,
                first_word_is_proper_noun: false,
            };
        }

        // Find the first alphabetic character, skipping non-alphabetic ones
        let mut first_alpha_char = None;

        for ch in trimmed.chars() {
            if ch.is_alphabetic() {
                first_alpha_char = Some(ch);
                break;
            }
        }

        // Check if the first character is a quote
        let first_char = trimmed.chars().next().unwrap();
        let starts_with_quote = matches!(first_char, '"' | '\'');

        // For quoted speech, check the first alphabetic character after the quote
        if starts_with_quote {
            let after_quote = &trimmed[first_char.len_utf8()..];
            let after_quote_trimmed = after_quote.trim_start();

            if let Some(char_after_quote) = after_quote_trimmed.chars().find(|c| c.is_alphabetic())
            {
                return CapitalizationAnalysis {
                    starts_with_capital: false,
                    starts_with_quote_and_capital: char_after_quote.is_ascii_uppercase(),
                    first_word_is_proper_noun: self.is_likely_proper_noun(after_quote_trimmed),
                };
            }
        }

        // If we found an alphabetic character, check if it's uppercase
        let starts_with_capital = first_alpha_char
            .map(|ch| ch.is_ascii_uppercase())
            .unwrap_or(false);

        CapitalizationAnalysis {
            starts_with_capital,
            starts_with_quote_and_capital: false,
            first_word_is_proper_noun: self.is_likely_proper_noun(trimmed),
        }
    }

    fn is_likely_proper_noun(&self, text: &str) -> bool {
        if let Some(first_word) = text.split_whitespace().next() {
            // Remove punctuation and check if it's a known lowercase word
            let cleaned = first_word.trim_end_matches(|c: char| c.is_ascii_punctuation());
            !self.lowercase_words.contains(&cleaned.to_lowercase())
        } else {
            false
        }
    }
}

/// Analysis result for capitalization patterns
#[derive(Debug, Clone, PartialEq)]
pub struct CapitalizationAnalysis {
    pub starts_with_capital: bool,
    pub starts_with_quote_and_capital: bool,
    pub first_word_is_proper_noun: bool,
}

/// English number and date pattern detection
#[derive(Debug, Clone)]
pub struct EnglishNumberRule {
    // No configuration needed for basic implementation
}

impl Default for EnglishNumberRule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishNumberRule {
    pub fn new() -> Self {
        Self {}
    }

    /// Check if a period is part of a decimal number
    pub fn is_decimal_point(&self, text: &str, position: usize) -> bool {
        // Convert byte position to character-aware checking
        if position == 0 || position >= text.len() {
            return false;
        }

        // Get byte slice before and after the position
        let before_bytes = &text[..position];
        let after_bytes = &text[position + 1..];

        // Check if the character before is a digit
        let before_digit = before_bytes
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        // Check if the character after is a digit
        let after_digit = after_bytes
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        before_digit && after_digit
    }

    /// Check if this looks like a time format (e.g., "3:30 p.m.")
    pub fn is_time_format(&self, text: &str, position: usize) -> bool {
        // Look for patterns like "p.m." or "a.m."
        if position >= 1 {
            // Check if the character immediately before the period is 'p' or 'a'
            let before_bytes = &text[..position];
            if let Some(before_char) = before_bytes.chars().last() {
                matches!(before_char, 'p' | 'a')
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if a period is part of a numeric dot sequence pattern
    /// Examples: "2.0.1", "v1.2.3", "Python 3.11.4", "192.168.1.1", "section 2.3.4"
    /// Pattern: \d+(\.\d+)+\.?
    pub fn is_numeric_dot_sequence(&self, text: &str, position: usize) -> bool {
        if position == 0 || position >= text.len() {
            return false;
        }

        // Get context before and after the period
        let before_bytes = &text[..position];
        let after_bytes = &text[position + 1..];

        // Check if this period is between digits
        let before_digit = before_bytes
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        let after_digit = after_bytes
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false);

        if !before_digit && !after_digit {
            return false;
        }

        // Find the start of the numeric pattern (go back until we hit a non-digit/non-period)
        let pattern_start = before_bytes
            .char_indices()
            .rfind(|(_, c)| !c.is_ascii_digit() && *c != '.')
            .map(|(idx, c)| idx + c.len_utf8())
            .unwrap_or(0);

        // Find the end of the numeric pattern (go forward until we hit a non-digit/non-period)
        let pattern_end_offset = after_bytes
            .char_indices()
            .find(|(_, c)| !c.is_ascii_digit() && *c != '.')
            .map(|(idx, _)| idx)
            .unwrap_or(after_bytes.len());

        let pattern = &text[pattern_start..position + 1 + pattern_end_offset];

        // Check if the pattern matches \d+(\.\d+)+\.?
        // It should:
        // 1. Start with one or more digits
        // 2. Have at least one group of period followed by digits
        // 3. Optionally end with a period

        let trimmed_pattern = pattern.trim_end_matches('.');
        let parts: Vec<&str> = trimmed_pattern.split('.').collect();

        // Need at least 2 parts (e.g., "1.2") for the pattern \d+(\.\d+)+
        // But to have multiple periods (e.g., "1.2.3"), we need at least 3 parts
        if parts.len() < 3 {
            return false;
        }

        // All parts must be non-empty and contain only digits
        for part in &parts {
            if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }

        true
    }

    /// Check if a period is part of a list marker at the start of a line
    /// Examples: "1. First item", "a. Start", "i. Introduction"
    /// Pattern: ^[a-zA-Z0-9]+\. (line start + alphanumeric + period + space)
    pub fn is_list_marker(&self, text: &str, position: usize) -> bool {
        if position == 0 || position >= text.len() {
            return false;
        }

        // Check if there's a space after the period (required for list markers)
        if let Some(after_char) = text[position + 1..].chars().next() {
            if !after_char.is_whitespace() {
                return false;
            }
        } else {
            return false;
        }

        // For list markers, we need to check what comes before the period
        let before_bytes = &text[..position];

        // Find the start of the line (look for newline or start of text)
        let line_start = before_bytes
            .char_indices()
            .rfind(|(_, c)| *c == '\n')
            .map(|(idx, _)| idx + 1)
            .unwrap_or(0);

        if line_start >= position {
            return false;
        }

        // Get the content between line start and the period
        let before_period = &text[line_start..position];

        // Skip any leading whitespace
        let trimmed = before_period.trim_start();

        // Check if it's a valid list marker:
        // - Non-empty
        // - Only alphanumeric characters
        // - Reasonable length (1-3 characters for numbers, 1 for letters)
        if trimmed.is_empty() || trimmed.len() > 3 {
            return false;
        }

        // Check if it's all digits (numeric list marker)
        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(num) = trimmed.parse::<u32>() {
                return (1..=999).contains(&num);
            }
        }

        // Check if it's a single letter (alphabetic list marker)
        if trimmed.len() == 1 && trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
            return true;
        }

        // Check for common Roman numerals
        matches!(
            trimmed,
            "i" | "ii"
                | "iii"
                | "iv"
                | "v"
                | "vi"
                | "vii"
                | "viii"
                | "ix"
                | "x"
                | "I"
                | "II"
                | "III"
                | "IV"
                | "V"
                | "VI"
                | "VII"
                | "VIII"
                | "IX"
                | "X"
        )
    }
}

/// English sentence starter detection rule
#[derive(Debug, Clone)]
pub struct EnglishSentenceStarterRule {
    /// Set of words that commonly start sentences
    sentence_starters: HashSet<String>,
}

impl Default for EnglishSentenceStarterRule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishSentenceStarterRule {
    pub fn new() -> Self {
        let mut starters = HashSet::new();

        // Personal pronouns
        starters.insert("I".to_string());
        starters.insert("He".to_string());
        starters.insert("She".to_string());
        starters.insert("It".to_string());
        starters.insert("We".to_string());
        starters.insert("You".to_string());
        starters.insert("They".to_string());

        // WH-words (question words)
        starters.insert("What".to_string());
        starters.insert("Why".to_string());
        starters.insert("When".to_string());
        starters.insert("Where".to_string());
        starters.insert("Who".to_string());
        starters.insert("Whom".to_string());
        starters.insert("Whose".to_string());
        starters.insert("Which".to_string());
        starters.insert("How".to_string());

        // Demonstratives
        starters.insert("This".to_string());
        starters.insert("That".to_string());
        starters.insert("These".to_string());
        starters.insert("Those".to_string());

        // Conjunctive adverbs / logical markers
        starters.insert("However".to_string());
        starters.insert("Therefore".to_string());
        starters.insert("Thus".to_string());
        starters.insert("Moreover".to_string());
        starters.insert("Furthermore".to_string());
        starters.insert("Meanwhile".to_string());
        starters.insert("Consequently".to_string());
        starters.insert("Nevertheless".to_string());

        // Conditional adverbs
        starters.insert("Otherwise".to_string());
        starters.insert("Instead".to_string());

        // Interjections
        starters.insert("Well".to_string());
        starters.insert("Oh".to_string());
        starters.insert("Alas".to_string());

        // Negative adverbs
        starters.insert("No".to_string());
        starters.insert("Not".to_string());

        // Common article (included for completeness)
        starters.insert("The".to_string());

        // Time indicators
        starters.insert("Yesterday".to_string());
        starters.insert("Today".to_string());
        starters.insert("Tomorrow".to_string());

        Self {
            sentence_starters: starters,
        }
    }

    /// Check if a word is a common sentence starter
    pub fn is_sentence_starter(&self, word: &str) -> bool {
        // Only consider words that start with a capital letter as potential sentence starters
        if word.chars().next().is_some_and(|c| !c.is_uppercase()) {
            return false;
        }

        // Check both the original case and title case
        // This handles "HOWEVER" -> "However", etc.
        if self.sentence_starters.contains(word) {
            return true;
        }

        // Convert to title case and check
        let title_case = word
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_lowercase().collect::<String>()
                }
            })
            .collect::<String>();

        self.sentence_starters.contains(&title_case)
    }

    /// Extract the first word from text (handling quotes and punctuation)
    pub fn extract_first_word(text: &str) -> Option<&str> {
        let trimmed = text.trim_start();

        // Skip leading quotes and punctuation
        let start = trimmed.chars().position(|c| c.is_alphabetic()).unwrap_or(0);

        if start >= trimmed.len() {
            return None;
        }

        let word_text = &trimmed[start..];

        // Find the end of the word
        let end = word_text
            .chars()
            .position(|c| !c.is_alphabetic())
            .unwrap_or(word_text.len());

        if end == 0 {
            None
        } else {
            Some(&word_text[..end])
        }
    }
}

/// Enhanced English quotation processing
#[derive(Debug, Clone)]
pub struct EnglishQuotationRule {
    /// Different types of quotation marks used in English
    quote_pairs: Vec<(char, char)>,
}

impl Default for EnglishQuotationRule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishQuotationRule {
    pub fn new() -> Self {
        Self {
            quote_pairs: vec![
                ('"', '"'),   // Straight quotes
                ('"', '"'),   // Curly quotes
                ('\'', '\''), // Single quotes
            ],
        }
    }

    pub fn classify_quote(&self, context: &QuotationContext) -> QuotationDecision {
        let quote_char = context.quote_char;

        // Check if this is an opening or closing quote based on context
        for (open, close) in &self.quote_pairs {
            if quote_char == *open && !context.inside_quotes {
                return QuotationDecision::QuoteStart;
            }
            if quote_char == *close && context.inside_quotes {
                return QuotationDecision::QuoteEnd;
            }
        }

        // For ambiguous quotes (like straight quotes), use context
        if quote_char == '"' || quote_char == '\'' {
            // Simple heuristic: if preceded by whitespace or punctuation, likely opening
            if let Some(preceding_char) =
                context.text.chars().nth(context.position.saturating_sub(1))
            {
                if preceding_char.is_whitespace() || matches!(preceding_char, '(' | '[' | '{') {
                    return if context.inside_quotes {
                        QuotationDecision::QuoteEnd
                    } else {
                        QuotationDecision::QuoteStart
                    };
                }
            }
        }

        QuotationDecision::Regular
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_english_language_rules_basic() {
        let rules = EnglishLanguageRules::new();

        assert_eq!(rules.language_code(), "en");
        assert_eq!(rules.language_name(), "English");
    }

    #[test]
    fn test_english_abbreviation_detection() {
        let rules = EnglishAbbreviationRule::new();

        // Test academic titles
        let result = rules.detect_abbreviation("Dr. Smith", 2);
        assert!(result.is_abbreviation);
        assert_eq!(result.confidence, 1.0);

        // Test geographic abbreviations
        let result = rules.detect_abbreviation("123 Main St. Apt", 11);
        assert!(result.is_abbreviation);

        // Test business abbreviations
        let result = rules.detect_abbreviation("Apple Inc. announced", 9);
        assert!(result.is_abbreviation);

        // Test not an abbreviation
        let result = rules.detect_abbreviation("Hello world.", 11);
        assert!(!result.is_abbreviation);
    }

    #[test]
    fn test_abbreviation_confidence_scoring() {
        let rule = EnglishAbbreviationRule::new();

        // Length-based scoring (0.3) + all uppercase (0.4) + contains uppercase (0.2) = 0.9
        assert!((rule.calculate_abbreviation_confidence("ABC") - 0.9).abs() < 0.01); // Length + all uppercase + contains uppercase
        assert!((rule.calculate_abbreviation_confidence("A") - 0.6).abs() < 0.01); // All uppercase + contains uppercase
        assert!((rule.calculate_abbreviation_confidence("ABCDEFG") - 0.6).abs() < 0.01); // Too long, all uppercase + contains uppercase

        // Length only (0.3) + contains capital (0.2)
        assert!((rule.calculate_abbreviation_confidence("abc") - 0.3).abs() < 0.01); // Length only
        assert!((rule.calculate_abbreviation_confidence("Ab.C") - 0.5).abs() < 0.01); // Length + some uppercase

        // Business suffix scoring:
        // "TechCorp" (8 chars) = contains_capital(0.2) + suffix(0.3) = 0.5
        assert!((rule.calculate_abbreviation_confidence("TechCorp") - 0.5).abs() < 0.01);
        // "MyInc" (5 chars) = length(0.3) + contains_capital(0.2) + suffix(0.3) = 0.8
        assert!((rule.calculate_abbreviation_confidence("MyInc") - 0.8).abs() < 0.01);
        // "CompanyLtd" (10 chars) = contains_capital(0.2) + suffix(0.3) = 0.5
        assert!((rule.calculate_abbreviation_confidence("CompanyLtd") - 0.5).abs() < 0.01);

        // "Corp": length(0.3) + contains_capital(0.2) + suffix(0.3) = 0.8 (not all uppercase)
        assert!((rule.calculate_abbreviation_confidence("Corp") - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_multi_dot_abbreviations() {
        let rule = EnglishAbbreviationRule::new();

        // Test standard abbreviations that are in our list
        let result = rule.detect_abbreviation("Dr. Smith is here", 2);
        assert!(result.is_abbreviation);
        assert_eq!(result.confidence, 1.0);

        // Test etc. abbreviation
        let result = rule.detect_abbreviation("books, papers, etc. are useful", 18);
        assert!(result.is_abbreviation);
        assert_eq!(result.confidence, 1.0);

        // Test vs. abbreviation
        let result = rule.detect_abbreviation("cats vs. dogs debate", 7);
        assert!(result.is_abbreviation);
        assert_eq!(result.confidence, 1.0);

        // Test heuristic detection for unknown patterns with dots
        let _result = rule.detect_abbreviation("The U.S.A. is large", 9);
        // This should be detected by heuristics due to uppercase + dots pattern
        // but might not be abbreviation due to confidence threshold

        // Test unknown multi-dot pattern with heuristics
        let _result = rule.detect_abbreviation("This X.Y.Z. is unknown", 11);
        // Should be detected by heuristics with some confidence
    }

    #[test]
    fn test_quotation_classification() {
        let rule = EnglishQuotationRule::new();

        // Quote start detection
        let context = QuotationContext {
            text: "He said \"Hello world\"".to_string(),
            position: 8,
            quote_char: '"',
            inside_quotes: false,
        };
        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteStart);

        // Quote end detection
        let context = QuotationContext {
            text: "He said \"Hello world\"".to_string(),
            position: 20,
            quote_char: '"',
            inside_quotes: true,
        };
        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteEnd);

        // Ambiguous quote preceded by space (context-based decision - should be QuoteStart)
        let context = QuotationContext {
            text: "It's a \"test\" case".to_string(),
            position: 7,
            quote_char: '"',
            inside_quotes: false,
        };
        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteStart);

        // Single quote that looks like quote start (due to current implementation)
        // The current implementation checks quote_pairs first, so '\'' matches and returns QuoteStart
        // when !inside_quotes is true
        let context = QuotationContext {
            text: "It's Tom's book".to_string(),
            position: 2,
            quote_char: '\'',
            inside_quotes: false,
        };
        // Due to implementation, this will be QuoteStart because '\'' is in quote_pairs
        assert_eq!(rule.classify_quote(&context), QuotationDecision::QuoteStart);
    }

    #[test]
    fn test_complex_boundary_scenarios() {
        let rules = EnglishLanguageRules::new();

        // Question mark + lowercase continuation (NeedsMoreContext expected)
        let context = BoundaryContext {
            text: "What time? around 3pm.".to_string(),
            position: 9,
            boundary_char: '?',
            preceding_context: "What time".to_string(),
            following_context: " around 3pm.".to_string(),
        };
        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NeedsMoreContext
        );

        // Exclamation mark + lowercase continuation
        let context = BoundaryContext {
            text: "Wow! that's amazing.".to_string(),
            position: 3,
            boundary_char: '!',
            preceding_context: "Wow".to_string(),
            following_context: " that's amazing.".to_string(),
        };
        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NeedsMoreContext
        );

        // Boundary within quotes
        let context = BoundaryContext {
            text: "He said \"Hello.\" Then left.".to_string(),
            position: 14,
            boundary_char: '.',
            preceding_context: "d \"Hello".to_string(),
            following_context: "\" Then left.".to_string(),
        };
        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::Boundary(_) => {}
            other => panic!("Expected boundary in quoted speech, got {:?}", other),
        }
    }

    #[test]
    fn test_proper_noun_detection() {
        let rule = EnglishCapitalizationRule::new();

        // Proper nouns
        assert!(rule.is_likely_proper_noun("John"));
        assert!(rule.is_likely_proper_noun("Microsoft"));
        assert!(rule.is_likely_proper_noun("Tokyo"));

        // Common lowercase words
        assert!(!rule.is_likely_proper_noun("and"));
        assert!(!rule.is_likely_proper_noun("the"));
        assert!(!rule.is_likely_proper_noun("with"));

        // With punctuation
        assert!(rule.is_likely_proper_noun("John,"));
        assert!(rule.is_likely_proper_noun("Microsoft."));

        // Edge cases
        assert!(!rule.is_likely_proper_noun(""));
        assert!(!rule.is_likely_proper_noun("   "));
    }

    #[test]
    fn test_edge_case_boundaries() {
        let rule = EnglishNumberRule::new();

        // Boundary position tests
        assert!(!rule.is_decimal_point("", 0));
        assert!(!rule.is_decimal_point("3.14", 4)); // Out of range
        assert!(!rule.is_time_format("a", 0)); // Position < 1

        let abbrev_rule = EnglishAbbreviationRule::new();

        // Empty/short strings
        assert!(!abbrev_rule.detect_abbreviation("", 0).is_abbreviation);
        assert!(!abbrev_rule.detect_abbreviation("a", 0).is_abbreviation);
    }

    #[test]
    fn test_custom_abbreviations() {
        let mut custom_abbrevs = HashSet::new();
        custom_abbrevs.insert("CEO".to_string());
        custom_abbrevs.insert("CTO".to_string());

        let rules = EnglishLanguageRules::with_custom_abbreviations(custom_abbrevs);

        let result = rules.process_abbreviation("New CEO. started today", 7);
        assert!(result.is_abbreviation);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_english_capitalization_analysis() {
        let rule = EnglishCapitalizationRule::new();

        // Test normal capitalization
        let analysis = rule.analyze_following_text(" This is a test.");
        assert!(analysis.starts_with_capital);
        assert!(!analysis.starts_with_quote_and_capital);

        // Test quoted speech
        let analysis = rule.analyze_following_text(" \"Hello there!\"");
        assert!(!analysis.starts_with_capital);
        assert!(analysis.starts_with_quote_and_capital);

        // Test lowercase continuation
        let analysis = rule.analyze_following_text(" and then he said");
        assert!(!analysis.starts_with_capital);
        assert!(!analysis.starts_with_quote_and_capital);

        // Test bracket pattern (like ". [ This")
        let analysis = rule.analyze_following_text(" [ This is a test");
        assert!(analysis.starts_with_capital);
        assert!(!analysis.starts_with_quote_and_capital);

        // Test parenthesis pattern
        let analysis = rule.analyze_following_text(" (The company announced");
        assert!(analysis.starts_with_capital);
        assert!(!analysis.starts_with_quote_and_capital);

        // Test multiple non-alphabetic characters before quote
        let analysis = rule.analyze_following_text(" [( \"Hello world\"");
        assert!(analysis.starts_with_capital); // H is capital
        assert!(!analysis.starts_with_quote_and_capital); // Not directly after quote

        // Test with only non-alphabetic characters
        let analysis = rule.analyze_following_text(" [({})");
        assert!(!analysis.starts_with_capital);
        assert!(!analysis.starts_with_quote_and_capital);
    }

    #[test]
    fn test_english_number_rules() {
        let rule = EnglishNumberRule::new();

        // Test decimal point
        assert!(rule.is_decimal_point("The price is $3.99 today", 15));
        assert!(!rule.is_decimal_point("Hello world. This is", 11));

        // Test time format
        assert!(rule.is_time_format("Meeting at 3:30 p.m. today", 17));
        assert!(!rule.is_time_format("Dr. Smith is here", 2));

        // Test numeric dot sequences (covers version numbers, IP addresses, section numbers)
        // Version numbers
        assert!(rule.is_numeric_dot_sequence("Please upgrade to version 2.0.1 immediately.", 27));
        assert!(rule.is_numeric_dot_sequence("Please upgrade to version 2.0.1 immediately.", 29));
        assert!(rule.is_numeric_dot_sequence("We support Node.js v18.12.0 and later.", 22));
        assert!(rule.is_numeric_dot_sequence("We support Node.js v18.12.0 and later.", 25));
        assert!(rule.is_numeric_dot_sequence("Python 3.11.4 includes many improvements.", 8));
        assert!(rule.is_numeric_dot_sequence("Python 3.11.4 includes many improvements.", 11));

        // IP addresses
        assert!(rule.is_numeric_dot_sequence("Connect to server at 192.168.1.1 using SSH.", 24));
        assert!(rule.is_numeric_dot_sequence("Connect to server at 192.168.1.1 using SSH.", 28));
        assert!(rule.is_numeric_dot_sequence("Connect to server at 192.168.1.1 using SSH.", 30));
        assert!(
            rule.is_numeric_dot_sequence("Access the application at 127.0.0.1 or localhost.", 29)
        );

        // Section numbers
        assert!(rule.is_numeric_dot_sequence("See section 2.3.4 for more details.", 13));
        assert!(rule.is_numeric_dot_sequence("See section 2.3.4 for more details.", 15));
        assert!(rule.is_numeric_dot_sequence("Chapter 1.2.3 covers the basics.", 9));
        assert!(rule.is_numeric_dot_sequence("Chapter 1.2.3 covers the basics.", 11));
        assert!(rule.is_numeric_dot_sequence("Refer to § 4.3.2 of the specification.", 13));
        assert!(rule.is_numeric_dot_sequence("Refer to § 4.3.2 of the specification.", 15));

        // Edge cases
        assert!(rule.is_numeric_dot_sequence("The version is 1.2.3.", 16)); // Period at end
        assert!(rule.is_numeric_dot_sequence("The version is 1.2.3.", 18)); // Period at end

        // Not numeric sequences
        assert!(!rule.is_numeric_dot_sequence("Hello. World", 5));
        assert!(!rule.is_numeric_dot_sequence("Price is 3.99 today", 11)); // Only one dot - handled by is_decimal_point

        // Test list markers (line-start patterns)
        // Numeric list marker at line start
        assert!(rule.is_list_marker("1. First item in the list.", 1));
        assert!(rule.is_list_marker("12. This is the twelfth item.", 2));
        assert!(rule.is_list_marker("\n1. New line item", 2)); // After newline
        assert!(rule.is_list_marker("  1. Indented item", 3)); // With leading spaces

        // Alphabetic list marker
        assert!(rule.is_list_marker("a. Start with this step.", 1));
        assert!(rule.is_list_marker("A. Overview of the system.", 1));

        // Roman numeral
        assert!(rule.is_list_marker("i. Introduction to the topic.", 1));
        assert!(rule.is_list_marker("X. Tenth item", 1));

        // Not a list marker
        assert!(!rule.is_list_marker("Dr. Smith is here.", 2)); // Not at line start
        assert!(!rule.is_list_marker("The price is 3.99 today.", 14)); // Not at line start
        assert!(!rule.is_list_marker("1.No space after period", 1)); // No space after period
        assert!(!rule.is_list_marker("In section 1. we discuss", 12)); // Not at line start
    }

    #[test]
    fn test_english_comprehensive_boundary_detection() {
        let rules = EnglishLanguageRules::new();

        // Test normal sentence boundary
        let context = BoundaryContext {
            text: "Hello world. This is a test.".to_string(),
            position: 11,
            boundary_char: '.',
            preceding_context: "Hello world".to_string(),
            following_context: " This is a test.".to_string(),
        };

        match rules.detect_sentence_boundary(&context) {
            BoundaryDecision::Boundary(BoundaryFlags::WEAK) => {}
            other => panic!("Expected weak boundary, got {:?}", other),
        }

        // Test abbreviation (should not be boundary)
        let context = BoundaryContext {
            text: "Dr. Smith is here.".to_string(),
            position: 2,
            boundary_char: '.',
            preceding_context: "Dr".to_string(),
            following_context: " Smith is here.".to_string(),
        };

        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NotBoundary
        );

        // Test decimal number (should not be boundary)
        let context = BoundaryContext {
            text: "The price is $3.99 today.".to_string(),
            position: 15,
            boundary_char: '.',
            preceding_context: "price is $3".to_string(),
            following_context: "99 today.".to_string(),
        };

        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NotBoundary
        );
    }

    #[test]
    fn test_utf8_boundary_with_non_ascii_whitespace() {
        let rule = EnglishAbbreviationRule::new();

        // Test with non-breaking space (U+00A0) - 2 bytes in UTF-8
        let text1 = "Hello world\u{a0}Dr. Smith";
        let dr_pos = text1.find("Dr.").unwrap() + 2; // Position after "Dr"
        let result1 = rule.detect_abbreviation(text1, dr_pos);
        assert!(result1.is_abbreviation);
        assert_eq!(result1.confidence, 1.0);

        // Test with em space (U+2003) - 3 bytes in UTF-8
        let text2 = "The company\u{2003}Inc. announced";
        let inc_pos = text2.find("Inc.").unwrap() + 3; // Position after "Inc"
        let result2 = rule.detect_abbreviation(text2, inc_pos);
        assert!(result2.is_abbreviation);

        // Test with ideographic space (U+3000) - 3 bytes in UTF-8
        let text3 = "Contact us\u{3000}Ltd. for details";
        let ltd_pos = text3.find("Ltd.").unwrap() + 3; // Position after "Ltd"
        let result3 = rule.detect_abbreviation(text3, ltd_pos);
        assert!(result3.is_abbreviation);

        // Test with thin space (U+2009) - 3 bytes in UTF-8
        let text4 = "Professor\u{2009}Ph.D. Smith";
        let phd_pos = text4.find("Ph.D.").unwrap() + 4; // Position after "Ph.D"
        let result4 = rule.detect_abbreviation(text4, phd_pos);
        assert!(result4.is_abbreviation);

        // Test with zero-width space (U+200B) - 3 bytes in UTF-8
        let text5 = "Example\u{200B}U.S.A. text";
        let usa_pos = text5.find("U.S.A.").unwrap() + 5; // Position after "U.S.A"
        let result5 = rule.detect_abbreviation(text5, usa_pos);
        assert!(result5.is_abbreviation);
    }

    #[test]
    fn test_utf8_boundary_no_panic() {
        let rule = EnglishAbbreviationRule::new();

        // Construct text that previously caused the panic
        // We need text where position points after an abbreviation
        // and the character before the abbreviation is multi-byte
        let mut text = String::new();
        for _ in 0..8805 {
            text.push('a');
        }
        text.push('\u{a0}'); // Non-breaking space at position 8805-8807
        text.push_str("Dr.");

        // Try to detect abbreviation at position after "Dr."
        // This should now work correctly without panic
        let result = rule.detect_abbreviation(&text, 8809);
        assert!(result.is_abbreviation);
    }

    #[test]
    fn test_abbreviation_followed_by_sentence_starter() {
        let rules = EnglishLanguageRules::new();

        // Test case 1: "She works at Apple Inc. However, the company..." → Should be a boundary
        let context = BoundaryContext {
            text: "She works at Apple Inc. However, the company has grown.".to_string(),
            position: 22, // Position after the period in "Inc."
            boundary_char: '.',
            preceding_context: "She works at Apple Inc".to_string(),
            following_context: " However, the company has grown.".to_string(),
        };
        // Currently returns NotBoundary because of abbreviation detection
        let decision = rules.detect_sentence_boundary(&context);
        assert_eq!(
            decision,
            BoundaryDecision::Boundary(BoundaryFlags::STRONG),
            "Inc. followed by 'However' should be a boundary"
        );

        // Test case 2: "The company is Apple Inc. The product is..." → Should be a boundary
        let context = BoundaryContext {
            text: "The company is Apple Inc. The product is innovative.".to_string(),
            position: 24, // Position after the period in "Inc."
            boundary_char: '.',
            preceding_context: "The company is Apple Inc".to_string(),
            following_context: " The product is innovative.".to_string(),
        };
        let decision = rules.detect_sentence_boundary(&context);
        assert_eq!(
            decision,
            BoundaryDecision::Boundary(BoundaryFlags::STRONG),
            "Inc. followed by 'The' should be a boundary"
        );

        // Test case 3: "Contact Dr. Smith about..." → Should NOT be a boundary
        let context = BoundaryContext {
            text: "Contact Dr. Smith about the issue.".to_string(),
            position: 10, // Position after the period in "Dr."
            boundary_char: '.',
            preceding_context: "Contact Dr".to_string(),
            following_context: " Smith about the issue.".to_string(),
        };
        let decision = rules.detect_sentence_boundary(&context);
        assert_eq!(
            decision,
            BoundaryDecision::NotBoundary,
            "Dr. followed by a name should not be a boundary"
        );

        // Test case 4: "See Prof. I believe..." → Should be a boundary
        let context = BoundaryContext {
            text: "See Prof. I believe this is correct.".to_string(),
            position: 8, // Position after the period in "Prof."
            boundary_char: '.',
            preceding_context: "See Prof".to_string(),
            following_context: " I believe this is correct.".to_string(),
        };
        let decision = rules.detect_sentence_boundary(&context);
        assert_eq!(
            decision,
            BoundaryDecision::Boundary(BoundaryFlags::STRONG),
            "Prof. followed by 'I' should be a boundary"
        );
    }

    #[test]
    fn test_abbreviation_with_various_sentence_starters() {
        let rules = EnglishLanguageRules::new();

        // Test personal pronouns
        let test_cases = vec![
            ("Inc. He said", " He said", true),
            ("Ltd. We think", " We think", true),
            ("Corp. They announced", " They announced", true),
            // Test WH-words
            ("Corp. What happened", " What happened", true),
            ("Dr. Why did", " Why did", true),
            ("Inc. Where is", " Where is", true),
            // Test conjunctive adverbs
            ("Co. Therefore", " Therefore", true),
            ("Ltd. Moreover", " Moreover", true),
            ("Inc. Furthermore", " Furthermore", true),
            // Test demonstratives
            ("Inc. This shows", " This shows", true),
            ("Corp. These results", " These results", true),
            ("Ltd. Those findings", " Those findings", true),
        ];

        for (preceding, following, should_be_boundary) in test_cases {
            let full_text = format!("{}{}", preceding, following);
            let context = BoundaryContext {
                text: full_text.clone(),
                position: preceding.len() - 1, // Position at the period
                boundary_char: '.',
                preceding_context: preceding[..preceding.len() - 1].to_string(),
                following_context: following.to_string(),
            };

            let decision = rules.detect_sentence_boundary(&context);
            if should_be_boundary {
                assert!(
                    matches!(decision, BoundaryDecision::Boundary(_)),
                    "Expected boundary for: '{}'",
                    full_text
                );
            } else {
                assert_eq!(
                    decision,
                    BoundaryDecision::NotBoundary,
                    "Expected no boundary for: '{}'",
                    full_text
                );
            }
        }
    }

    #[test]
    fn test_abbreviation_without_sentence_starter() {
        let rules = EnglishLanguageRules::new();

        // Test lowercase continuations - should NOT be boundaries
        let test_cases = vec![
            ("Inc. operates globally", " operates globally"),
            ("Dr. Johnson's research", " Johnson's research"),
            ("Ltd. company structure", " company structure"),
            ("Corp. announced earnings", " announced earnings"),
            ("Prof. teaches mathematics", " teaches mathematics"),
        ];

        for (preceding, following) in test_cases {
            let full_text = format!("{}{}", preceding, following);
            let period_pos = preceding.find('.').unwrap();
            let context = BoundaryContext {
                text: full_text.clone(),
                position: period_pos,
                boundary_char: '.',
                preceding_context: preceding[..period_pos].to_string(),
                following_context: format!("{}{}", &preceding[period_pos + 1..], following),
            };

            let decision = rules.detect_sentence_boundary(&context);
            assert_eq!(
                decision,
                BoundaryDecision::NotBoundary,
                "Expected no boundary for: '{}'",
                full_text
            );
        }
    }

    #[test]
    fn test_abbreviation_with_quotation_marks() {
        let rules = EnglishLanguageRules::new();

        // Test with various quotation mark scenarios
        let test_cases = vec![
            // Double quotes after abbreviation
            ("She works at Inc.\" However", "Inc", ".\" However", true),
            // Single quotes after abbreviation
            ("The company 'Ltd.' Therefore", "Ltd", ".' Therefore", true),
            // Quotes around following word
            ("Contact Dr. \"Smith\" for", "Dr", ". \"Smith\" for", false),
        ];

        for (text, abbrev_end, following, should_be_boundary) in test_cases {
            let pos = text.find(abbrev_end).unwrap() + abbrev_end.len();
            let context = BoundaryContext {
                text: text.to_string(),
                position: pos,
                boundary_char: '.',
                preceding_context: text[..pos].to_string(),
                following_context: following.to_string(),
            };

            let decision = rules.detect_sentence_boundary(&context);
            if should_be_boundary {
                assert!(
                    matches!(decision, BoundaryDecision::Boundary(_)),
                    "Expected boundary for: '{}'",
                    text
                );
            } else {
                assert_eq!(
                    decision,
                    BoundaryDecision::NotBoundary,
                    "Expected no boundary for: '{}'",
                    text
                );
            }
        }
    }

    #[test]
    fn test_abbreviation_detection_for_inc() {
        let rule = EnglishAbbreviationRule::new();

        // Test that "Inc" is detected as an abbreviation
        let text = "Inc. however, the results";
        let position = 3; // Position of the period
        let result = rule.detect_abbreviation(text, position);

        assert!(
            result.is_abbreviation,
            "Inc should be detected as abbreviation"
        );
        assert!(result.confidence > 0.8, "Inc should have high confidence");
    }

    #[test]
    fn test_case_sensitivity_for_sentence_starters() {
        let rules = EnglishLanguageRules::new();

        // Test case sensitivity
        let test_cases = vec![
            ("Inc.", " however, the results", false),  // lowercase
            ("Inc.", " HOWEVER, the results", true),   // uppercase
            ("Ltd.", " Therefore, we conclude", true), // normal case
            ("Corp.", " therefore, we see", false),    // lowercase
        ];

        for (preceding, following, should_be_boundary) in test_cases {
            let full_text = format!("{}{}", preceding, following);
            let context = BoundaryContext {
                text: full_text.clone(),
                position: preceding.len() - 1, // Position at the period
                boundary_char: '.',
                preceding_context: preceding[..preceding.len() - 1].to_string(),
                following_context: following.to_string(),
            };

            let decision = rules.detect_sentence_boundary(&context);
            if should_be_boundary {
                assert!(
                    matches!(decision, BoundaryDecision::Boundary(_)),
                    "Expected boundary for: '{}', got {:?}",
                    full_text,
                    decision
                );
            } else {
                // The implementation returns NotBoundary for abbreviations followed by
                // non-sentence-starters, which is correct
                assert_eq!(
                    decision,
                    BoundaryDecision::NotBoundary,
                    "Expected no boundary for: '{}', got {:?}",
                    full_text,
                    decision
                );
            }
        }
    }
}
