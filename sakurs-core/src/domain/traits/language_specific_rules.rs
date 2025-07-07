//! Language-specific rules for sentence boundary detection

/// Type of quotation mark
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteType {
    /// Single quote (')
    Single,
    /// Double quote (")
    Double,
    /// Japanese corner bracket (「」)
    JapaneseCorner,
    /// Japanese double corner bracket (『』)
    JapaneseDoubleCorner,
    /// Other quote type
    Other,
}

/// Behavior for quotes in sentence boundary detection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteBehavior {
    /// Quote can contain sentence boundaries
    AllowBoundaries,
    /// Quote suppresses internal boundaries
    SuppressBoundaries,
    /// Context-dependent behavior
    Contextual,
}

/// Language-specific rules (pure logic)
pub trait LanguageSpecificRules: Send + Sync {
    /// Check if a word is an abbreviation
    fn is_abbreviation(&self, word: &str) -> bool;

    /// Get quote behavior for a quote type
    fn quote_behavior(&self, quote_type: QuoteType) -> QuoteBehavior;

    /// Check if a period after a word likely indicates abbreviation
    fn is_abbreviation_context(&self, word_before: &str, char_after: Option<char>) -> bool {
        // Default implementation
        if self.is_abbreviation(word_before) {
            // If known abbreviation, check what follows
            match char_after {
                Some(ch) if ch.is_uppercase() => false, // "Dr. Smith" - might be sentence boundary
                Some(ch) if ch.is_whitespace() => true, // Abbreviation with space
                _ => true,                              // Default to abbreviation
            }
        } else {
            false
        }
    }

    /// Check if sentence boundary should be suppressed based on following context
    fn should_suppress_boundary(&self, text_after: &str) -> bool {
        // Default: don't suppress if followed by uppercase or significant whitespace
        if let Some(first_non_ws) = text_after.chars().find(|c| !c.is_whitespace()) {
            !first_non_ws.is_uppercase()
        } else {
            true // Suppress if nothing follows
        }
    }

    /// Get language code
    fn language_code(&self) -> &str;
}
