//! Language configuration loader
//!
//! Manages embedded and dynamic language rules with caching.

#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::sync::{Arc, OnceLock};

use crate::language::{interface::LanguageRules, runtime::ConfigurableLanguageRules};

/// Embedded language configurations
#[cfg(feature = "std")]
static EMBEDDED: OnceLock<HashMap<String, Arc<dyn LanguageRules>>> = OnceLock::new();

/// Load language rules by code
#[cfg(feature = "std")]
pub fn get_rules(code: &str) -> Result<Arc<dyn LanguageRules>, String> {
    // Initialize embedded languages on first access
    let embedded = EMBEDDED.get_or_init(|| {
        let mut map = HashMap::new();

        // Load English
        match load_embedded_language("en", include_str!("../../configs/languages/english.toml")) {
            Ok(rules) => {
                map.insert("en".to_string(), rules);
                map.insert("english".to_string(), map["en"].clone());
            }
            Err(e) => {
                eprintln!("Warning: Failed to load English config: {e}");
            }
        }

        // Load Japanese
        match load_embedded_language("ja", include_str!("../../configs/languages/japanese.toml")) {
            Ok(rules) => {
                map.insert("ja".to_string(), rules);
                map.insert("japanese".to_string(), map["ja"].clone());
            }
            Err(e) => {
                eprintln!("Warning: Failed to load Japanese config: {e}");
            }
        }

        map
    });

    // Look up in embedded languages
    embedded
        .get(code)
        .cloned()
        .ok_or_else(|| format!("Unknown language code: {code}"))
}

/// Load embedded language from TOML string
#[cfg(feature = "std")]
fn load_embedded_language(code: &str, toml_str: &str) -> Result<Arc<dyn LanguageRules>, String> {
    let config: crate::language::config::LanguageConfig =
        toml::from_str(toml_str).map_err(|e| format!("Failed to parse {code} config: {e}"))?;

    let rules = ConfigurableLanguageRules::from_config(&config)?;
    Ok(Arc::new(rules))
}

/// Simple rules for testing and no_std environments
#[cfg(not(feature = "std"))]
pub fn get_simple_rules() -> impl LanguageRules {
    use crate::language::interface::{BoundaryDecision, BoundaryStrength, DotRole, EnclosureInfo};

    struct SimpleRules;

    impl LanguageRules for SimpleRules {
        fn is_terminator_char(&self, ch: char) -> bool {
            matches!(ch, '.' | '!' | '?')
        }

        fn enclosure_info(&self, ch: char) -> Option<EnclosureInfo> {
            match ch {
                '(' => Some(EnclosureInfo {
                    type_id: 0,
                    delta: 1,
                    symmetric: false,
                }),
                ')' => Some(EnclosureInfo {
                    type_id: 0,
                    delta: -1,
                    symmetric: false,
                }),
                _ => None,
            }
        }

        fn dot_role(&self, _prev: Option<char>, _next: Option<char>) -> DotRole {
            DotRole::Ordinary
        }

        fn boundary_decision(&self, _text: &str, _pos: usize) -> BoundaryDecision {
            BoundaryDecision::Accept(BoundaryStrength::Strong)
        }
    }

    SimpleRules
}
