use super::types::LanguageConfig;
use crate::domain::error::DomainError;
use std::collections::HashMap;
use std::sync::OnceLock;

static LANGUAGE_CONFIGS: OnceLock<HashMap<String, LanguageConfig>> = OnceLock::new();

macro_rules! embed_language_config {
    ($code:expr, $path:expr) => {
        ($code, include_str!($path))
    };
}

fn load_embedded_configs() -> Result<HashMap<String, LanguageConfig>, DomainError> {
    let mut configs = HashMap::new();

    let embedded_configs = [
        embed_language_config!("en", "../../../../configs/languages/english.toml"),
        embed_language_config!("ja", "../../../../configs/languages/japanese.toml"),
    ];

    for (code, toml_content) in embedded_configs {
        let config: LanguageConfig = toml::from_str(toml_content).map_err(|e| {
            DomainError::ConfigurationError(format!("Failed to parse {code} config: {e}"))
        })?;

        // Validate that the config code matches
        if config.metadata.code != code {
            return Err(DomainError::ConfigurationError(format!(
                "Config code mismatch: expected {}, got {}",
                code, config.metadata.code
            )));
        }

        configs.insert(code.to_string(), config);
    }

    Ok(configs)
}

pub fn get_language_config(code: &str) -> Result<&'static LanguageConfig, DomainError> {
    let configs = LANGUAGE_CONFIGS
        .get_or_init(|| load_embedded_configs().expect("Failed to load embedded language configs"));

    configs
        .get(code)
        .ok_or_else(|| DomainError::UnsupportedLanguage(code.to_string()))
}

pub fn list_available_languages() -> Vec<&'static str> {
    let configs = LANGUAGE_CONFIGS
        .get_or_init(|| load_embedded_configs().expect("Failed to load embedded language configs"));

    configs.keys().map(|s| s.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_config_unsupported() {
        match get_language_config("nonexistent") {
            Err(DomainError::UnsupportedLanguage(code)) => {
                assert_eq!(code, "nonexistent");
            }
            _ => panic!("Expected UnsupportedLanguage error"),
        }
    }

    #[test]
    fn test_get_language_config_english() {
        let config = get_language_config("en").expect("English config should exist");
        assert_eq!(config.metadata.code, "en");
        assert_eq!(config.metadata.name, "English");
        assert!(!config.abbreviations.categories.is_empty());
    }

    #[test]
    fn test_get_language_config_japanese() {
        let config = get_language_config("ja").expect("Japanese config should exist");
        assert_eq!(config.metadata.code, "ja");
        assert_eq!(config.metadata.name, "Japanese");
    }

    #[test]
    fn test_list_available_languages() {
        let languages = list_available_languages();
        assert!(languages.contains(&"en"));
        assert!(languages.contains(&"ja"));
        assert_eq!(languages.len(), 2);
    }

    #[test]
    fn test_list_available_languages_sorted() {
        let mut languages = list_available_languages();
        languages.sort();
        assert_eq!(languages, vec!["en", "ja"]);
    }

    #[test]
    fn test_get_language_config_multiple_times() {
        // Test that the static initialization works correctly
        let config1 = get_language_config("en").unwrap();
        let config2 = get_language_config("en").unwrap();
        assert!(std::ptr::eq(config1, config2)); // Same reference
    }

    #[test]
    fn test_config_code_validation() {
        // The embedded configs should have matching codes
        let en_config = get_language_config("en").unwrap();
        assert_eq!(en_config.metadata.code, "en");

        let ja_config = get_language_config("ja").unwrap();
        assert_eq!(ja_config.metadata.code, "ja");
    }
}
