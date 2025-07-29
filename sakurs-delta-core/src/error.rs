//! Core error types (deterministic only)

use core::fmt;

/// Core algorithm errors (no I/O, no external failures)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Integer overflow in calculations
    Overflow,
    /// Invalid UTF-8 boundary
    InvalidUtf8Boundary,
    /// Enclosure limit exceeded
    TooManyEnclosureTypes,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Overflow => write!(f, "integer overflow in boundary calculation"),
            CoreError::InvalidUtf8Boundary => write!(f, "invalid UTF-8 boundary"),
            CoreError::TooManyEnclosureTypes => write!(f, "exceeded maximum enclosure types"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CoreError {}

/// Result type for core operations
pub type Result<T> = core::result::Result<T, CoreError>;
