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
        let patterns: Vec<String> = config
            .terminators
            .patterns
            .iter()
            .map(|p| p.pattern.clone())
            .collect();
        let term_table = TermTable::with_patterns(config.terminators.chars.clone(), patterns);

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

        // Check if this terminator is part of a multi-character pattern
        // For patterns like "!?" or "?!", if we're at the first character,
        // we should check if the next character completes a pattern
        if self.term_table.has_patterns() && (term_char == '!' || term_char == '?') {
            // Check if we're part of a multi-character terminator pattern
            // Look back to see if we complete a pattern
            if pos >= 2 {
                let prev_char = text.chars().nth(text[..pos - 1].chars().count() - 1);
                if let Some(prev) = prev_char {
                    let pattern = format!("{prev}{term_char}");
                    if self.term_table.is_pattern(&pattern) {
                        // This completes a pattern, so it's a boundary
                        return BoundaryDecision::Accept(BoundaryStrength::Strong);
                    }
                }
            }

            // Look ahead to see if this starts a pattern
            let next_char = text.chars().nth(text[..pos].chars().count());
            if let Some(next) = next_char {
                let pattern = format!("{term_char}{next}");
                if self.term_table.is_pattern(&pattern) {
                    // This starts a pattern, so it's not a boundary yet
                    return BoundaryDecision::Reject;
                }
            }
        }

        // Special handling for dots
        if term_char == '.' {
            // Check if it's part of ellipsis
            // We need to check if we're in the middle or at the end of an ellipsis
            // Check both the current position and look ahead/behind for dots
            let mut is_ellipsis = false;

            // First check if an ellipsis pattern ends at our position
            if self.ellipsis.is_ellipsis_at(text, pos - 1) {
                is_ellipsis = true;
            } else {
                // Check if we're in the middle of consecutive dots
                let chars: Vec<char> = text.chars().collect();
                let char_pos = text[..pos].chars().count();

                // Look for dots before and after
                let mut dots_found = 1; // Current dot

                // Count dots before
                let mut i = char_pos.saturating_sub(2); // Start before current
                while i < char_pos - 1 && i < chars.len() && chars[i] == '.' {
                    dots_found += 1;
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                }

                // Count dots after
                if char_pos < chars.len() {
                    let mut j = char_pos;
                    while j < chars.len() && chars[j] == '.' {
                        dots_found += 1;
                        j += 1;
                    }
                }

                // If we have 3 or more consecutive dots, it's an ellipsis
                if dots_found >= 3 {
                    is_ellipsis = true;
                }
            }

            if is_ellipsis {
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

            // Check for decimal point or IP address context
            // pos is the position after the dot, so dot is at pos-1
            let dot_pos = pos - 1;
            if dot_pos > 0 && pos < text.len() {
                // Get char before the dot
                let prev_char = if dot_pos > 0 {
                    text[..dot_pos].chars().last()
                } else {
                    None
                };
                // Get char after the dot (at pos)
                let next_char = text[pos..].chars().next();

                // Check for number.number pattern (decimal or IP)
                if let (Some(p), Some(n)) = (prev_char, next_char) {
                    if p.is_ascii_digit() && n.is_ascii_digit() {
                        // This is digit.digit - reject as boundary
                        return BoundaryDecision::Reject;
                    }
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
