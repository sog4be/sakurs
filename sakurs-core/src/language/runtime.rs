//! Runtime implementation of language rules
//!
//! This module provides the concrete implementation that bridges
//! configuration and the hot-path trait interface.

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::language::{
    config::LanguageConfig,
    interface::{BoundaryDecision, BoundaryStrength, DotRole, EnclosureInfo, LanguageRules},
    tables::*,
};

/// Configurable language rules implementation
#[derive(Debug, Clone)]
pub struct ConfigurableLanguageRules {
    /// Language metadata  
    #[allow(dead_code)]
    code: String,
    #[allow(dead_code)]
    name: String,

    /// Runtime tables
    term_table: TermTable,
    dot_table: DotTable,
    enclosures: EncTable,
    abbv_trie: Trie,
    ellipsis: EllipsisSet,
    suppress: Suppresser,
}

impl ConfigurableLanguageRules {
    /// Create from configuration
    pub fn from_config(config: &LanguageConfig) -> Result<Self, String> {
        // Validate configuration
        config.validate()?;
        // Build terminator table
        let term_table = TermTable::new(config.terminators.chars.clone());

        // Build dot table
        let dot_table = DotTable::new(config.ellipsis.patterns.clone());

        // Build enclosure table
        let enclosures = EncTable::new(
            config
                .enclosures
                .pairs
                .iter()
                .map(|p| (p.open, p.close, p.symmetric))
                .collect(),
        );

        // Build abbreviation trie
        let abbv_trie = Trie::from_categories(
            config.abbreviations.categories.clone(),
            false, // Case insensitive
        );

        // Build ellipsis set
        let ellipsis = EllipsisSet::new(
            config.ellipsis.patterns.clone(),
            config.ellipsis.treat_as_boundary,
        );

        // Build suppressor
        let suppress = Suppresser::new(
            config
                .suppression
                .fast_patterns
                .iter()
                .map(|p| (p.char, p.line_start, p.before.clone(), p.after.clone()))
                .collect(),
        );

        Ok(Self {
            code: config.metadata.code.clone(),
            name: config.metadata.name.clone(),
            term_table,
            dot_table,
            enclosures,
            abbv_trie,
            ellipsis,
            suppress,
        })
    }
}

impl LanguageRules for ConfigurableLanguageRules {
    #[inline]
    fn is_terminator_char(&self, ch: char) -> bool {
        self.term_table.is_terminator(ch)
    }

    #[inline]
    fn enclosure_info(&self, ch: char) -> Option<EnclosureInfo> {
        self.enclosures.get(ch)
    }

    #[inline]
    fn dot_role(&self, prev: Option<char>, next: Option<char>) -> DotRole {
        // First check dot table for special patterns
        // If ordinary dot, it might still be abbreviation (checked later)
        self.dot_table.classify(prev, next)
    }

    fn boundary_decision(&self, text: &str, pos: usize) -> BoundaryDecision {
        if pos == 0 || pos > text.len() {
            return BoundaryDecision::Reject;
        }

        // Get the terminator character
        let term_char = text.chars().nth(text[..pos].chars().count() - 1);
        if term_char.is_none() {
            return BoundaryDecision::Reject;
        }
        let term_char = term_char.unwrap();

        // Check suppression rules first
        if self.suppress.should_suppress(text, pos) {
            return BoundaryDecision::Reject;
        }

        // Special handling for dots
        if term_char == '.' {
            // Check if it's part of ellipsis
            if self.ellipsis.is_ellipsis_at(text, pos - 1) {
                return if self.ellipsis.treat_as_boundary() {
                    BoundaryDecision::Accept(BoundaryStrength::Weak)
                } else {
                    BoundaryDecision::Reject
                };
            }

            // Check if it's an abbreviation
            if self.abbv_trie.find_abbrev(text, pos - 1) {
                return BoundaryDecision::Reject;
            }

            // Check for decimal point context
            let chars: Vec<char> = text.chars().collect();
            let char_pos = text[..pos].chars().count();
            if char_pos > 1 && char_pos < chars.len() {
                let prev = chars.get(char_pos - 2).copied();
                let next = chars.get(char_pos).copied();
                if let DotRole::DecimalDot = self.dot_table.classify(prev, next) {
                    return BoundaryDecision::Reject;
                }
            }
        }

        // Default: accept as strong boundary
        BoundaryDecision::Accept(BoundaryStrength::Strong)
    }

    fn max_enclosure_pairs(&self) -> usize {
        self.enclosures.max_type_id() as usize + 1
    }
}
