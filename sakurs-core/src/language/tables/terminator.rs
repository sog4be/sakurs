//! Terminator character detection with O(1) lookup
//!
//! Optimized for hot-path performance with ASCII fast-path.

use std::collections::HashSet;

/// Fast terminator lookup table
#[derive(Debug, Clone)]
pub struct TermTable {
    /// ASCII lookup table for chars 0-127
    ascii_table: [bool; 128],
    /// HashSet for non-ASCII terminators (rare)
    non_ascii: HashSet<char>,
}

impl TermTable {
    /// Create from list of terminator characters
    pub fn new(terminators: Vec<char>) -> Self {
        let mut ascii_table = [false; 128];
        let mut non_ascii = HashSet::new();

        for ch in terminators {
            if ch.is_ascii() {
                ascii_table[ch as usize] = true;
            } else {
                non_ascii.insert(ch);
            }
        }

        Self {
            ascii_table,
            non_ascii,
        }
    }

    /// Check if character is a terminator - hot path
    #[inline]
    pub fn is_terminator(&self, ch: char) -> bool {
        if ch.is_ascii() {
            // Fast path: direct array lookup
            self.ascii_table[ch as usize]
        } else {
            // Slow path: hash lookup
            self.non_ascii.contains(&ch)
        }
    }
}

/// Dot context classifier
#[derive(Debug, Clone)]
pub struct DotTable;

impl DotTable {
    /// Create dot classifier
    pub fn new(_ellipsis_patterns: Vec<String>) -> Self {
        Self
    }

    /// Classify dot based on context - used by dot_role()
    #[inline]
    pub fn classify(
        &self,
        prev: Option<char>,
        next: Option<char>,
    ) -> crate::language::interface::DotRole {
        use crate::language::interface::DotRole;

        // Check for decimal point: digit.digit
        if let (Some(p), Some(n)) = (prev, next) {
            if p.is_ascii_digit() && n.is_ascii_digit() {
                return DotRole::DecimalDot;
            }
        }

        // Check for ellipsis: multiple dots
        if prev == Some('.') || next == Some('.') {
            return DotRole::EllipsisTail;
        }

        // Default is ordinary (abbreviation check happens elsewhere)
        DotRole::Ordinary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminator_lookup() {
        let table = TermTable::new(vec!['.', '!', '?', '。', '！', '？']);

        // ASCII fast path
        assert!(table.is_terminator('.'));
        assert!(table.is_terminator('!'));
        assert!(table.is_terminator('?'));
        assert!(!table.is_terminator(','));

        // Non-ASCII
        assert!(table.is_terminator('。'));
        assert!(table.is_terminator('！'));
        assert!(table.is_terminator('？'));
    }

    #[test]
    fn test_dot_classification() {
        use crate::language::interface::DotRole;
        let table = DotTable::new(vec![]);

        // Decimal
        assert_eq!(table.classify(Some('3'), Some('1')), DotRole::DecimalDot);

        // Ellipsis
        assert_eq!(table.classify(Some('.'), None), DotRole::EllipsisTail);
        assert_eq!(table.classify(None, Some('.')), DotRole::EllipsisTail);
    }
}
