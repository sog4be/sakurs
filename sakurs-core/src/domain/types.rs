//! Shared type definitions for the domain layer.

use smallvec::SmallVec;

/// Classification flags for sentence boundaries
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundaryFlags {
    /// Strong boundary (e.g., `!`, `?`, multi-character terminator patterns)
    pub is_strong: bool,
    /// Boundary confirmed after abbreviation resolution
    pub from_abbreviation: bool,
}

impl BoundaryFlags {
    pub const STRONG: Self = Self {
        is_strong: true,
        from_abbreviation: false,
    };
    pub const WEAK: Self = Self {
        is_strong: false,
        from_abbreviation: false,
    };
}

/// Optimized vector for local depths
/// Enclosure depth rarely exceeds 8 levels
pub type DepthVec = SmallVec<[i32; 8]>;
