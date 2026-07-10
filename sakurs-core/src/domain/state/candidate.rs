//! Boundary candidates and the judgment interface.
//!
//! Linguistic judgment is a pure function of a fixed-size window around a
//! candidate (see `docs/DELTA_STACK_ALGORITHM.md`, "The Judgment Function").
//! The state model is parameterized over that function through the [`Judge`]
//! trait, which keeps the monoid algebra independent of any concrete language
//! rules — associativity holds for *every* pure judge.

use crate::domain::types::{BoundaryFlags, DepthVec};

/// The terminator character that produced a candidate, carried so the
/// judgment can be re-invoked on a reconstructed window later. Multi-character
/// patterns and ellipses are re-detected from the window content itself,
/// which keeps the kind chunk-invariant by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminatorKind {
    /// A single terminator character (e.g. `.`, `。`).
    Char(char),
    /// A closing-capable enclosure character with a terminator behind it
    /// (possibly through a short chain of further closers), e.g. the `"` of
    /// `great." She`. The candidate sits just after the enclosure character;
    /// the judgment walks back through the chain, re-judges the terminator,
    /// and applies the boundary-after-closers follow condition.
    AfterClosers(char),
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

/// The pure linguistic decisions of a language: both are functions of a
/// window and nothing else — purity is what makes deferred judgment
/// order-independent.
pub(crate) trait Judge {
    /// Given the candidate's window, its byte offset inside the window, and
    /// the terminator kind, decide whether it ends a sentence.
    fn judge(&self, window: &str, pos_in_window: usize, kind: TerminatorKind) -> Judgment;

    /// Given an enclosure character's window and its byte offset inside it,
    /// decide whether the character is a non-enclosure use (contraction,
    /// possessive, …) that must be excluded from depth/parity tracking.
    /// Only invoked for characters the language marks as suppressible.
    fn suppress_enclosure(&self, window: &str, pos_in_window: usize, ch: char) -> bool;
}

/// The depth/parity effect an enclosure character has if it turns out to be a
/// real enclosure (not suppressed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnclosureSlot {
    /// Asymmetric type: add `delta` (+1 opener / −1 closer) to net depth
    /// `index`.
    Asym { index: u8, delta: i8 },
    /// Symmetric type: flip parity bit `bit`.
    Sym { bit: u8 },
}

impl EnclosureSlot {
    /// Whether this effect can close an enclosure. Symmetric toggles always
    /// can — whether a particular occurrence opens or closes is a global
    /// parity question the reduce predicate answers, not a local one.
    pub(crate) fn closing_capable(self) -> bool {
        match self {
            EnclosureSlot::Sym { .. } => true,
            EnclosureSlot::Asym { delta, .. } => delta < 0,
        }
    }
}

/// A suppressible enclosure character within `k` characters of a state edge,
/// carried with its depth/parity effect *excluded* from the state until the
/// suppression decision can be made on a full window. Resolving it as a real
/// enclosure retroactively applies [`EnclosureSlot`] to every candidate and
/// state total positioned after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PendingEnclosure {
    /// Byte position of the enclosure character, relative to the state's
    /// start.
    pub local_offset: usize,
    /// The character, for the suppression re-check.
    pub ch: char,
    /// Effect to apply if not suppressed.
    pub slot: EnclosureSlot,
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
