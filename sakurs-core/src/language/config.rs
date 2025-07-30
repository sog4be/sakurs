//! Configuration structures and validation
//!
//! This module defines the TOML schema for language configuration.

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::collections::HashMap;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Root language configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub metadata: Metadata,
    pub terminators: Terminators,
    pub ellipsis: Ellipsis,
    pub enclosures: Enclosures,
    pub suppression: Suppression,
    pub abbreviations: Abbreviations,
    #[serde(default)]
    pub sentence_starters: SentenceStarters,
}

/// Language metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub code: String,
    pub name: String,
}

/// Terminator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Terminators {
    pub chars: Vec<char>,
    #[serde(default)]
    pub patterns: Vec<TerminatorPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminatorPattern {
    pub pattern: String,
    pub name: String,
}

/// Ellipsis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ellipsis {
    #[serde(default = "default_true")]
    pub treat_as_boundary: bool,
    pub patterns: Vec<String>,
    #[serde(default)]
    pub context_rules: Vec<ContextRule>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRule {
    pub condition: String,
    pub boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exception {
    pub regex: String,
    pub boundary: bool,
}

/// Enclosure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enclosures {
    pub pairs: Vec<EnclosurePair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclosurePair {
    pub open: char,
    pub close: char,
    #[serde(default)]
    pub symmetric: bool,
}

/// Suppression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suppression {
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
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    pub pattern: String,
    pub description: String,
}

/// Abbreviation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Abbreviations {
    #[serde(flatten)]
    pub categories: HashMap<String, Vec<String>>,
}

/// Sentence starters configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentenceStarters {
    /// Whether to require following space (optional)
    #[serde(default)]
    pub require_following_space: bool,
    /// Minimum word length to consider (optional)
    #[serde(default)]
    pub min_word_length: usize,
    /// Categories of sentence starters
    #[serde(flatten)]
    pub categories: HashMap<String, Vec<String>>,
}

fn default_true() -> bool {
    true
}

impl LanguageConfig {
    /// Validate configuration
    pub(crate) fn validate(&self) -> Result<(), String> {
        // Check enclosure pairs limit
        if self.enclosures.pairs.len() > 255 {
            return Err("Too many enclosure pairs (max 255)".to_string());
        }

        // Check terminator chars not empty
        if self.terminators.chars.is_empty() {
            return Err("No terminator characters defined".to_string());
        }

        Ok(())
    }
}
