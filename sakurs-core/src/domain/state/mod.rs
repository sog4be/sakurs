//! Partial-state model for the Δ-Stack Monoid algorithm.
//!
//! A [`PartialState`] represents a scanned text span as
//! `⟨B, P, Δ, π, H, T⟩` (see `docs/DELTA_STACK_ALGORITHM.md`): confirmed
//! boundary candidates, pending candidates awaiting cross-chunk context,
//! `(net, min)` deltas for asymmetric enclosures, a parity bitset for
//! symmetric enclosures, and head/tail context buffers of `2k` characters.
//!
//! [`PartialState::combine_with`] is the monoid operation, parameterized by a
//! [`Judge`] so the algebra stays independent of concrete language rules;
//! associativity holds for every pure judge and is verified by property tests
//! in this module.

mod candidate;
mod context;

pub(crate) use candidate::{Candidate, Judge, Judgment, PendingCandidate, TerminatorKind};
pub(crate) use context::{window_around, ContextBuf, CONTEXT_CHARS, WINDOW_CHARS};

use crate::domain::types::{DeltaEntry, DeltaVec};
use smallvec::SmallVec;

/// Candidates per state; spills to the heap for large chunks.
pub(crate) type CandidateVec = SmallVec<[Candidate; 8]>;

/// Pending candidates per state; at most a handful of terminators fall within
/// `k` characters of a chunk edge in practice.
pub(crate) type PendingVec = SmallVec<[PendingCandidate; 4]>;

/// Parsing state of a text span under the Δ-Stack Monoid algorithm.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PartialState {
    /// Linguistically confirmed candidates (B), sorted by offset.
    pub boundaries: CandidateVec,
    /// Unjudged candidates within `k` characters of a state edge (P), sorted
    /// by offset.
    pub pending: PendingVec,
    /// `(net, min)` per asymmetric enclosure type (Δ).
    pub deltas: DeltaVec,
    /// Toggle parity per symmetric enclosure type (π).
    pub parity: u32,
    /// First `min(2k, |span|)` characters of the span (H).
    pub head_ctx: ContextBuf,
    /// Last `min(2k, |span|)` characters of the span (T).
    pub tail_ctx: ContextBuf,
    /// Byte length of the span.
    pub chunk_len: usize,
}

impl PartialState {
    /// The identity element: the state of the empty span.
    pub(crate) fn identity() -> Self {
        Self {
            boundaries: CandidateVec::new(),
            pending: PendingVec::new(),
            deltas: DeltaVec::new(),
            parity: 0,
            head_ctx: ContextBuf::empty(),
            tail_ctx: ContextBuf::empty(),
            chunk_len: 0,
        }
    }

    /// Characters available before byte offset `p`, saturating at
    /// [`CONTEXT_CHARS`]. Exact whenever the true count is below the
    /// saturation point, which is all the `< WINDOW_CHARS` comparisons need.
    fn chars_before(&self, p: usize) -> usize {
        if p <= self.head_ctx.byte_len() {
            self.head_ctx.as_str()[..p].chars().count()
        } else {
            // p lies beyond the head buffer, hence beyond its 2k characters.
            CONTEXT_CHARS
        }
    }

    /// Characters available after byte offset `p`, saturating at
    /// [`CONTEXT_CHARS`] (see [`Self::chars_before`]).
    fn chars_after(&self, p: usize) -> usize {
        let tail_start = self.chunk_len - self.tail_ctx.byte_len();
        if p >= tail_start {
            self.tail_ctx.as_str()[p - tail_start..].chars().count()
        } else {
            CONTEXT_CHARS
        }
    }

    /// Rebases a right-hand candidate to the combined state's origin.
    fn rebase(&self, c: &Candidate) -> Candidate {
        Candidate {
            local_offset: c.local_offset + self.chunk_len,
            local_depths: self.rebase_depths(&c.local_depths),
            local_parity: c.local_parity ^ self.parity,
            flags: c.flags,
        }
    }

    /// Rebases a right-hand pending candidate to the combined state's origin.
    fn rebase_pending(&self, p: &PendingCandidate) -> PendingCandidate {
        PendingCandidate {
            local_offset: p.local_offset + self.chunk_len,
            local_depths: self.rebase_depths(&p.local_depths),
            local_parity: p.local_parity ^ self.parity,
            kind: p.kind,
        }
    }

    fn rebase_depths(
        &self,
        depths: &crate::domain::types::DepthVec,
    ) -> crate::domain::types::DepthVec {
        let len = depths.len().max(self.deltas.len());
        (0..len)
            .map(|i| depths.get(i).copied().unwrap_or(0) + self.deltas.get(i).map_or(0, |e| e.net))
            .collect()
    }

    /// The monoid operation: state of `text(self) ++ text(other)`.
    ///
    /// Pending candidates whose ±k window becomes available are judged here,
    /// on a window reconstructed from `self.tail_ctx ++ other.head_ctx`; the
    /// window's content equals the corresponding substring of the original
    /// text regardless of combine order, so any pure `judge` yields the same
    /// verdicts for every parenthesization.
    pub(crate) fn combine_with<J: Judge>(&self, other: &Self, judge: &J) -> Self {
        // Δ: classical (net, min) merge, padding missing types with identity.
        let max_deltas = self.deltas.len().max(other.deltas.len());
        let identity = DeltaEntry::identity();
        let mut deltas = DeltaVec::with_capacity(max_deltas);
        for i in 0..max_deltas {
            let l = self.deltas.get(i).unwrap_or(&identity);
            let r = other.deltas.get(i).unwrap_or(&identity);
            deltas.push(l.combine(r));
        }

        let mut combined = Self {
            boundaries: CandidateVec::new(),
            pending: PendingVec::new(),
            deltas,
            parity: self.parity ^ other.parity,
            head_ctx: ContextBuf::compose_head(&self.head_ctx, &other.head_ctx),
            tail_ctx: ContextBuf::compose_tail(&self.tail_ctx, &other.tail_ctx),
            chunk_len: self.chunk_len + other.chunk_len,
        };

        combined.boundaries.extend(self.boundaries.iter().cloned());
        combined
            .boundaries
            .extend(other.boundaries.iter().map(|c| self.rebase(c)));

        // The joint context window covers the byte range
        // [self.chunk_len - |tail|, self.chunk_len + |head_r|) of the combined
        // span; every candidate resolvable at this combine falls inside it
        // (docs/DELTA_STACK_ALGORITHM.md, "Window Availability").
        let mut joint = [0u8; 2 * CONTEXT_CHARS * 4];
        let lt = self.tail_ctx.as_str().as_bytes();
        let rh = other.head_ctx.as_str().as_bytes();
        joint[..lt.len()].copy_from_slice(lt);
        joint[lt.len()..lt.len() + rh.len()].copy_from_slice(rh);
        let joint_str = std::str::from_utf8(&joint[..lt.len() + rh.len()])
            .expect("context buffers hold valid UTF-8");
        let joint_start = self.chunk_len - self.tail_ctx.byte_len();

        let left_pending = self.pending.iter().cloned();
        let right_pending = other.pending.iter().map(|p| self.rebase_pending(p));
        for pc in left_pending.chain(right_pending) {
            let p = pc.local_offset;
            if combined.chars_before(p) >= WINDOW_CHARS && combined.chars_after(p) >= WINDOW_CHARS {
                debug_assert!(
                    p >= joint_start && p <= joint_start + joint_str.len(),
                    "resolvable pending candidate must lie inside the joint window"
                );
                let (window, pos) = window_around(joint_str, p - joint_start, WINDOW_CHARS);
                debug_assert_eq!(
                    window.chars().count(),
                    2 * WINDOW_CHARS,
                    "a window resolved at combine is never clipped (Window Availability)"
                );
                if let Judgment::Boundary(flags) = judge.judge(window, pos, pc.kind) {
                    combined.boundaries.push(pc.confirm(flags));
                }
            } else {
                combined.pending.push(pc);
            }
        }

        combined.boundaries.sort_unstable_by_key(|c| c.local_offset);
        combined
    }

    /// Judges the remaining pending candidates with the knowledge that no
    /// more text is coming: missing left context resolves against the start
    /// of text, missing right context against the end (the window is clipped
    /// instead of completed). Called once by the driver after the final
    /// combine; sits outside the monoid.
    pub(crate) fn resolve_edges<J: Judge>(mut self, judge: &J) -> Self {
        let pending = std::mem::take(&mut self.pending);
        for pc in pending {
            let p = pc.local_offset;
            // Pick the buffer whose coverage contains the candidate's clipped
            // window: candidates lacking left context sit within k characters
            // of the text start (window ⊆ head buffer), all others lack right
            // context and sit within k characters of the end (⊆ tail buffer).
            let (buf, buf_start) = if self.chars_before(p) < WINDOW_CHARS {
                (self.head_ctx.as_str(), 0)
            } else {
                let tail_start = self.chunk_len - self.tail_ctx.byte_len();
                debug_assert!(p >= tail_start, "pending candidate outside both buffers");
                (self.tail_ctx.as_str(), tail_start)
            };
            let (window, pos) = window_around(buf, p - buf_start, WINDOW_CHARS);
            if let Judgment::Boundary(flags) = judge.judge(window, pos, pc.kind) {
                self.boundaries.push(pc.confirm(flags));
            }
        }
        self.boundaries.sort_unstable_by_key(|c| c.local_offset);
        self
    }
}

#[cfg(test)]
mod tests;
