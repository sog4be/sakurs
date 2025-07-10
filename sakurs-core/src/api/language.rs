//! Language type for the API

use crate::api::Error;
use std::fmt;
use std::str::FromStr;

/// Supported languages for sentence segmentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    /// English language with standard English rules
    #[default]
    English,
    /// Japanese language with specific punctuation rules
    Japanese,
}

impl Language {
    /// Create a Language from a language code
    pub fn from_code(code: &str) -> Self {
        match code.trim().to_lowercase().as_str() {
            "en" | "eng" | "english" => Language::English,
            "ja" | "jpn" | "japanese" => Language::Japanese,
            _ => Language::English, // Default to English
        }
    }

    /// Get the language code
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Japanese => "ja",
        }
    }

    /// Get the full language name
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Japanese => "Japanese",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "en" | "eng" | "english" => Ok(Language::English),
            "ja" | "jpn" | "japanese" => Ok(Language::Japanese),
            _ => Err(Error::InvalidLanguage(format!("Unsupported language: {s}"))),
        }
    }
}
