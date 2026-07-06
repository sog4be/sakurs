//! Shared type definitions for the domain layer.

use smallvec::SmallVec;

/// Represents a sentence boundary with metadata
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Boundary {
    /// Byte offset of the boundary in the text
    pub offset: usize,
    /// Boundary classification flags
    pub flags: BoundaryFlags,
}

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
    pub const FROM_ABBR: Self = Self {
        is_strong: true,
        from_abbreviation: true,
    };

    /// Check if this flags contains all the flags set in `other`
    ///
    /// Returns true if all flags that are set in `other` are also set in `self`.
    pub fn contains(&self, other: Self) -> bool {
        if other.is_strong && !self.is_strong {
            return false;
        }
        if other.from_abbreviation && !self.from_abbreviation {
            return false;
        }
        true
    }
}

impl Boundary {
    /// Creates a new boundary at the specified offset
    pub fn new(offset: usize) -> Self {
        Self {
            offset,
            flags: BoundaryFlags::WEAK,
        }
    }

    /// Creates a new boundary with specific flags
    pub fn with_flags(offset: usize, flags: BoundaryFlags) -> Self {
        Self { offset, flags }
    }
}

/// Optimized vector for local depths
/// Enclosure depth rarely exceeds 8 levels
pub type DepthVec = SmallVec<[i32; 8]>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_ordering() {
        let b1 = Boundary {
            offset: 10,
            flags: BoundaryFlags::STRONG,
        };
        let b2 = Boundary {
            offset: 20,
            flags: BoundaryFlags::WEAK,
        };
        assert!(b1 < b2);
    }

    #[test]
    fn flags_containment() {
        let both = BoundaryFlags {
            is_strong: true,
            from_abbreviation: true,
        };
        assert!(both.contains(BoundaryFlags::STRONG));
        assert!(both.contains(BoundaryFlags::FROM_ABBR));
        assert!(!BoundaryFlags::WEAK.contains(BoundaryFlags::STRONG));
    }
}
