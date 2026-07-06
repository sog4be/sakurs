//! Boundary candidates and the judgment interface.
//!
//! Linguistic judgment is a pure function of a fixed-size window around a
//! candidate (see `docs/DELTA_STACK_ALGORITHM.md`, "The Judgment Function").
//! The state model is parameterized over that function through the [`Judge`]
//! trait, which keeps the monoid algebra independent of any concrete language
//! rules — associativity holds for *every* pure judge.

use crate::domain::types::{BoundaryFlags, DepthVec};

/// Classification of the terminator that produced a candidate, carried so the
/// judgment can be re-invoked on a reconstructed window later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminatorKind {
    /// A single terminator character (e.g. `.`, `。`).
    Char(char),
    /// A multi-character terminator pattern ending at the candidate (e.g. `!?`).
    Pattern { len: u8 },
    /// An ellipsis sequence ending at the candidate (e.g. `...`, `…`).
    Ellipsis { len: u8 },
}

/// Verdict of the judgment function for one candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Judgment {
    /// The candidate ends a sentence (subject to the structural depth/parity
    /// check in the reduce phase).
    Boundary(BoundaryFlags),
    /// The candidate does not end a sentence (abbreviation, decimal point, …).
    NotBoundary,
}

/// A pure linguistic judgment: given the candidate's window, its byte offset
/// inside the window, and the terminator kind, decide whether it ends a
/// sentence. Implementations must depend on nothing but these arguments —
/// purity is what makes deferred judgment order-independent.
pub(crate) trait Judge {
    fn judge(&self, window: &str, pos_in_window: usize, kind: TerminatorKind) -> Judgment;
}

/// A linguistically confirmed candidate. Only the structural check against
/// global enclosure depth/parity remains for the reduce phase.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Candidate {
    /// Byte offset just after the terminator, relative to the state's start.
    pub local_offset: usize,
    /// Asymmetric enclosure depths at the candidate, relative to the state's
    /// start (rebased on combine).
    pub local_depths: DepthVec,
    /// Symmetric enclosure parity at the candidate, relative to the state's
    /// start (one bit per symmetric type, rebased on combine).
    pub local_parity: u32,
    /// Classification flags assigned by the judgment.
    pub flags: BoundaryFlags,
}

/// A candidate within `k` characters of a state edge, carried unjudged until
/// a combine (or edge resolution) supplies the missing context.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PendingCandidate {
    /// Byte offset just after the terminator, relative to the state's start.
    pub local_offset: usize,
    /// Asymmetric enclosure depths at the candidate (as in [`Candidate`]).
    pub local_depths: DepthVec,
    /// Symmetric enclosure parity at the candidate (as in [`Candidate`]).
    pub local_parity: u32,
    /// What to re-judge once the window is available.
    pub kind: TerminatorKind,
}

impl PendingCandidate {
    /// Confirms this candidate with the flags assigned by the judgment.
    pub(crate) fn confirm(&self, flags: BoundaryFlags) -> Candidate {
        Candidate {
            local_offset: self.local_offset,
            local_depths: self.local_depths.clone(),
            local_parity: self.local_parity,
            flags,
        }
    }
}
