//! Language rules implementation for the engine
//!
//! This module provides language rules by delegating to the core's
//! configuration-based implementation.

use sakurs_core::LanguageRules;
use std::sync::Arc;

/// Get language rules by language code (backward compatibility)
pub fn get_language_rules(language: &str) -> Option<LanguageRulesImpl> {
    match LanguageRulesImpl::from_language(language) {
        Ok(rules) => Some(rules),
        Err(e) => {
            eprintln!("Failed to get language rules for '{language}': {e}");
            None
        }
    }
}

/// Get language rules by language code
pub fn get_rules(language: &str) -> Result<Arc<dyn LanguageRules>, String> {
    #[cfg(feature = "std")]
    {
        // Use the loader from core
        sakurs_core::language::get_rules(language)
    }

    #[cfg(not(feature = "std"))]
    {
        // For no_std, we can only provide simple built-in rules
        match language {
            "en" | "english" => Ok(Arc::new(SimpleEnglishRules)),
            "ja" | "japanese" => Ok(Arc::new(SimpleJapaneseRules)),
            _ => Err(format!("Unsupported language: {}", language)),
        }
    }
}

/// Language rules implementation enum
#[derive(Clone)]
pub enum LanguageRulesImpl {
    /// Dynamic rules loaded from core
    Dynamic(Arc<dyn LanguageRules>),
}

impl std::fmt::Debug for LanguageRulesImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageRulesImpl::Dynamic(_) => write!(f, "LanguageRulesImpl::Dynamic(...)"),
        }
    }
}

impl LanguageRulesImpl {
    /// Create from language code
    pub fn from_language(language: &str) -> Result<Self, String> {
        let rules = get_rules(language)?;
        Ok(LanguageRulesImpl::Dynamic(rules))
    }
}

impl LanguageRules for LanguageRulesImpl {
    fn is_terminator_char(&self, ch: char) -> bool {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.is_terminator_char(ch),
        }
    }

    fn enclosure_info(&self, ch: char) -> Option<sakurs_core::language::EnclosureInfo> {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.enclosure_info(ch),
        }
    }

    fn dot_role(&self, prev: Option<char>, next: Option<char>) -> sakurs_core::language::DotRole {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.dot_role(prev, next),
        }
    }

    fn boundary_decision(
        &self,
        text: &str,
        pos: usize,
        terminator_char: char,
        prev_char: Option<char>,
        next_char: Option<char>,
    ) -> sakurs_core::language::BoundaryDecision {
        match self {
            LanguageRulesImpl::Dynamic(rules) => {
                rules.boundary_decision(text, pos, terminator_char, prev_char, next_char)
            }
        }
    }

    #[inline]
    fn classify_char(&self, ch: char) -> sakurs_core::Class {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.classify_char(ch),
        }
    }

    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.is_abbreviation(text, dot_pos),
        }
    }

    fn abbrev_match(&self, abbrev: &str) -> bool {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.abbrev_match(abbrev),
        }
    }

    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.get_enclosure_pair(ch),
        }
    }

    fn pair_id(&self, ch: char) -> Option<u8> {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.pair_id(ch),
        }
    }

    fn is_terminator(&self, ch: char) -> bool {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.is_terminator(ch),
        }
    }

    fn max_enclosure_pairs(&self) -> usize {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.max_enclosure_pairs(),
        }
    }

    // --- High-Performance O(1) Methods using CharacterWindow ---

    fn boundary_decision_efficient(
        &self,
        window: &sakurs_core::character_window::CharacterWindow,
        byte_pos: usize,
    ) -> sakurs_core::language::BoundaryDecision {
        match self {
            LanguageRulesImpl::Dynamic(rules) => {
                rules.boundary_decision_efficient(window, byte_pos)
            }
        }
    }

    fn is_abbreviation_efficient(
        &self,
        window: &sakurs_core::character_window::CharacterWindow,
    ) -> bool {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.is_abbreviation_efficient(window),
        }
    }

    fn should_suppress_efficient(
        &self,
        window: &sakurs_core::character_window::CharacterWindow,
    ) -> bool {
        match self {
            LanguageRulesImpl::Dynamic(rules) => rules.should_suppress_efficient(window),
        }
    }
}

// Simple fallback rules for no_std environments
#[cfg(not(feature = "std"))]
struct SimpleEnglishRules;

#[cfg(not(feature = "std"))]
impl LanguageRules for SimpleEnglishRules {
    fn is_terminator_char(&self, ch: char) -> bool {
        matches!(ch, '.' | '!' | '?')
    }

    fn enclosure_info(&self, ch: char) -> Option<sakurs_core::language::EnclosureInfo> {
        match ch {
            '(' => Some(sakurs_core::language::EnclosureInfo {
                type_id: 0,
                delta: 1,
                symmetric: false,
            }),
            ')' => Some(sakurs_core::language::EnclosureInfo {
                type_id: 0,
                delta: -1,
                symmetric: false,
            }),
            _ => None,
        }
    }

    fn dot_role(&self, _prev: Option<char>, _next: Option<char>) -> sakurs_core::language::DotRole {
        sakurs_core::language::DotRole::Ordinary
    }

    fn boundary_decision(
        &self,
        text: &str,
        pos: usize,
        terminator_char: char,
        prev_char: Option<char>,
        next_char: Option<char>,
    ) -> sakurs_core::language::BoundaryDecision {
        use sakurs_core::language::{BoundaryDecision, BoundaryStrength};

        if pos > 0 && pos <= text.len() {
            if terminator_char == '.' {
                // Check simple abbreviations
                if pos >= 3 {
                    let start = pos.saturating_sub(3);
                    let before = &text[start..pos - 1];
                    if matches!(before, "Dr" | "Mr") {
                        return BoundaryDecision::Reject;
                    }
                }
            }
            BoundaryDecision::Accept(BoundaryStrength::Strong)
        } else {
            BoundaryDecision::Reject
        }
    }

    fn classify_char(&self, ch: char) -> sakurs_core::Class {
        sakurs_core::Class::from_char(ch)
    }

    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool {
        if dot_pos == 0 {
            return false;
        }
        let before = &text[..dot_pos];
        matches!(before, "Dr" | "Mr" | "Mrs" | "Ms")
    }

    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        match ch {
            '(' => Some((0, true)),
            ')' => Some((0, false)),
            _ => None,
        }
    }

    fn is_terminator(&self, ch: char) -> bool {
        matches!(ch, '.' | '!' | '?')
    }

    fn max_enclosure_pairs(&self) -> usize {
        1
    }

    // Simple implementations for efficient methods
    fn boundary_decision_efficient(
        &self,
        window: &sakurs_core::character_window::CharacterWindow,
        byte_pos: usize,
    ) -> sakurs_core::language::BoundaryDecision {
        // Use the character window for efficient processing
        let terminator_char = window.current_char().unwrap_or('.');
        let prev_char = window.prev_char();
        let next_char = window.next_char();

        use sakurs_core::language::{BoundaryDecision, BoundaryStrength};

        if terminator_char == '.' {
            // Simple abbreviation check using window
            if let Some(prev) = prev_char {
                if prev == 'r' {
                    if let Some(prev_prev) = window.prev_prev_char() {
                        if prev_prev == 'D' {
                            return BoundaryDecision::Reject; // "Dr."
                        }
                        if prev_prev == 'M' {
                            return BoundaryDecision::Reject; // "Mr."
                        }
                    }
                }
            }
        }
        BoundaryDecision::Accept(BoundaryStrength::Strong)
    }

    fn is_abbreviation_efficient(
        &self,
        window: &sakurs_core::character_window::CharacterWindow,
    ) -> bool {
        // Simple check using character window
        if window.current_char() != Some('.') {
            return false;
        }
        if let Some(prev) = window.prev_char() {
            if prev == 'r' {
                if let Some(prev_prev) = window.prev_prev_char() {
                    return matches!(prev_prev, 'D' | 'M');
                }
            }
        }
        false
    }

    fn should_suppress_efficient(
        &self,
        _window: &sakurs_core::character_window::CharacterWindow,
    ) -> bool {
        false
    }
}

#[cfg(not(feature = "std"))]
struct SimpleJapaneseRules;

#[cfg(not(feature = "std"))]
impl LanguageRules for SimpleJapaneseRules {
    fn is_terminator_char(&self, ch: char) -> bool {
        matches!(ch, '。' | '！' | '？' | '.' | '!' | '?')
    }

    fn enclosure_info(&self, ch: char) -> Option<sakurs_core::language::EnclosureInfo> {
        match ch {
            '「' => Some(sakurs_core::language::EnclosureInfo {
                type_id: 0,
                delta: 1,
                symmetric: false,
            }),
            '」' => Some(sakurs_core::language::EnclosureInfo {
                type_id: 0,
                delta: -1,
                symmetric: false,
            }),
            '『' => Some(sakurs_core::language::EnclosureInfo {
                type_id: 1,
                delta: 1,
                symmetric: false,
            }),
            '』' => Some(sakurs_core::language::EnclosureInfo {
                type_id: 1,
                delta: -1,
                symmetric: false,
            }),
            _ => None,
        }
    }

    fn dot_role(&self, _prev: Option<char>, _next: Option<char>) -> sakurs_core::language::DotRole {
        sakurs_core::language::DotRole::Ordinary
    }

    fn boundary_decision(
        &self,
        _text: &str,
        pos: usize,
        terminator_char: char,
        prev_char: Option<char>,
        next_char: Option<char>,
    ) -> sakurs_core::language::BoundaryDecision {
        use sakurs_core::language::{BoundaryDecision, BoundaryStrength};

        if pos > 0 {
            BoundaryDecision::Accept(BoundaryStrength::Strong)
        } else {
            BoundaryDecision::Reject
        }
    }

    fn classify_char(&self, ch: char) -> sakurs_core::Class {
        match ch {
            '。' | '！' | '？' => sakurs_core::Class::Terminator,
            '「' | '『' => sakurs_core::Class::Open,
            '」' | '』' => sakurs_core::Class::Close,
            _ => sakurs_core::Class::from_char(ch),
        }
    }

    fn is_abbreviation(&self, _text: &str, _dot_pos: usize) -> bool {
        false
    }

    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        match ch {
            '「' => Some((0, true)),
            '」' => Some((0, false)),
            '『' => Some((1, true)),
            '』' => Some((1, false)),
            _ => None,
        }
    }

    fn is_terminator(&self, ch: char) -> bool {
        matches!(ch, '。' | '！' | '？' | '.' | '!' | '?')
    }

    fn max_enclosure_pairs(&self) -> usize {
        2
    }

    // Simple implementations for efficient methods
    fn boundary_decision_efficient(
        &self,
        window: &sakurs_core::character_window::CharacterWindow,
        _byte_pos: usize,
    ) -> sakurs_core::language::BoundaryDecision {
        use sakurs_core::language::{BoundaryDecision, BoundaryStrength};

        if window.current_char().is_some() {
            BoundaryDecision::Accept(BoundaryStrength::Strong)
        } else {
            BoundaryDecision::Reject
        }
    }

    fn is_abbreviation_efficient(
        &self,
        _window: &sakurs_core::character_window::CharacterWindow,
    ) -> bool {
        false // Japanese rarely uses abbreviations with dots
    }

    fn should_suppress_efficient(
        &self,
        _window: &sakurs_core::character_window::CharacterWindow,
    ) -> bool {
        false
    }
}
