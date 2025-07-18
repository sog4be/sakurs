//! Language source management for CLI

use crate::commands::process::Language;
use std::path::PathBuf;

/// Source of language rules
#[derive(Debug, Clone)]
pub enum LanguageSource {
    /// Built-in language (existing behavior)
    BuiltIn(Language),
    /// External configuration file
    External {
        /// Path to the configuration file
        path: PathBuf,
        /// Optional language code override
        language_code: Option<String>,
    },
}

impl LanguageSource {
    /// Get the display name for the language source
    pub fn display_name(&self) -> String {
        match self {
            LanguageSource::BuiltIn(lang) => format!("Built-in: {}", lang.as_str()),
            LanguageSource::External {
                path,
                language_code,
            } => {
                if let Some(code) = language_code {
                    format!("External: {} (code: {})", path.display(), code)
                } else {
                    format!("External: {}", path.display())
                }
            }
        }
    }
}

impl Language {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Japanese => "Japanese",
        }
    }

    /// Get language code
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Japanese => "ja",
        }
    }
}
