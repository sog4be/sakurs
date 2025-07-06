//! Type aliases for SmallVec optimizations

use super::{BoundaryCandidate, DeltaEntry};
use smallvec::SmallVec;

/// Optimized vector for boundary candidates
/// Most chunks have < 32 boundary candidates
pub type BoundaryVec = SmallVec<[BoundaryCandidate; 32]>;

/// Optimized vector for delta entries
/// Enclosure tracking rarely needs > 16 entries per chunk
pub type DeltaVec = SmallVec<[DeltaEntry; 16]>;

/// Optimized vector for local depths
/// Enclosure depth rarely exceeds 8 levels
pub type DepthVec = SmallVec<[i32; 8]>;

/// Optimized vector for small integer collections
pub type SmallIntVec = SmallVec<[i32; 4]>;
