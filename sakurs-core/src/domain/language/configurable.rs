use crate::domain::{
    enclosure::EnclosureChar,
    enclosure_suppressor::EnclosureSuppressor,
    error::DomainError,
    language::{
        config::{get_language_config, LanguageConfig, SentenceStarterConfig},
        rules::{
            AbbreviationTrie, EllipsisRules, EnclosureMap, PatternContext, Suppressor,
            TerminatorRules,
        },
        traits::{
            AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRules, QuotationContext,
            QuotationDecision,
        },
    },
    BoundaryFlags,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Extract the next word from the following context and return (word, context_after_word)
fn extract_next_word(following_context: &str) -> Option<(String, &str)> {
    let mut char_indices = following_context.char_indices().peekable();

    // Skip whitespace and track position
    let mut word_start = 0;
    while let Some((idx, ch)) = char_indices.peek() {
        if ch.is_whitespace() {
            word_start = *idx + ch.len_utf8();
            char_indices.next();
        } else {
            break;
        }
    }

    // Extract word characters
    let mut word = String::new();
    let mut word_end = word_start;
    while let Some((idx, ch)) = char_indices.peek() {
        if ch.is_alphabetic() {
            word.push(*ch);
            word_end = *idx + ch.len_utf8();
            char_indices.next();
        } else {
            word_end = *idx;
            break;
        }
    }

    // If we consumed all characters, word_end should be the length
    if word_end == word_start && !word.is_empty() {
        word_end = following_context.len();
    }

    if word.is_empty() {
        None
    } else {
        Some((word, &following_context[word_end..]))
    }
}

/// Configurable language rules based on TOML configuration
pub struct ConfigurableLanguageRules {
    /// Language metadata
    code: String,
    name: String,

    /// Rule components
    terminator_rules: TerminatorRules,
    ellipsis_rules: EllipsisRules,
    abbreviation_trie: AbbreviationTrie,
    enclosure_map: EnclosureMap,
    suppressor: Suppressor,

    /// Sentence starter configuration
    sentence_starter_config: SentenceStarterConfig,
    /// Fast lookup set for sentence starters
    sentence_starter_set: HashSet<String>,
}

impl ConfigurableLanguageRules {
    /// Create language rules from a language code
    pub fn from_code(code: &str) -> Result<Self, DomainError> {
        let config = get_language_config(code)?;
        Self::from_config(config)
    }

    /// Create language rules from external file
    pub fn from_file(path: &Path, language_code: Option<&str>) -> Result<Self, DomainError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            DomainError::ConfigurationError(format!(
                "Failed to read file '{}': {}",
                path.display(),
                e
            ))
        })?;

        let mut config: LanguageConfig = toml::from_str(&content).map_err(|e| {
            DomainError::ConfigurationError(format!(
                "Failed to parse TOML from '{}': {}",
                path.display(),
                e
            ))
        })?;

        // Override language code if provided
        if let Some(code) = language_code {
            config.metadata.code = code.to_string();
        }

        // Validate configuration
        config.validate()?;

        Self::from_config(&config)
    }

    /// Create language rules from configuration
    pub fn from_config(config: &LanguageConfig) -> Result<Self, DomainError> {
        // Build terminator rules
        let terminator_rules = TerminatorRules::new(
            config.terminators.chars.clone(),
            config
                .terminators
                .patterns
                .iter()
                .map(|p| (p.pattern.clone(), p.name.clone()))
                .collect(),
        );

        // Build ellipsis rules
        let ellipsis_rules = EllipsisRules::new(
            config.ellipsis.treat_as_boundary,
            config.ellipsis.patterns.clone(),
            config
                .ellipsis
                .context_rules
                .iter()
                .map(|r| (r.condition.clone(), r.boundary))
                .collect(),
            config
                .ellipsis
                .exceptions
                .iter()
                .map(|e| (e.regex.clone(), e.boundary))
                .collect(),
        )
        .map_err(DomainError::InvalidLanguageRules)?;

        // Build abbreviation trie
        let abbreviation_trie = AbbreviationTrie::from_categories(
            config.abbreviations.categories.clone(),
            false, // Case insensitive by default
        );

        // Build enclosure map
        let enclosure_map = EnclosureMap::new(
            config
                .enclosures
                .pairs
                .iter()
                .map(|p| (p.open, p.close, p.symmetric))
                .collect(),
        );

        // Build suppressor with regex patterns if available
        let suppressor = if config.suppression.regex_patterns.is_empty() {
            Suppressor::new(
                config
                    .suppression
                    .fast_patterns
                    .iter()
                    .map(|p| (p.char, p.line_start, p.before.clone(), p.after.clone()))
                    .collect(),
            )
        } else {
            Suppressor::with_regex_patterns(
                config
                    .suppression
                    .fast_patterns
                    .iter()
                    .map(|p| (p.char, p.line_start, p.before.clone(), p.after.clone()))
                    .collect(),
                config
                    .suppression
                    .regex_patterns
                    .iter()
                    .map(|p| (p.pattern.clone(), p.description.clone()))
                    .collect(),
            )
            .map_err(|e| DomainError::InvalidLanguageRules(format!("Invalid regex pattern: {e}")))?
        };

        // Build sentence starter set for fast lookups (exact match, no normalization)
        let mut sentence_starter_set = HashSet::new();
        let sentence_starter_config = if let Some(ref starters) = config.sentence_starters {
            for words in starters.categories.values() {
                for word in words {
                    if word.len() >= starters.min_word_length {
                        sentence_starter_set.insert(word.clone());
                    }
                }
            }
            starters.clone()
        } else {
            // Default config when not present
            SentenceStarterConfig {
                categories: HashMap::new(),
                require_following_space: true,
                min_word_length: 1,
            }
        };

        Ok(Self {
            code: config.metadata.code.clone(),
            name: config.metadata.name.clone(),
            terminator_rules,
            ellipsis_rules,
            abbreviation_trie,
            enclosure_map,
            suppressor,
            sentence_starter_config,
            sentence_starter_set,
        })
    }
}

impl LanguageRules for ConfigurableLanguageRules {
    fn detect_sentence_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        let ch = context.boundary_char;

        // Check if it's an ellipsis pattern
        if self.ellipsis_rules.is_ellipsis_pattern(context) {
            return self.ellipsis_rules.evaluate_boundary(context);
        }

        // Check if current character is part of an incomplete ellipsis pattern
        // For example, if we're at the first or second '.' of "..."
        if ch == '.' {
            // Check if we're followed by more dots (incomplete ellipsis)
            if let Some(next_ch) = context.following_context.chars().next() {
                if next_ch == '.' {
                    // We're part of an ellipsis pattern but not at the end
                    return BoundaryDecision::NotBoundary;
                }
            }

            // Check if we're preceded by dots (middle of ellipsis)
            if let Some(prev_ch) = context.preceding_context.chars().last() {
                if prev_ch == '.' {
                    // Check if we're followed by another dot
                    if let Some(next_ch) = context.following_context.chars().next() {
                        if next_ch == '.' {
                            // We're in the middle of "..."
                            return BoundaryDecision::NotBoundary;
                        }
                    }
                    // We might be at the end of ellipsis, let the ellipsis rules handle it
                }
            }

            // Check for multi-period abbreviations like U.S.A., Ph.D., etc.
            // Pattern: single letter + period + single letter + period
            if self.is_multi_period_abbreviation_context(context) {
                return BoundaryDecision::NotBoundary;
            }
        }

        // Check if it's a terminator
        if self.terminator_rules.is_terminator(ch) {
            // Create pattern context for pattern matching
            let pattern_context = PatternContext {
                text: context.text,
                position: context.position,
                current_char: ch,
                next_char: context.following_context.chars().next(),
                prev_char: context.preceding_context.chars().last(),
            };

            // Check for terminator patterns
            if let Some(_pattern) = self.terminator_rules.match_pattern(&pattern_context) {
                // Pattern terminators are always strong boundaries
                return BoundaryDecision::Boundary(BoundaryFlags::STRONG);
            }

            // Check if current character is part of a future pattern
            // This prevents creating boundaries at the first character of multi-character patterns
            let next_char = context.following_context.chars().next();
            if let Some(next) = next_char {
                // Check if current + next forms a known pattern
                let potential_pattern = format!("{ch}{next}");
                for (pattern_str, _) in self.terminator_rules.patterns() {
                    if pattern_str == &potential_pattern {
                        // This is the first character of a pattern, don't create boundary
                        return BoundaryDecision::NotBoundary;
                    }
                }
            }

            // Check for abbreviations
            // context.position is the byte offset BEFORE the terminator (period)
            // We check if there's an abbreviation ending at this position
            let abbr_result = self.process_abbreviation(context.text, context.position);
            if abbr_result.is_abbreviation {
                // Check if the next word is a sentence starter
                if let Some((next_word, remaining_context)) =
                    extract_next_word(&context.following_context)
                {
                    if self.is_sentence_starter(&next_word, remaining_context) {
                        // Abbreviation followed by sentence starter - create boundary
                        return BoundaryDecision::Boundary(BoundaryFlags::WEAK);
                    }
                    // Abbreviation followed by non-sentence starter - not a boundary
                    return BoundaryDecision::NotBoundary;
                } else {
                    // No following text (end of input) - create boundary
                    return BoundaryDecision::Boundary(BoundaryFlags::WEAK);
                }
            }

            // Default terminator evaluation
            return self.terminator_rules.evaluate_single_terminator(context);
        }

        BoundaryDecision::NotBoundary
    }

    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        // Look for abbreviations ending before the period
        // position is the position of the period, so we check at position - 1
        if position > 0 {
            if let Some(abbr_match) = self.abbreviation_trie.find_at_position(text, position - 1) {
                // Check for word boundary at the start of the abbreviation
                let abbr_start = position - abbr_match.length;

                // Simple word boundary check: the character before the abbreviation should not be alphanumeric
                let has_word_boundary = if abbr_start == 0 {
                    true // Start of text is a valid boundary
                } else {
                    // Check the character before the abbreviation
                    text.chars()
                        .nth(abbr_start.saturating_sub(1))
                        .map(|ch| !ch.is_alphanumeric())
                        .unwrap_or(true)
                };

                if has_word_boundary {
                    AbbreviationResult {
                        is_abbreviation: true,
                        length: abbr_match.length,
                        confidence: 1.0, // High confidence for exact matches
                    }
                } else {
                    // Abbreviation found but not at word boundary
                    AbbreviationResult {
                        is_abbreviation: false,
                        length: 0,
                        confidence: 0.0,
                    }
                }
            } else {
                AbbreviationResult {
                    is_abbreviation: false,
                    length: 0,
                    confidence: 0.0,
                }
            }
        } else {
            AbbreviationResult {
                is_abbreviation: false,
                length: 0,
                confidence: 0.0,
            }
        }
    }

    fn handle_quotation(&self, _context: &QuotationContext) -> QuotationDecision {
        // Basic quotation handling - can be enhanced later
        QuotationDecision::QuoteStart
    }

    fn language_code(&self) -> &str {
        &self.code
    }

    fn language_name(&self) -> &str {
        &self.name
    }

    fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        self.enclosure_map.get_enclosure_char(ch)
    }

    fn get_enclosure_type_id(&self, ch: char) -> Option<usize> {
        self.enclosure_map.get_type_id(ch)
    }

    fn enclosure_type_count(&self) -> usize {
        self.enclosure_map.type_count()
    }

    fn enclosure_suppressor(&self) -> Option<&dyn EnclosureSuppressor> {
        Some(&self.suppressor)
    }
}

impl ConfigurableLanguageRules {
    /// Check if a word is a sentence starter based on configuration
    /// Returns true if the word is in the configured list and optionally requires following space
    fn is_sentence_starter(&self, word: &str, following_context: &str) -> bool {
        if word.len() < self.sentence_starter_config.min_word_length {
            return false;
        }

        // Direct O(1) HashSet lookup (exact match, no normalization)
        if !self.sentence_starter_set.contains(word) {
            return false;
        }

        // Check if following space is required
        if self.sentence_starter_config.require_following_space {
            // Check if the context starts with whitespace
            following_context
                .chars()
                .next()
                .is_some_and(|c| c.is_whitespace())
        } else {
            true
        }
    }

    /// Check if we're in the middle of a multi-period abbreviation pattern
    /// like U.S.A., Ph.D., M.D., etc.
    fn is_multi_period_abbreviation_context(&self, context: &BoundaryContext) -> bool {
        // We're at a period. Check if:
        // 1. We're preceded by 1-2 letters (to handle Ph.D., M.D., etc.)
        // 2. We're followed by optional whitespace + 1-2 letters + period

        // Check preceding context - should end with 1-2 letters
        let preceding_chars: Vec<char> = context.preceding_context.chars().collect();
        if preceding_chars.is_empty() {
            return false;
        }

        // Look back to find the start of the letter sequence
        let mut letter_count = 0;
        let mut idx = preceding_chars.len();

        while idx > 0 && preceding_chars[idx - 1].is_alphabetic() && letter_count < 3 {
            idx -= 1;
            letter_count += 1;
        }

        // Must have 1-2 letters before the period
        if letter_count == 0 || letter_count > 2 {
            return false;
        }

        // Check that before the letters is either start or non-letter
        if idx > 0 && preceding_chars[idx - 1].is_alphabetic() {
            return false;
        }

        // Check following context - should be optional whitespace + letters + period
        let following_chars: Vec<char> = context.following_context.chars().collect();
        if following_chars.len() < 2 {
            return false;
        }

        // Skip optional whitespace
        let mut idx = 0;
        while idx < following_chars.len() && following_chars[idx].is_whitespace() {
            idx += 1;
        }

        // Need at least 2 more chars (letter + period)
        if idx + 1 >= following_chars.len() {
            return false;
        }

        // Count letters until we hit a non-letter
        let mut letter_count = 0;
        let _letter_start = idx;
        while idx < following_chars.len()
            && following_chars[idx].is_alphabetic()
            && letter_count < 3
        {
            idx += 1;
            letter_count += 1;
        }

        // Must have 1-2 letters and be followed by a period
        if letter_count > 0
            && letter_count <= 2
            && idx < following_chars.len()
            && following_chars[idx] == '.'
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_from_file_valid_config() {
        let toml_content = r#"
[metadata]
code = "custom"
name = "Custom Language"

[terminators]
chars = [".", "!", "?"]

[ellipsis]
patterns = ["..."]

[enclosures]
pairs = [
    { open = "(", close = ")" }
]

[suppression]

[abbreviations]
common = ["etc", "vs"]

[sentence_starters]
common = ["The", "A"]
"#;

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();

        // Test loading from file
        let rules = ConfigurableLanguageRules::from_file(temp_file.path(), None).unwrap();
        assert_eq!(rules.language_code(), "custom");
        assert_eq!(rules.language_name(), "Custom Language");
    }

    #[test]
    fn test_from_file_with_language_code_override() {
        let toml_content = r#"
[metadata]
code = "original"
name = "Original Language"

[terminators]
chars = ["."]

[ellipsis]
patterns = []

[enclosures]
pairs = []

[suppression]

[abbreviations]

[sentence_starters]
common = ["The"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();

        // Test loading with code override
        let rules =
            ConfigurableLanguageRules::from_file(temp_file.path(), Some("overridden")).unwrap();
        assert_eq!(rules.language_code(), "overridden");
        assert_eq!(rules.language_name(), "Original Language");
    }

    #[test]
    fn test_from_file_invalid_toml() {
        let invalid_toml = r#"
[metadata
code = "test"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", invalid_toml).unwrap();

        let result = ConfigurableLanguageRules::from_file(temp_file.path(), None);
        assert!(result.is_err());
        match result {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("Failed to parse TOML"));
            }
            _ => panic!("Expected ConfigurationError for invalid TOML"),
        }
    }

    #[test]
    fn test_from_file_nonexistent() {
        let result =
            ConfigurableLanguageRules::from_file(Path::new("/nonexistent/file.toml"), None);
        assert!(result.is_err());
        match result {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("Failed to read file"));
            }
            _ => panic!("Expected ConfigurationError for nonexistent file"),
        }
    }

    #[test]
    fn test_from_file_validation_error() {
        let toml_content = r#"
[metadata]
code = ""
name = "Test"

[terminators]
chars = ["."]

[ellipsis]
patterns = []

[enclosures]
pairs = []

[suppression]

[abbreviations]

[sentence_starters]
common = ["The"]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();

        let result = ConfigurableLanguageRules::from_file(temp_file.path(), None);
        assert!(result.is_err());
        match result {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("Language code is required"));
            }
            _ => panic!("Expected ConfigurationError for validation failure"),
        }
    }
}
