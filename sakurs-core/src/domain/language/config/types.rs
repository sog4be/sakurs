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

fn default_true() -> bool {
    true
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
        "#;

        let config: LanguageConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.metadata.code, "en");
        assert_eq!(config.terminators.chars.len(), 3);
        assert_eq!(config.enclosures.pairs.len(), 2);
        assert_eq!(config.abbreviations.categories["titles"].len(), 3);
    }
}
