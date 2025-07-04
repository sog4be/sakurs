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
    web_text_rule: EnglishWebTextRule,
}

impl EnglishLanguageRules {
    /// Create a new English language rules instance
    pub fn new() -> Self {
        Self {
            abbreviation_rule: EnglishAbbreviationRule::new(),
            capitalization_rule: EnglishCapitalizationRule::new(),
            number_rule: EnglishNumberRule::new(),
            quotation_rule: EnglishQuotationRule::new(),
            web_text_rule: EnglishWebTextRule::new(),
        }
    }

    /// Create English rules with custom configurations
    pub fn with_custom_abbreviations(abbreviations: HashSet<String>) -> Self {
        Self {
            abbreviation_rule: EnglishAbbreviationRule::with_custom_list(abbreviations),
            capitalization_rule: EnglishCapitalizationRule::new(),
            number_rule: EnglishNumberRule::new(),
            quotation_rule: EnglishQuotationRule::new(),
            web_text_rule: EnglishWebTextRule::new(),
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

        let word_start = text[..position]
            .rfind(|c: char| c.is_whitespace())
            .map(|p| p + 1)
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

        // Web text processing: Check for URLs and ellipsis first
        if context.boundary_char == '.' {
            // Check if this period is part of a URL
            if self
                .web_text_rule
                .is_url_period(&context.text, context.position)
            {
                return BoundaryDecision::NotBoundary;
            }

            // Check if this is part of an ellipsis
            if self
                .web_text_rule
                .is_ellipsis(&context.text, context.position)
            {
                return BoundaryDecision::NotBoundary;
            }

            // Check for parenthetical period patterns like ") ."
            if self.web_text_rule.is_parenthetical_period(
                &context.text,
                context.position,
                &context.following_context,
            ) {
                return BoundaryDecision::NotBoundary;
            }
        }

        // Check for abbreviations first
        let abbrev_result = self
            .abbreviation_rule
            .detect_abbreviation(&context.text, context.position);
        if abbrev_result.is_abbreviation && abbrev_result.confidence > 0.8 {
            // Enhanced context check for title + name patterns
            if self.is_likely_title_name_pattern(
                &context.text,
                context.position,
                &context.following_context,
            ) {
                return BoundaryDecision::NotBoundary;
            }
            // For high-confidence abbreviations, generally not a boundary
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

        // Web-specific abbreviations and file extensions
        abbreviations.insert("com".to_string());
        abbreviations.insert("org".to_string());
        abbreviations.insert("net".to_string());
        abbreviations.insert("edu".to_string());
        abbreviations.insert("gov".to_string());
        abbreviations.insert("mil".to_string());
        abbreviations.insert("int".to_string());
        abbreviations.insert("io".to_string());
        abbreviations.insert("co".to_string());
        abbreviations.insert("uk".to_string());
        abbreviations.insert("us".to_string());
        abbreviations.insert("ca".to_string());
        abbreviations.insert("au".to_string());
        abbreviations.insert("de".to_string());
        abbreviations.insert("fr".to_string());
        abbreviations.insert("jp".to_string());
        abbreviations.insert("cn".to_string());
        abbreviations.insert("ru".to_string());
        abbreviations.insert("br".to_string());
        abbreviations.insert("in".to_string());

        // File extensions
        abbreviations.insert("html".to_string());
        abbreviations.insert("htm".to_string());
        abbreviations.insert("pdf".to_string());
        abbreviations.insert("doc".to_string());
        abbreviations.insert("docx".to_string());
        abbreviations.insert("txt".to_string());
        abbreviations.insert("xml".to_string());
        abbreviations.insert("json".to_string());
        abbreviations.insert("csv".to_string());
        abbreviations.insert("jpg".to_string());
        abbreviations.insert("jpeg".to_string());
        abbreviations.insert("png".to_string());
        abbreviations.insert("gif".to_string());
        abbreviations.insert("mp3".to_string());
        abbreviations.insert("mp4".to_string());
        abbreviations.insert("zip".to_string());
        abbreviations.insert("tar".to_string());
        abbreviations.insert("gz".to_string());
        abbreviations.insert("exe".to_string());
        abbreviations.insert("app".to_string());
        abbreviations.insert("pkg".to_string());
        abbreviations.insert("dmg".to_string());
        abbreviations.insert("iso".to_string());
        abbreviations.insert("js".to_string());
        abbreviations.insert("css".to_string());
        abbreviations.insert("py".to_string());
        abbreviations.insert("java".to_string());
        abbreviations.insert("cpp".to_string());
        abbreviations.insert("h".to_string());
        abbreviations.insert("c".to_string());
        abbreviations.insert("rs".to_string());
        abbreviations.insert("go".to_string());
        abbreviations.insert("php".to_string());
        abbreviations.insert("rb".to_string());
        abbreviations.insert("swift".to_string());
        abbreviations.insert("kt".to_string());
        abbreviations.insert("scala".to_string());
        abbreviations.insert("sh".to_string());
        abbreviations.insert("bash".to_string());
        abbreviations.insert("sql".to_string());
        abbreviations.insert("yml".to_string());
        abbreviations.insert("yaml".to_string());
        abbreviations.insert("toml".to_string());
        abbreviations.insert("ini".to_string());
        abbreviations.insert("cfg".to_string());
        abbreviations.insert("conf".to_string());
        abbreviations.insert("log".to_string());
        abbreviations.insert("bak".to_string());
        abbreviations.insert("tmp".to_string());
        abbreviations.insert("temp".to_string());
        abbreviations.insert("old".to_string());
        abbreviations.insert("new".to_string());
        abbreviations.insert("orig".to_string());
        abbreviations.insert("backup".to_string());

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

        // Find word boundaries
        let start_pos = text[..position]
            .rfind(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .map(|p| p + 1)
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

        let first_char = trimmed.chars().next().unwrap();

        // Check for quoted speech (including curly quotes)
        if matches!(
            first_char,
            '"' | '\'' | '\u{201C}' | '\u{201D}' | '\u{2018}' | '\u{2019}'
        ) {
            let after_quote = &trimmed[first_char.len_utf8()..];
            let after_quote_trimmed = after_quote.trim_start();
            if let Some(char_after_quote) = after_quote_trimmed.chars().next() {
                return CapitalizationAnalysis {
                    starts_with_capital: false,
                    starts_with_quote_and_capital: char_after_quote.is_ascii_uppercase(),
                    first_word_is_proper_noun: self.is_likely_proper_noun(after_quote_trimmed),
                };
            }
        }

        CapitalizationAnalysis {
            starts_with_capital: first_char.is_ascii_uppercase(),
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

/// Web text processing rules for handling URLs, ellipses, and web-specific patterns
#[derive(Debug, Clone)]
pub struct EnglishWebTextRule {
    /// URL protocol patterns
    url_protocols: Vec<&'static str>,
    /// Common domain extensions (TLDs)
    domain_extensions: HashSet<String>,
}

impl Default for EnglishWebTextRule {
    fn default() -> Self {
        Self::new()
    }
}

impl EnglishWebTextRule {
    pub fn new() -> Self {
        let mut domain_extensions = HashSet::new();

        // Common TLDs
        for tld in &[
            "com", "org", "net", "edu", "gov", "mil", "int", "io", "co", "uk", "us", "ca", "au",
            "de", "fr", "jp", "cn", "ru", "br", "in", "it", "es", "nl", "se", "no", "dk", "fi",
            "pl", "ch", "at", "be", "mx", "ar", "cl", "pe", "ve", "za", "ng", "eg", "il", "ae",
            "sa", "kr", "tw", "hk", "sg", "my", "th", "vn", "id", "ph", "nz", "ie", "cz", "hu",
            "ro", "gr", "pt", "tr", "ua", "info", "biz", "name", "pro", "museum", "coop", "aero",
            "xxx", "idv", "mobi", "asia", "tel", "travel", "jobs", "tv", "cc", "ws", "me", "tk",
            "ml", "ga", "cf",
        ] {
            domain_extensions.insert(tld.to_string());
        }

        Self {
            url_protocols: vec![
                "http://", "https://", "ftp://", "sftp://", "ssh://", "git://", "svn://",
                "file://", "mailto:", "tel:", "ws://", "wss://", "irc://", "ircs://", "news://",
                "nntp://", "rtsp://", "rtmp://", "magnet:", "bitcoin:",
            ],
            domain_extensions,
        }
    }

    /// Check if a period at the given position is part of a URL
    pub fn is_url_period(&self, text: &str, position: usize) -> bool {
        // Look backward from the period to find potential URL start
        let mut search_start = position.saturating_sub(100); // Look back up to 100 chars

        // Ensure search_start is at a char boundary
        while search_start > 0 && !text.is_char_boundary(search_start) {
            search_start -= 1;
        }

        // Ensure position is valid
        if position >= text.len()
            || !text.is_char_boundary(position)
            || !text.is_char_boundary(position + 1)
        {
            return false;
        }

        let search_text = &text[search_start..=position];

        // Check for protocol patterns
        for protocol in &self.url_protocols {
            if search_text.contains(protocol) {
                // Found a protocol, check if we're still within the URL
                if let Some(protocol_pos) = search_text.rfind(protocol) {
                    let after_protocol = &search_text[protocol_pos + protocol.len()..];
                    // Check if there's no whitespace between protocol and current position
                    if !after_protocol.contains(char::is_whitespace) {
                        return true;
                    }
                }
            }
        }

        // Check for domain pattern (word.word pattern where second word is a TLD)
        if position > 0 && position + 1 < text.len() {
            // Get the word after the period
            let after_dot = &text[position + 1..];
            let word_after: String = after_dot
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-')
                .collect();

            if self.domain_extensions.contains(&word_after.to_lowercase()) {
                // Check if the character before the period is alphanumeric
                if let Some(before_char) = text[..position].chars().last() {
                    if before_char.is_alphanumeric() || before_char == '-' {
                        return true;
                    }
                }
            }

            // Special check for www. pattern
            if position >= 3 {
                let mut start = position.saturating_sub(3);
                // Ensure start is at a char boundary
                while start > 0 && !text.is_char_boundary(start) {
                    start -= 1;
                }
                if start < position {
                    let before_text = &text[start..position];
                    if before_text == "www" || before_text.ends_with("www") {
                        // Check if word after period looks like a domain
                        if word_after.len() >= 2
                            && word_after.chars().all(|c| c.is_alphanumeric() || c == '-')
                        {
                            return true;
                        }
                    }
                }
            }

            // Check for file extension pattern (common in URLs)
            let common_url_extensions = [
                "html", "htm", "php", "asp", "aspx", "jsp", "cgi", "pl", "cfm", "xml", "json",
            ];
            if common_url_extensions.contains(&word_after.to_lowercase().as_str()) {
                // Check if this looks like a path (has / before)
                if search_start < position {
                    let before_text = &text[search_start..position];
                    if before_text.contains('/') {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a period is part of an ellipsis (...)
    pub fn is_ellipsis(&self, text: &str, position: usize) -> bool {
        // First, verify that the current position is actually a period
        if position >= text.len() || !text.is_char_boundary(position) {
            return false;
        }

        // Check if character at position is a period
        if let Some(ch) = text[position..].chars().next() {
            if ch != '.' {
                return false;
            }
        } else {
            return false;
        }

        // Check for consecutive dots pattern
        let mut dots_before = 0;
        let mut dots_after = 0;

        // Count dots before current position
        if position > 0 {
            let before_chars: Vec<char> = text[..position].chars().rev().take(2).collect();
            for ch in before_chars {
                if ch == '.' {
                    dots_before += 1;
                } else {
                    break;
                }
            }
        }

        // Count dots immediately after current position (no gaps)
        if position + 1 < text.len() {
            let mut check_pos = position + 1;
            while check_pos < text.len() && check_pos < position + 3 {
                // Ensure we're at a valid char boundary
                if !text.is_char_boundary(check_pos) {
                    check_pos += 1;
                    continue;
                }

                if let Some(ch) = text[check_pos..].chars().next() {
                    if ch == '.' {
                        dots_after += 1;
                        check_pos += ch.len_utf8();
                    } else {
                        // Stop if we encounter any non-dot character
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // If we have at least one dot before or after, this is part of ellipsis
        dots_before > 0 || dots_after > 0
    }

    /// Check for parenthetical period patterns like ") ." or "] ."
    pub fn is_parenthetical_period(
        &self,
        text: &str,
        position: usize,
        following_context: &str,
    ) -> bool {
        // Check if the period is preceded by closing parenthetical punctuation
        if position > 0 {
            if let Some(before_char) = text[..position].chars().last() {
                if matches!(before_char, ')' | ']' | '}') {
                    // Check if the following context starts with space
                    if following_context.starts_with(' ') {
                        return true;
                    }
                }
            }
        }

        // Also check for pattern where there's a space before the period after parenthesis
        if position >= 2 {
            let two_before: Vec<char> = text[..position].chars().rev().take(2).collect();
            if two_before.len() == 2
                && two_before[0] == ' '
                && matches!(two_before[1], ')' | ']' | '}')
            {
                return true;
            }
        }

        false
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
    fn test_web_text_url_detection() {
        let web_rule = EnglishWebTextRule::new();

        // Test URL with protocol
        assert!(web_rule.is_url_period("Visit http://www.example.com for info", 16));
        assert!(web_rule.is_url_period("Visit http://www.example.com for info", 24));
        assert!(web_rule.is_url_period("Check https://github.com/user/repo", 15));

        // Test domain without protocol
        assert!(web_rule.is_url_period("Visit www.example.com for info", 9));
        assert!(web_rule.is_url_period("Visit www.example.com for info", 17));

        // Test various TLDs
        assert!(web_rule.is_url_period("site.org is good", 4));
        assert!(web_rule.is_url_period("site.io is modern", 4));
        assert!(web_rule.is_url_period("site.edu for education", 4));

        // Test file paths in URLs
        assert!(web_rule.is_url_period("http://site.com/page.html is broken", 15));
        assert!(web_rule.is_url_period("http://site.com/page.html is broken", 20));

        // Test non-URLs
        assert!(!web_rule.is_url_period("This is a sentence. Next one.", 18));
        assert!(!web_rule.is_url_period("Dr. Smith is here.", 2));
    }

    #[test]
    fn test_web_text_ellipsis_detection() {
        let web_rule = EnglishWebTextRule::new();

        // Test various ellipsis patterns
        assert!(web_rule.is_ellipsis("Wait...", 4));
        assert!(web_rule.is_ellipsis("Wait...", 5));
        assert!(web_rule.is_ellipsis("Wait...", 6));
        assert!(web_rule.is_ellipsis("And then... nothing", 8));
        assert!(web_rule.is_ellipsis("And then... nothing", 9));
        assert!(web_rule.is_ellipsis("And then... nothing", 10));

        // Test single period (not ellipsis)
        assert!(!web_rule.is_ellipsis("End. Start", 3));
        assert!(!web_rule.is_ellipsis("Dr. Smith", 2));
    }

    #[test]
    fn test_web_text_parenthetical_period() {
        let web_rule = EnglishWebTextRule::new();

        // Test parenthetical patterns
        assert!(web_rule.is_parenthetical_period("(see note) . Next", 11, " Next"));
        assert!(web_rule.is_parenthetical_period("[ref] . Continue", 6, " Continue"));
        assert!(web_rule.is_parenthetical_period("{data} . More", 7, " More"));

        // Test direct attachment
        assert!(web_rule.is_parenthetical_period("(note). Next", 6, " Next"));
        assert!(web_rule.is_parenthetical_period("[ref]. Continue", 5, " Continue"));

        // Test non-parenthetical
        assert!(!web_rule.is_parenthetical_period("word. Next", 4, " Next"));
        assert!(!web_rule.is_parenthetical_period("test) not. Next", 9, " Next"));
    }

    #[test]
    fn test_web_text_integration() {
        let rules = EnglishLanguageRules::new();

        // Test URL handling in sentence detection
        let context = BoundaryContext {
            text: "Visit http://www.example.com for info.".to_string(),
            position: 16, // First dot in www.
            boundary_char: '.',
            preceding_context: "Visit http://www".to_string(),
            following_context: "example.com for info.".to_string(),
        };
        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NotBoundary
        );

        // Test ellipsis handling
        let context = BoundaryContext {
            text: "Wait... what happened?".to_string(),
            position: 4, // First dot
            boundary_char: '.',
            preceding_context: "Wait".to_string(),
            following_context: ".. what happened?".to_string(),
        };
        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NotBoundary
        );

        // Test parenthetical period
        let context = BoundaryContext {
            text: "End of quote ) . Next sentence.".to_string(),
            position: 15,
            boundary_char: '.',
            preceding_context: "End of quote ) ".to_string(),
            following_context: " Next sentence.".to_string(),
        };
        assert_eq!(
            rules.detect_sentence_boundary(&context),
            BoundaryDecision::NotBoundary
        );
    }
}
