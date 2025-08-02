//! Core types for sentence boundary detection

use core::fmt;

/// Type of sentence boundary detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "std", derive(Default))]
pub enum BoundaryKind {
    /// Strong boundary (., !, ?)
    #[cfg_attr(feature = "std", default)]
    Strong,
    /// Weak boundary (may be overridden)
    Weak,
    /// Abbreviation boundary (special handling)
    Abbreviation,
}

/// A detected sentence boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boundary {
    /// Byte offset in the text
    pub byte_offset: usize,
    /// Character offset in the text
    pub char_offset: usize,
    /// Type of boundary
    pub kind: BoundaryKind,
}

impl Boundary {
    /// Create a new boundary
    pub fn new(byte_offset: usize, char_offset: usize, kind: BoundaryKind) -> Self {
        Self {
            byte_offset,
            char_offset,
            kind,
        }
    }
}

/// Coarse character classification for fast lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Alphabetic character
    Alpha,
    /// Numeric digit
    Digit,
    /// Period/dot character
    Dot,
    /// Sentence terminator (., !, ?)
    Terminator,
    /// Opening enclosure
    Open,
    /// Closing enclosure
    Close,
    /// Whitespace
    Space,
    /// Other character
    Other,
}

impl Class {
    /// Classify a character
    pub fn from_char(ch: char) -> Self {
        match ch {
            'a'..='z' | 'A'..='Z' => Class::Alpha,
            '0'..='9' => Class::Digit,
            '.' => Class::Dot,
            '!' | '?' => Class::Terminator,
            '(' | '[' | '{' | '「' | '『' => Class::Open,
            ')' | ']' | '}' | '」' | '』' => Class::Close,
            ' ' | '\t' | '\n' | '\r' => Class::Space,
            _ => Class::Other,
        }
    }
}

impl fmt::Display for BoundaryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundaryKind::Strong => write!(f, "strong"),
            BoundaryKind::Weak => write!(f, "weak"),
            BoundaryKind::Abbreviation => write!(f, "abbreviation"),
        }
    }
}
