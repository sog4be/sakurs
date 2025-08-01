//! Public contract for language rules - minimal and performance-focused
//!
//! This module defines the core traits and types used by the Δ-Stack scanner.
//! Everything here is designed for hot-path performance with zero allocations.

use crate::{character_window::CharacterWindow, types::Class};

// ========= Value types used by core scanner =========

/// Strength of a sentence boundary
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BoundaryStrength {
    /// Weak boundary (e.g., semicolon)
    Weak,
    /// Strong boundary (e.g., period, exclamation)
    Strong,
}

/// Information about an enclosure character (bracket, quote, etc.)
#[derive(Debug, Copy, Clone)]
pub struct EnclosureInfo {
    /// Numeric ID, used by Δ-stack index (0-254)
    pub type_id: u8,
    /// +1 for opening, -1 for closing, 0 for symmetric "unknown" mark
    pub delta: i8,
    /// true if identical char can mean both open/close (straight quote)
    pub symmetric: bool,
}

/// Role of a dot character in context
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DotRole {
    /// Regular sentence-ending period
    Ordinary,
    /// Part of ellipsis (...)
    EllipsisTail,
    /// Decimal point (3.14)
    DecimalDot,
    /// Abbreviation dot (Dr.)
    AbbrevDot,
}

/// Decision about a potential boundary
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BoundaryDecision {
    /// Accept as boundary with given strength
    Accept(BoundaryStrength),
    /// Reject - not a boundary
    Reject,
    /// Parallel mode: cannot decide because abbreviation may straddle chunk edge
    NeedsLookahead,
}

// ========= Core trait for language rules =========

/// Language-specific rules for sentence boundary detection
///
/// All methods are designed for hot-loop performance:
/// - O(1) character classification
/// - No allocations
/// - Minimal branching
/// - Cache-friendly access patterns
pub trait LanguageRules: Send + Sync + 'static {
    // --- Hot-loop primitives (all inlineable) ---

    /// Fast O(1) check: is this a terminator char or last byte of multi-byte terminator?
    fn is_terminator_char(&self, ch: char) -> bool;

    /// O(1): enclosure lookup; None if 'ch' is not any bracket/quote
    fn enclosure_info(&self, ch: char) -> Option<EnclosureInfo>;

    /// O(1): classify ASCII dot '.' context-aware
    fn dot_role(&self, prev: Option<char>, next: Option<char>) -> DotRole;

    // --- Semi-hot helpers (≤ one per candidate) ---

    /// Boundary decision for index 'pos' where `text[pos]` is guaranteed terminator
    /// prev_char and next_char are provided to avoid expensive character lookups
    fn boundary_decision(
        &self,
        text: &str,
        pos: usize,
        terminator_char: char,
        prev_char: Option<char>,
        next_char: Option<char>,
    ) -> BoundaryDecision;

    // --- Compatibility methods for existing code ---

    /// Character classification (for delta_stack compatibility)
    #[inline]
    fn classify_char(&self, ch: char) -> Class {
        if self.is_terminator_char(ch) {
            Class::Terminator
        } else if let Some(enc) = self.enclosure_info(ch) {
            if enc.delta > 0 {
                Class::Open
            } else {
                Class::Close
            }
        } else {
            Class::from_char(ch)
        }
    }

    /// Check if text ending at dot_pos is an abbreviation
    /// WARNING: This method has O(n) complexity and should not be used in hot paths
    /// Use is_abbreviation_efficient with CharacterWindow instead
    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool {
        if dot_pos == 0 || dot_pos >= text.len() {
            return false;
        }

        // Check if the dot at dot_pos is an abbreviation
        // boundary_decision expects pos to be AFTER the terminator
        // WARNING: These are O(n) operations that cause O(n²) overall complexity!
        let prev_char = if dot_pos > 0 {
            text[..dot_pos].chars().last()  // O(n) operation!
        } else {
            None
        };
        let next_char = if dot_pos + 1 < text.len() {
            text[dot_pos + 1..].chars().next()  // O(n) slice creation!
        } else {
            None
        };
        match self.boundary_decision(text, dot_pos + 1, '.', prev_char, next_char) {
            BoundaryDecision::Reject => true, // Rejected because it's an abbreviation
            _ => false,
        }
    }

    /// Check if a word matches known abbreviations (without dot)
    fn abbrev_match(&self, abbrev: &str) -> bool {
        // Build a test string with dot and check
        let test = format!("{abbrev}.");
        self.is_abbreviation(&test, abbrev.len()) // Position of the dot
    }

    /// Get enclosure pair info for compatibility
    fn get_enclosure_pair(&self, ch: char) -> Option<(u8, bool)> {
        self.enclosure_info(ch)
            .map(|info| (info.type_id, info.delta > 0))
    }

    /// Check if character is a terminator (compatibility)
    #[inline]
    fn is_terminator(&self, ch: char) -> bool {
        self.is_terminator_char(ch)
    }

    /// Get maximum number of enclosure pairs
    fn max_enclosure_pairs(&self) -> usize {
        // Default to a reasonable maximum
        16
    }

    /// Get pair ID for a character (if it's an enclosure)
    fn pair_id(&self, ch: char) -> Option<u8> {
        self.enclosure_info(ch).map(|info| info.type_id)
    }

    // --- High-Performance O(1) Methods using CharacterWindow ---

    /// Efficient boundary decision using character window (O(1))
    /// 
    /// This replaces the O(n) text scanning with O(1) character window access.
    /// Use this method instead of boundary_decision for optimal performance.
    fn boundary_decision_efficient(
        &self,
        window: &CharacterWindow,
        _byte_pos: usize,
    ) -> BoundaryDecision {
        // Default implementation falls back to boundary_decision method
        // Performance-critical implementations should override this to avoid text access
        
        
        // We need text access for the fallback, but we don't have it in the window
        // So we'll implement a basic check that should work for most cases
        let terminator_char = window.current_char().unwrap_or('.');
        let prev_char = window.prev_char();
        let next_char = window.next_char();
        
        // Basic terminator decision based on character context  
        match terminator_char {
            '.' => {
                // Check for common abbreviation patterns (Dr., Mr., etc.)
                if let Some(prev) = prev_char {
                    if prev.is_alphabetic() {
                        if let Some(prev_prev) = window.prev_prev_char() {
                            // Pattern like "Dr." - uppercase followed by lowercase
                            if prev_prev.is_uppercase() && prev.is_lowercase() {
                                return BoundaryDecision::Reject;
                            }
                            // Pattern like "U.S." - dot followed by uppercase
                            if prev_prev == '.' && prev.is_uppercase() {
                                return BoundaryDecision::Reject;
                            }
                            // Pattern for single uppercase letters in sequence "U."
                            if !prev_prev.is_alphabetic() && prev.is_uppercase() {
                                return BoundaryDecision::Reject;
                            }
                        } else {
                            // Single letter at start like "A."
                            if prev.is_uppercase() {
                                return BoundaryDecision::Reject;
                            }
                        }
                    }
                }
                
                // Check for decimal numbers (digit.digit)
                if let (Some(prev), Some(next)) = (prev_char, next_char) {
                    if prev.is_ascii_digit() && next.is_ascii_digit() {
                        return BoundaryDecision::Reject;
                    }
                }
                
                BoundaryDecision::Accept(BoundaryStrength::Strong)
            }
            '!' | '?' => BoundaryDecision::Accept(BoundaryStrength::Strong),
            _ => BoundaryDecision::Reject,
        }
    }

    /// Efficient abbreviation check using character window (O(1))
    ///
    /// Checks if the current position in the window represents an abbreviation.
    /// This eliminates the expensive text scanning of the legacy is_abbreviation method.
    fn is_abbreviation_efficient(&self, window: &CharacterWindow) -> bool {
        // Default implementation that should be overridden for performance
        // This is a placeholder - real implementations will use window data directly
        
        // Check if current character is a dot
        if window.current_char() != Some('.') {
            return false;
        }

        // Check if preceded by alphanumeric (typical abbreviation pattern)
        if let Some(prev) = window.prev_char() {
            prev.is_alphanumeric()
        } else {
            false
        }
    }

    /// Check if current position should be suppressed using character window (O(1))
    ///
    /// Determines if a boundary should be suppressed based on local context
    /// without expensive text scanning.
    fn should_suppress_efficient(&self, window: &CharacterWindow) -> bool {
        // Default implementation - should be overridden
        // Check for common suppression patterns
        match window.context_triple() {
            // Apostrophe between letters (contractions)
            (Some(prev), Some('\''), Some(next)) if prev.is_alphabetic() && next.is_alphabetic() => true,
            // Other common patterns can be added here
            _ => false,
        }
    }
}
