use crate::domain::error::DomainError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub metadata: MetadataConfig,
    pub terminators: TerminatorConfig,
    pub ellipsis: EllipsisConfig,
    pub enclosures: EnclosureConfig,
    pub suppression: SuppressionConfig,
    pub abbreviations: AbbreviationConfig,
    pub sentence_starters: SentenceStarterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminatorConfig {
    pub chars: Vec<char>,
    #[serde(default)]
    pub patterns: Vec<TerminatorPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminatorPattern {
    pub pattern: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EllipsisConfig {
    #[serde(default = "default_true")]
    pub treat_as_boundary: bool,
    pub patterns: Vec<String>,
    #[serde(default)]
    pub context_rules: Vec<ContextRule>,
    #[serde(default)]
    pub exceptions: Vec<ExceptionPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRule {
    pub condition: String,
    pub boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionPattern {
    pub regex: String,
    pub boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclosureConfig {
    pub pairs: Vec<EnclosurePair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclosurePair {
    pub open: char,
    pub close: char,
    #[serde(default)]
    pub symmetric: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionConfig {
    #[serde(default)]
    pub fast_patterns: Vec<FastPattern>,
    #[serde(default)]
    pub regex_patterns: Vec<RegexPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPattern {
    pub char: char,
    #[serde(default)]
    pub line_start: bool,
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    pub pattern: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbbreviationConfig {
    #[serde(flatten)]
    pub categories: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceStarterConfig {
    /// Categories of sentence starter words
    #[serde(flatten)]
    pub categories: HashMap<String, Vec<String>>,

    /// Whether to use case-sensitive matching (default: false)
    #[serde(default)]
    pub case_sensitive: bool,

    /// Whether to treat any uppercase-starting word as a sentence starter (default: false)
    /// When false, only words in the configured lists are considered sentence starters.
    /// This prevents false positives with proper nouns like "Smith" after abbreviations.
    #[serde(default)]
    pub use_uppercase_fallback: bool,

    /// Minimum word length to consider (default: 1)
    #[serde(default = "default_one")]
    pub min_word_length: usize,
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

impl LanguageConfig {
    /// Validate the language configuration
    pub fn validate(&self) -> Result<(), DomainError> {
        // Validate metadata
        if self.metadata.code.is_empty() {
            return Err(DomainError::ConfigurationError(
                "Language code is required".to_string(),
            ));
        }

        if self.metadata.name.is_empty() {
            return Err(DomainError::ConfigurationError(
                "Language name is required".to_string(),
            ));
        }

        // Validate terminators
        if self.terminators.chars.is_empty() {
            return Err(DomainError::ConfigurationError(
                "At least one terminator character is required".to_string(),
            ));
        }

        // Validate regex patterns in suppression rules
        for pattern in &self.suppression.regex_patterns {
            regex::Regex::new(&pattern.pattern).map_err(|e| {
                DomainError::ConfigurationError(format!(
                    "Invalid regex pattern '{}': {}",
                    pattern.pattern, e
                ))
            })?;
        }

        // Validate regex patterns in ellipsis exceptions
        for exception in &self.ellipsis.exceptions {
            regex::Regex::new(&exception.regex).map_err(|e| {
                DomainError::ConfigurationError(format!(
                    "Invalid regex pattern in ellipsis exception '{}': {}",
                    exception.regex, e
                ))
            })?;
        }

        // Validate abbreviation categories are not empty
        for (category, abbreviations) in &self.abbreviations.categories {
            if abbreviations.is_empty() {
                return Err(DomainError::ConfigurationError(format!(
                    "Abbreviation category '{category}' cannot be empty"
                )));
            }
        }

        // It's OK to have no abbreviation categories at all

        // Validate enclosure pairs
        if self.enclosures.pairs.is_empty() {
            // It's OK to have no enclosure pairs
        }

        // Validate ellipsis patterns
        if self.ellipsis.patterns.is_empty() {
            // It's OK to have no ellipsis patterns
        }

        // Validate sentence starters
        if self.sentence_starters.categories.is_empty() {
            return Err(DomainError::ConfigurationError(
                "At least one sentence starter category is required".to_string(),
            ));
        }

        // Validate each category has at least one word
        for (category, words) in &self.sentence_starters.categories {
            if words.is_empty() {
                return Err(DomainError::ConfigurationError(format!(
                    "Sentence starter category '{category}' cannot be empty"
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_config_deserialize() {
        let toml_str = r#"
            [metadata]
            code = "en"
            name = "English"

            [terminators]
            chars = [".", "!", "?"]
            patterns = [
                { pattern = "!?", name = "surprised_question" }
            ]

            [ellipsis]
            treat_as_boundary = true
            patterns = ["...", "…"]

            [enclosures]
            pairs = [
                { open = "(", close = ")" },
                { open = '"', close = '"', symmetric = true }
            ]

            [suppression]
            fast_patterns = [
                { char = "'", before = "alpha", after = "alpha" }
            ]

            [abbreviations]
            titles = ["Dr", "Mr", "Mrs"]
            common = ["etc", "vs"]

            [sentence_starters]
            pronouns = ["I", "You", "He"]
            articles = ["The", "A", "An"]
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.metadata.code, "en");
        assert_eq!(config.terminators.chars.len(), 3);
        assert_eq!(config.enclosures.pairs.len(), 2);
        assert_eq!(config.abbreviations.categories["titles"].len(), 3);
        assert_eq!(config.sentence_starters.categories["pronouns"].len(), 3);
        assert_eq!(config.sentence_starters.categories["articles"].len(), 3);
    }

    #[test]
    fn test_language_config_validate_success() {
        let toml_str = r#"
            [metadata]
            code = "test"
            name = "Test Language"

            [terminators]
            chars = ["."]

            [ellipsis]
            patterns = []

            [enclosures]
            pairs = []

            [suppression]

            [abbreviations]

            [sentence_starters]
            common = ["The", "A"]
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_language_config_validate_empty_code() {
        let toml_str = r#"
            [metadata]
            code = ""
            name = "Test Language"

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

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        match config.validate() {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("Language code is required"));
            }
            _ => panic!("Expected ConfigurationError for empty language code"),
        }
    }

    #[test]
    fn test_language_config_validate_empty_terminators() {
        let toml_str = r#"
            [metadata]
            code = "test"
            name = "Test Language"

            [terminators]
            chars = []

            [ellipsis]
            patterns = []

            [enclosures]
            pairs = []

            [suppression]

            [abbreviations]

            [sentence_starters]
            common = ["The"]
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        match config.validate() {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("At least one terminator character is required"));
            }
            _ => panic!("Expected ConfigurationError for empty terminators"),
        }
    }

    #[test]
    fn test_language_config_validate_invalid_regex() {
        let toml_str = r#"
            [metadata]
            code = "test"
            name = "Test Language"

            [terminators]
            chars = ["."]

            [ellipsis]
            patterns = []

            [enclosures]
            pairs = []

            [suppression]
            regex_patterns = [
                { pattern = "\\w+[", description = "Invalid regex" }
            ]

            [abbreviations]

            [sentence_starters]
            common = ["The"]
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        match config.validate() {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("Invalid regex pattern"));
                assert!(msg.contains("\\w+["));
            }
            _ => panic!("Expected ConfigurationError for invalid regex"),
        }
    }

    #[test]
    fn test_language_config_validate_empty_abbreviation_category() {
        let toml_str = r#"
            [metadata]
            code = "test"
            name = "Test Language"

            [terminators]
            chars = ["."]

            [ellipsis]
            patterns = []

            [enclosures]
            pairs = []

            [suppression]

            [abbreviations]
            titles = []

            [sentence_starters]
            common = ["The"]
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        match config.validate() {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("Abbreviation category 'titles' cannot be empty"));
            }
            _ => panic!("Expected ConfigurationError for empty abbreviation category"),
        }
    }

    #[test]
    fn test_language_config_validate_empty_sentence_starters() {
        let toml_str = r#"
            [metadata]
            code = "test"
            name = "Test Language"

            [terminators]
            chars = ["."]

            [ellipsis]
            patterns = []

            [enclosures]
            pairs = []

            [suppression]

            [abbreviations]

            [sentence_starters]
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        match config.validate() {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("At least one sentence starter category is required"));
            }
            _ => panic!("Expected ConfigurationError for empty sentence starters"),
        }
    }

    #[test]
    fn test_language_config_validate_empty_sentence_starter_category() {
        let toml_str = r#"
            [metadata]
            code = "test"
            name = "Test Language"

            [terminators]
            chars = ["."]

            [ellipsis]
            patterns = []

            [enclosures]
            pairs = []

            [suppression]

            [abbreviations]

            [sentence_starters]
            common = []
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        match config.validate() {
            Err(DomainError::ConfigurationError(msg)) => {
                assert!(msg.contains("Sentence starter category 'common' cannot be empty"));
            }
            _ => panic!("Expected ConfigurationError for empty sentence starter category"),
        }
    }
}
