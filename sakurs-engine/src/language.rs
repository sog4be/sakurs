//! Language rules implementation for the engine

use sakurs_delta_core::{Class, LanguageRules};
use std::sync::Arc;

/// English language rules implementation
#[derive(Debug, Clone)]
pub struct EnglishRules;

impl LanguageRules for EnglishRules {
    fn classify_char(&self, ch: char) -> Class {
        Class::from_char(ch)
    }

    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool {
        // Simple abbreviation detection
        if dot_pos == 0 {
            return false;
        }

        // Check common patterns
        let before = &text[..dot_pos];
        matches!(
            before,
            "Dr" | "Mr"
                | "Mrs"
                | "Ms"
                | "Prof"
                | "Sr"
                | "Jr"
                | "Inc"
                | "Ltd"
                | "Co"
                | "Corp"
                | "vs"
                | "etc"
                | "i.e"
                | "e.g"
        )
    }

    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        match ch {
            '(' => Some((0, true)),
            ')' => Some((0, false)),
            '[' => Some((1, true)),
            ']' => Some((1, false)),
            '{' => Some((2, true)),
            '}' => Some((2, false)),
            '"' => Some((3, true)),  // Simplified: always opening
            '\'' => Some((4, true)), // Simplified: always opening
            _ => None,
        }
    }

    fn max_enclosure_pairs(&self) -> usize {
        5
    }
}

/// Japanese language rules implementation
#[derive(Debug, Clone)]
pub struct JapaneseRules;

impl LanguageRules for JapaneseRules {
    fn classify_char(&self, ch: char) -> Class {
        match ch {
            '。' | '！' | '？' => Class::Terminator,
            '「' | '『' | '（' | '【' => Class::Open,
            '」' | '』' | '）' | '】' => Class::Close,
            _ => Class::from_char(ch),
        }
    }

    fn is_abbreviation(&self, _text: &str, _dot_pos: usize) -> bool {
        // Japanese uses different punctuation, less abbreviation handling needed
        false
    }

    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        match ch {
            '「' => Some((0, true)),
            '」' => Some((0, false)),
            '『' => Some((1, true)),
            '』' => Some((1, false)),
            '（' => Some((2, true)),
            '）' => Some((2, false)),
            '【' => Some((3, true)),
            '】' => Some((3, false)),
            _ => None,
        }
    }

    fn is_terminator(&self, ch: char) -> bool {
        matches!(ch, '。' | '！' | '？' | '.' | '!' | '?')
    }

    fn max_enclosure_pairs(&self) -> usize {
        4
    }
}

/// Concrete enum for all supported language rules
#[derive(Debug, Clone)]
pub enum LanguageRulesImpl {
    /// English language rules
    English(EnglishRules),
    /// Japanese language rules
    Japanese(JapaneseRules),
}

impl LanguageRules for LanguageRulesImpl {
    fn classify_char(&self, ch: char) -> Class {
        match self {
            LanguageRulesImpl::English(rules) => rules.classify_char(ch),
            LanguageRulesImpl::Japanese(rules) => rules.classify_char(ch),
        }
    }

    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool {
        match self {
            LanguageRulesImpl::English(rules) => rules.is_abbreviation(text, dot_pos),
            LanguageRulesImpl::Japanese(rules) => rules.is_abbreviation(text, dot_pos),
        }
    }

    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        match self {
            LanguageRulesImpl::English(rules) => rules.get_enclosure_pair(ch),
            LanguageRulesImpl::Japanese(rules) => rules.get_enclosure_pair(ch),
        }
    }

    fn is_terminator(&self, ch: char) -> bool {
        match self {
            LanguageRulesImpl::English(rules) => rules.is_terminator(ch),
            LanguageRulesImpl::Japanese(rules) => rules.is_terminator(ch),
        }
    }

    fn max_enclosure_pairs(&self) -> usize {
        match self {
            LanguageRulesImpl::English(rules) => rules.max_enclosure_pairs(),
            LanguageRulesImpl::Japanese(rules) => rules.max_enclosure_pairs(),
        }
    }
}

/// Get language rules by name
pub fn get_language_rules(language: &str) -> Option<LanguageRulesImpl> {
    match language.to_lowercase().as_str() {
        "en" | "english" => Some(LanguageRulesImpl::English(EnglishRules)),
        "ja" | "japanese" => Some(LanguageRulesImpl::Japanese(JapaneseRules)),
        _ => None,
    }
}

/// Get language rules as trait object (for compatibility)
pub fn get_language_rules_dyn(language: &str) -> Option<Arc<dyn LanguageRules>> {
    get_language_rules(language).map(|rules| Arc::new(rules) as Arc<dyn LanguageRules>)
}
