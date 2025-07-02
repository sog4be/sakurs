//! Language rules traits for sentence boundary detection
//!
//! This module defines the core traits that enable language-specific
//! sentence boundary detection within the Delta-Stack Monoid framework.

use crate::domain::BoundaryFlags;

/// Context information for sentence boundary detection
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryContext {
    /// The text being analyzed
    pub text: String,
    /// Position of potential boundary in the text
    pub position: usize,
    /// Character at the boundary position
    pub boundary_char: char,
    /// Characters before the boundary (up to 10 chars for context)
    pub preceding_context: String,
    /// Characters after the boundary (up to 10 chars for context)
    pub following_context: String,
}

/// Decision about whether a position represents a sentence boundary
#[derive(Debug, Clone, PartialEq)]
pub enum BoundaryDecision {
    /// This is a sentence boundary with specified strength
    Boundary(BoundaryFlags),
    /// This is not a sentence boundary
    NotBoundary,
    /// Requires more context to decide
    NeedsMoreContext,
}

/// Result of abbreviation processing
#[derive(Debug, Clone, PartialEq)]
pub struct AbbreviationResult {
    /// Whether an abbreviation was detected
    pub is_abbreviation: bool,
    /// Length of the abbreviation if detected
    pub length: usize,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

/// Context for quotation mark processing
#[derive(Debug, Clone, PartialEq)]
pub struct QuotationContext {
    /// The text being analyzed
    pub text: String,
    /// Position of the quotation mark
    pub position: usize,
    /// Type of quotation mark
    pub quote_char: char,
    /// Whether we're inside a quoted section
    pub inside_quotes: bool,
}

/// Decision about quotation mark handling
#[derive(Debug, Clone, PartialEq)]
pub enum QuotationDecision {
    /// Start of quoted section
    QuoteStart,
    /// End of quoted section
    QuoteEnd,
    /// Regular quotation mark (not affecting sentence boundaries)
    Regular,
}

/// Core trait for language-specific sentence boundary detection rules
///
/// This trait enables the Delta-Stack Monoid algorithm to work with
/// different languages by providing language-specific logic for:
/// - Detecting sentence boundaries
/// - Handling abbreviations
/// - Processing quotation marks and other punctuation
///
/// Implementations must be thread-safe to support parallel processing.
pub trait LanguageRules: Send + Sync {
    /// Detect whether a position represents a sentence boundary
    ///
    /// # Arguments
    /// * `context` - Context information about the potential boundary
    ///
    /// # Returns
    /// Decision about whether this position is a sentence boundary
    fn detect_sentence_boundary(&self, context: &BoundaryContext) -> BoundaryDecision;

    /// Process potential abbreviations at a given position
    ///
    /// # Arguments
    /// * `text` - The text being analyzed
    /// * `position` - Position to check for abbreviations
    ///
    /// # Returns
    /// Result indicating if an abbreviation was found and its properties
    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult;

    /// Handle quotation marks and their effect on sentence boundaries
    ///
    /// # Arguments
    /// * `context` - Context information about the quotation mark
    ///
    /// # Returns
    /// Decision about how to handle this quotation mark
    fn handle_quotation(&self, context: &QuotationContext) -> QuotationDecision;

    /// Get the language identifier for this rule set
    ///
    /// # Returns
    /// Language code (e.g., "en", "ja", "es")
    fn language_code(&self) -> &str;

    /// Get the display name for this language
    ///
    /// # Returns
    /// Human-readable language name (e.g., "English", "Japanese", "Spanish")
    fn language_name(&self) -> &str;

    /// Get the enclosure character information for a character
    ///
    /// # Arguments
    /// * `ch` - Character to check
    ///
    /// # Returns
    /// Enclosure character info if this is an enclosure, None otherwise
    fn get_enclosure_char(&self, ch: char) -> Option<crate::domain::enclosure::EnclosureChar>;

    /// Get the enclosure type ID for a character
    ///
    /// Maps enclosure characters to their type IDs for delta stack tracking.
    /// The ID should be consistent for matching open/close pairs.
    ///
    /// # Arguments
    /// * `ch` - Character to check
    ///
    /// # Returns
    /// Type ID (0-based index) if this is an enclosure, None otherwise
    fn get_enclosure_type_id(&self, ch: char) -> Option<usize>;

    /// Get the total number of enclosure types supported
    ///
    /// This determines the size of the delta stack vector.
    ///
    /// # Returns
    /// Number of distinct enclosure types
    fn enclosure_type_count(&self) -> usize;
}

/// Trait for combining multiple language rules
///
/// This allows for handling mixed-language text or fallback behavior.
pub trait LanguageRuleSet: Send + Sync {
    /// Get the primary language rules
    fn primary_rules(&self) -> &dyn LanguageRules;

    /// Get fallback rules for unknown or mixed content
    fn fallback_rules(&self) -> &dyn LanguageRules;

    /// Detect the appropriate language rules for a text segment
    ///
    /// # Arguments
    /// * `text` - Text segment to analyze
    ///
    /// # Returns
    /// Reference to the most appropriate language rules
    fn detect_language_rules(&self, text: &str) -> &dyn LanguageRules;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_context_creation() {
        let context = BoundaryContext {
            text: "Hello world. This is a test.".to_string(),
            position: 11,
            boundary_char: '.',
            preceding_context: "Hello world".to_string(),
            following_context: " This is a".to_string(),
        };

        assert_eq!(context.position, 11);
        assert_eq!(context.boundary_char, '.');
    }

    #[test]
    fn test_boundary_decision_variants() {
        let strong_boundary = BoundaryDecision::Boundary(BoundaryFlags::STRONG);
        let not_boundary = BoundaryDecision::NotBoundary;
        let needs_context = BoundaryDecision::NeedsMoreContext;

        match strong_boundary {
            BoundaryDecision::Boundary(flags) => assert_eq!(flags, BoundaryFlags::STRONG),
            _ => panic!("Expected boundary decision"),
        }

        assert_eq!(not_boundary, BoundaryDecision::NotBoundary);
        assert_eq!(needs_context, BoundaryDecision::NeedsMoreContext);
    }

    #[test]
    fn test_abbreviation_result() {
        let result = AbbreviationResult {
            is_abbreviation: true,
            length: 3,
            confidence: 0.95,
        };

        assert!(result.is_abbreviation);
        assert_eq!(result.length, 3);
        assert!((result.confidence - 0.95).abs() < f32::EPSILON);
    }
}
