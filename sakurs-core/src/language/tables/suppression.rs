//! Suppression rules for special patterns
//!
//! Handles contractions, possessives, and other special cases.

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Fast pattern matcher for suppression rules
#[derive(Debug, Clone)]
pub struct Suppresser {
    /// Fast ASCII patterns (e.g., apostrophes in contractions)
    patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
struct Pattern {
    /// The character to match
    char: char,
    /// Must be at line start
    line_start: bool,
    /// Required before context (alpha, alnum, etc.)
    before: Option<CharClass>,
    /// Required after context
    after: Option<CharClass>,
}

#[derive(Debug, Clone, PartialEq)]
enum CharClass {
    Alpha,
    Alnum,
    Digit,
}

impl CharClass {
    fn matches(&self, ch: char) -> bool {
        match self {
            CharClass::Alpha => ch.is_alphabetic(),
            CharClass::Alnum => ch.is_alphanumeric(),
            CharClass::Digit => ch.is_numeric(),
        }
    }
}

impl Suppresser {
    /// Create with fast patterns only
    pub fn new(patterns: Vec<(char, bool, Option<String>, Option<String>)>) -> Self {
        let patterns = patterns
            .into_iter()
            .map(|(ch, line_start, before, after)| Pattern {
                char: ch,
                line_start,
                before: before.and_then(|s| match s.as_str() {
                    "alpha" => Some(CharClass::Alpha),
                    "alnum" => Some(CharClass::Alnum),
                    "digit" => Some(CharClass::Digit),
                    _ => None,
                }),
                after: after.and_then(|s| match s.as_str() {
                    "alpha" => Some(CharClass::Alpha),
                    "alnum" => Some(CharClass::Alnum),
                    "digit" => Some(CharClass::Digit),
                    _ => None,
                }),
            })
            .collect();

        Self { patterns }
    }

    /// Check if boundary should be suppressed at position
    pub fn should_suppress(&self, text: &str, pos: usize) -> bool {
        if pos == 0 || pos >= text.len() {
            return false;
        }

        // Get the character at pos-1 efficiently using byte slicing
        let ch = if let Some(c) = text[..pos].chars().last() {
            c
        } else {
            return false;
        };

        // Check fast patterns
        for pattern in &self.patterns {
            if pattern.char != ch {
                continue;
            }

            // Check line start - assume this means at beginning of text or after newline
            if pattern.line_start {
                // Check if we're at start or after newline
                let at_line_start = pos == 0 || text[..pos].chars().last() == Some('\n');
                if !at_line_start {
                    continue;
                }
            }

            // Check before context (character before ch)
            if let Some(ref before_class) = pattern.before {
                let before_char = if pos >= 2 {
                    text[..pos-1].chars().last()
                } else {
                    None
                };
                if let Some(before_char) = before_char {
                    if !before_class.matches(before_char) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Check after context (character after ch)
            if let Some(ref after_class) = pattern.after {
                let after_char = if pos < text.len() {
                    text[pos..].chars().next()
                } else {
                    None
                };
                if let Some(after_char) = after_char {
                    if !after_class.matches(after_char) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contraction_suppression() {
        let sup = Suppresser::new(vec![(
            '\'',
            false,
            Some("alpha".to_string()),
            Some("alpha".to_string()),
        )]);

        // Should suppress apostrophe in contractions
        assert!(sup.should_suppress("don't", 4)); // Position after apostrophe
        assert!(!sup.should_suppress("end'", 4)); // No char after
    }
}
