//! Ellipsis pattern detection
//!
//! Handles ... and other ellipsis patterns efficiently.

/// Ellipsis pattern matcher
#[derive(Debug, Clone)]
pub struct EllipsisSet {
    /// Known ellipsis patterns (e.g., "...", "…")
    patterns: Vec<String>,
    /// Whether ellipsis should be treated as boundary
    treat_as_boundary: bool,
}

impl EllipsisSet {
    /// Create ellipsis detector
    pub fn new(patterns: Vec<String>, treat_as_boundary: bool) -> Self {
        Self {
            patterns,
            treat_as_boundary,
        }
    }

    /// Check if position is part of ellipsis pattern
    pub fn is_ellipsis_at(&self, text: &str, pos: usize) -> bool {
        if pos >= text.len() {
            return false;
        }

        // Check each pattern
        for pattern in &self.patterns {
            if let Some(start) = pos.checked_sub(pattern.len() - 1) {
                if text[start..].starts_with(pattern) {
                    return true;
                }
            }
        }

        false
    }

    /// Should ellipsis be treated as boundary?
    pub fn treat_as_boundary(&self) -> bool {
        self.treat_as_boundary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsis_detection() {
        let set = EllipsisSet::new(vec!["...".to_string(), "…".to_string()], false);

        let text = "Hello...";
        println!(
            "Testing 'Hello...' at pos 5: {}",
            set.is_ellipsis_at(text, 5)
        );
        println!(
            "Testing 'Hello...' at pos 7: {}",
            set.is_ellipsis_at(text, 7)
        );
        assert!(!set.is_ellipsis_at(text, 5)); // 'o'
        assert!(set.is_ellipsis_at(text, 7)); // last '.'

        let text2 = "Test…";
        println!("\nText2: '{}'", text2);
        println!("Text2 bytes: {:?}", text2.as_bytes());
        println!("Text2 len: {}", text2.len());
        for i in 0..text2.len() {
            println!(
                "  pos {}: is_ellipsis_at = {}",
                i,
                set.is_ellipsis_at(text2, i)
            );
        }

        // The "…" character spans bytes 4-6 (3-byte UTF-8 character)
        // is_ellipsis_at returns true when checking at or near the end of the pattern
        assert!(set.is_ellipsis_at(text2, 6)); // End of '…'
    }
}
