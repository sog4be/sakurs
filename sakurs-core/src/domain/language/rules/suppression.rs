use crate::domain::enclosure_suppressor::{EnclosureContext, EnclosureSuppressor};

/// Pattern for fast enclosure suppression
#[derive(Debug, Clone)]
pub struct FastPattern {
    /// Character to match
    pub char: char,
    /// Whether this must be at line start
    pub line_start: bool,
    /// Character class to match before
    pub before_matcher: Option<String>,
    /// Character class to match after
    pub after_matcher: Option<String>,
}

/// Configurable suppression rules
#[derive(Debug)]
pub struct Suppressor {
    /// Fast patterns for suppression
    patterns: Vec<FastPattern>,
}

impl Suppressor {
    /// Create new suppressor from configuration
    pub fn new(patterns: Vec<(char, bool, Option<String>, Option<String>)>) -> Self {
        let parsed_patterns: Vec<FastPattern> = patterns
            .into_iter()
            .map(|(ch, line_start, before, after)| FastPattern {
                char: ch,
                line_start,
                before_matcher: before,
                after_matcher: after,
            })
            .collect();

        Self {
            patterns: parsed_patterns,
        }
    }

    /// Check if a character matches a pattern
    fn char_matches_pattern(ch: Option<char>, pattern: &str) -> bool {
        match (ch, pattern) {
            (Some(c), "alpha") => c.is_alphabetic(),
            (Some(c), "alnum") => c.is_alphanumeric(),
            (Some(c), "digit") => c.is_ascii_digit(),
            (Some(c), "whitespace") => c.is_whitespace(),
            (None, _) => false,
            _ => false,
        }
    }

    /// Check if any pattern matches the context
    fn matches_any_pattern(&self, ch: char, context: &EnclosureContext) -> bool {
        for pattern in &self.patterns {
            if self.pattern_matches(pattern, ch, context) {
                return true;
            }
        }
        false
    }

    /// Check if a specific pattern matches
    fn pattern_matches(&self, pattern: &FastPattern, ch: char, context: &EnclosureContext) -> bool {
        // Check character match
        if pattern.char != ch {
            return false;
        }

        // Check line start condition
        if pattern.line_start && context.line_offset > 0 {
            return false;
        }

        // Check before matcher
        if let Some(ref pattern_str) = pattern.before_matcher {
            let char_before = context.preceding_chars.last().copied();
            if !Self::char_matches_pattern(char_before, pattern_str) {
                return false;
            }
        }

        // Check after matcher
        if let Some(ref pattern_str) = pattern.after_matcher {
            let char_after = context.following_chars.first().copied();
            if !Self::char_matches_pattern(char_after, pattern_str) {
                return false;
            }
        }

        true
    }
}

impl EnclosureSuppressor for Suppressor {
    fn should_suppress_enclosure(&self, ch: char, context: &EnclosureContext) -> bool {
        self.matches_any_pattern(ch, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::SmallVec;

    fn create_context(
        preceding: &[char],
        following: &[char],
        line_offset: usize,
    ) -> EnclosureContext<'static> {
        let preceding_chars: SmallVec<[char; 3]> = preceding.iter().copied().collect();
        let following_chars: SmallVec<[char; 3]> = following.iter().copied().collect();

        EnclosureContext {
            position: 0,
            preceding_chars,
            following_chars,
            line_offset,
            chunk_text: "",
        }
    }

    #[test]
    fn test_apostrophe_suppression() {
        let suppressor = Suppressor::new(vec![(
            '\'',
            false,
            Some("alpha".to_string()),
            Some("alpha".to_string()),
        )]);

        // Should suppress apostrophe in contractions
        let context = create_context(&['t'], &['s'], 10);
        assert!(suppressor.should_suppress_enclosure('\'', &context));

        // Should not suppress when not surrounded by letters
        let context = create_context(&[' '], &['H'], 10);
        assert!(!suppressor.should_suppress_enclosure('\'', &context));
    }

    #[test]
    fn test_line_start_suppression() {
        let suppressor = Suppressor::new(vec![(')', true, Some("alnum".to_string()), None)]);

        // Should suppress at line start after alphanumeric
        let context = create_context(&['1'], &[' '], 0);
        assert!(suppressor.should_suppress_enclosure(')', &context));

        // Should not suppress when not at line start
        let context = create_context(&['1'], &[' '], 10);
        assert!(!suppressor.should_suppress_enclosure(')', &context));
    }

    #[test]
    fn test_no_patterns() {
        let suppressor = Suppressor::new(vec![]);

        let context = create_context(&['t'], &['s'], 10);
        assert!(!suppressor.should_suppress_enclosure('\'', &context));
    }
}
