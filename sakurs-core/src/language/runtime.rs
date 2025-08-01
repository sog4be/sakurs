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
    sentence_starters: SentenceStarterTable,
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

        // Build sentence starters table
        let sentence_starters =
            SentenceStarterTable::from_categories(config.sentence_starters.categories.clone());

        Ok(Self {
            code: config.metadata.code.clone(),
            name: config.metadata.name.clone(),
            term_table,
            dot_table,
            enclosures,
            abbv_trie,
            ellipsis,
            suppress,
            sentence_starters,
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

    fn boundary_decision(
        &self,
        text: &str,
        pos: usize,
        term_char: char,
        prev_char: Option<char>,
        next_char: Option<char>,
    ) -> BoundaryDecision {
        if pos == 0 || pos > text.len() {
            return BoundaryDecision::Reject;
        }

        // Check if this is actually a terminator character
        if !self.is_terminator_char(term_char) {
            return BoundaryDecision::Reject;
        }

        // NOTE: Expensive O(n) character lookup eliminated - terminator passed as parameter!

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
            if let Some(prev) = prev_char {
                let pattern = format!("{prev}{term_char}");
                if self.term_table.is_pattern(&pattern) {
                    // This completes a pattern, so it's a boundary
                    return BoundaryDecision::Accept(BoundaryStrength::Strong);
                }
            }

            // Look ahead to see if this starts a pattern
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
            // Need to ensure pos-1 is at a valid UTF-8 boundary
            if pos > 0 && text.is_char_boundary(pos - 1) {
                if self.ellipsis.is_ellipsis_at(text, pos - 1) {
                    is_ellipsis = true;
                }
            }
            
            // If not detected yet, do simple consecutive dots check - most ellipses are "..." (3 dots)
            if !is_ellipsis {
                // Check if we have dots immediately before and after this one
                let has_prev_dot = matches!(prev_char, Some('.'));
                let has_next_dot = matches!(next_char, Some('.'));

                // If surrounded by dots, very likely an ellipsis
                if has_prev_dot && has_next_dot {
                    is_ellipsis = true;
                } else if has_prev_dot || has_next_dot {
                    // Check if we're at the edge of an ellipsis pattern like "..."
                    // This is a simple heuristic - if one neighbor is a dot, assume ellipsis
                    // More sophisticated detection can be added later if needed
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
            // DEPRECATED: Using slow O(n) find_abbrev method - should be replaced with efficient version
            let is_abbrev = self.abbv_trie.find_abbrev(text, pos);

            if is_abbrev {
                // Re-enabled: Check if a sentence starter follows
                if !self.sentence_starters.is_empty() {
                    // Check if a sentence starter follows the abbreviation
                    let has_starter = self.sentence_starters.check_after_abbreviation(text, pos);

                    if has_starter {
                        // Sentence starter after abbreviation = boundary
                        return BoundaryDecision::Accept(BoundaryStrength::Strong);
                    }
                }
                // No sentence starter or none configured = not a boundary
                return BoundaryDecision::Reject;
            }

            // Check for decimal point or IP address context
            // Use the prev_char and next_char parameters to avoid O(n) operations
            if let (Some(p), Some(n)) = (prev_char, next_char) {
                if p.is_ascii_digit() && n.is_ascii_digit() {
                    // This is digit.digit (decimal or IP) - reject as boundary
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

    // --- High-Performance O(1) Methods using CharacterWindow ---

    fn boundary_decision_efficient(
        &self,
        window: &crate::character_window::CharacterWindow,
        _byte_pos: usize,
    ) -> crate::language::interface::BoundaryDecision {
        use crate::language::interface::{BoundaryDecision, BoundaryStrength};


        let term_char = match window.current_char() {
            Some(ch) => ch,
            None => return BoundaryDecision::Reject,
        };

        // Check if this is actually a terminator character
        if !self.is_terminator_char(term_char) {
            return BoundaryDecision::Reject;
        }

        let prev_char = window.prev_char();
        let next_char = window.next_char();

        // Check suppression rules first using window context
        if self.should_suppress_efficient(window) {
            return BoundaryDecision::Reject;
        }

        // Check if this terminator is part of a multi-character pattern
        if self.term_table.has_patterns() && (term_char == '!' || term_char == '?') {
            // Check if we complete a pattern with previous character
            if let Some(prev) = prev_char {
                let pattern = format!("{prev}{term_char}");
                if self.term_table.is_pattern(&pattern) {
                    return BoundaryDecision::Accept(BoundaryStrength::Strong);
                }
            }

            // Check if this starts a pattern with next character
            if let Some(next) = next_char {
                let pattern = format!("{term_char}{next}");
                if self.term_table.is_pattern(&pattern) {
                    return BoundaryDecision::Reject;
                }
            }
        }

        // Special handling for dots
        if term_char == '.' {
            // Check if it's part of ellipsis using character context
            let mut is_ellipsis = false;
            let has_prev_dot = matches!(prev_char, Some('.'));
            let has_next_dot = matches!(next_char, Some('.'));

            if has_prev_dot && has_next_dot {
                is_ellipsis = true;
            } else if has_prev_dot || has_next_dot {
                is_ellipsis = true;
            }

            if is_ellipsis {
                return if self.ellipsis.treat_as_boundary() {
                    BoundaryDecision::Accept(BoundaryStrength::Weak)
                } else {
                    BoundaryDecision::Reject
                };
            }

            // Check if it's an abbreviation using efficient window-based method
            if self.abbv_trie.find_abbrev_efficient(window) {
                // Use heuristic-based sentence boundary detection after abbreviations
                // This avoids O(n²) complexity from text scanning
                
                // 1. End of text = always a boundary
                if next_char.is_none() {
                    return BoundaryDecision::Accept(BoundaryStrength::Strong);
                }
                
                // 2. Check for sentence boundary patterns using O(1) operations
                // Pattern: abbreviation + space + uppercase = likely new sentence
                if let Some(next) = next_char {
                    if next.is_whitespace() {
                        // For abbreviations followed by whitespace, we need better heuristics
                        // to distinguish "Dr. Smith" (name) from "Dr. He arrived" (new sentence)
                        
                        // For now, we'll be conservative and reject boundaries after
                        // abbreviations with whitespace to avoid false positives.
                        // This maintains the original behavior before sentence starters.
                        
                        // A more sophisticated approach would:
                        // 1. Check if the word after space is very short (1-2 chars) and uppercase
                        // 2. Use a small list of common sentence starters
                        // 3. Or use two-pass preprocessing as suggested in the design doc
                        
                        return BoundaryDecision::Reject;
                    } else if next == ',' || next == ';' || next == ':' {
                        // Abbreviations followed by punctuation are NOT sentence boundaries
                        // Examples: "Prof., Dr., St., etc.,"
                        return BoundaryDecision::Reject;
                    } else if next == '.' {
                        // Multi-period abbreviation like U.S.A.
                        // Don't create boundary at internal dots
                        return BoundaryDecision::Reject;
                    } else if next.is_uppercase() && !next.is_alphabetic() {
                        // Special case: abbreviation followed by non-letter uppercase
                        // This might be a new sentence starting with a number or symbol
                        return BoundaryDecision::Accept(BoundaryStrength::Strong);
                    }
                }
                
                // 3. Default: reject boundary for abbreviations
                return BoundaryDecision::Reject;
            }

            // Check for decimal point or IP address context
            if let (Some(p), Some(n)) = (prev_char, next_char) {
                if p.is_ascii_digit() && n.is_ascii_digit() {
                    return BoundaryDecision::Reject;
                }
            }
        }

        // Default: accept as strong boundary
        BoundaryDecision::Accept(BoundaryStrength::Strong)
    }

    fn is_abbreviation_efficient(&self, window: &crate::character_window::CharacterWindow) -> bool {
        // Use our efficient abbreviation trie method
        self.abbv_trie.find_abbrev_efficient(window)
    }

    fn should_suppress_efficient(&self, window: &crate::character_window::CharacterWindow) -> bool {
        let current = window.current_char();
        let prev = window.prev_char();
        let next = window.next_char();

        // Check against suppressor patterns efficiently
        if let Some(ch) = current {
            // Use the Suppresser's efficient pattern matching
            // For now, implement basic patterns directly
            match ch {
                '\'' => {
                    // Apostrophe suppression: check if between letters (contractions)
                    if let (Some(p), Some(n)) = (prev, next) {
                        if p.is_alphabetic() && n.is_alphabetic() {
                            return true;
                        }
                    }
                }
                ')' => {
                    // List item suppression: (1) at line start
                    if let Some(p) = prev {
                        if p.is_ascii_alphanumeric() {
                            // Could be end of list item like "(1)" or "(a)"
                            // This is a simplified check
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }
}
