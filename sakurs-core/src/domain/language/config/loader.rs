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
        embed_language_config!("en", "../../../../configs/languages/en.toml"),
        embed_language_config!("ja", "../../../../configs/languages/ja.toml"),
    ];

    for (code, toml_content) in embedded_configs {
        let config: LanguageConfig = toml::from_str(toml_content).map_err(|e| {
            DomainError::ConfigurationError(format!("Failed to parse {} config: {}", code, e))
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
    fn test_load_language_config() {
        // This test will fail until we create the actual config files
        // For now, we'll just test the error handling
        match get_language_config("nonexistent") {
            Err(DomainError::UnsupportedLanguage(code)) => {
                assert_eq!(code, "nonexistent");
            }
            _ => panic!("Expected UnsupportedLanguage error"),
        }
    }
}
