//! Public contract for language rules - minimal and performance-focused
//!
//! This module defines the core traits and types used by the Δ-Stack scanner.
//! Everything here is designed for hot-path performance with zero allocations.

use crate::types::Class;

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
    fn is_abbreviation(&self, text: &str, dot_pos: usize) -> bool {
        if dot_pos == 0 || dot_pos >= text.len() {
            return false;
        }

        // Check if the dot at dot_pos is an abbreviation
        // boundary_decision expects pos to be AFTER the terminator
        let prev_char = if dot_pos > 0 {
            text.chars().nth(dot_pos - 1)
        } else {
            None
        };
        let next_char = if dot_pos + 1 < text.len() {
            text.chars().nth(dot_pos + 1)
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
}
