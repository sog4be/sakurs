//! Partial-state model for the Δ-Stack Monoid algorithm.
//!
//! A [`PartialState`] represents a scanned text span as
//! `⟨B, P, E, Δ, π, H, T⟩` (see `docs/DELTA_STACK_ALGORITHM.md`): confirmed
//! boundary candidates, pending candidates awaiting cross-chunk context,
//! pending enclosure characters whose suppression decision needs cross-chunk
//! context, net depth deltas for asymmetric enclosures, a parity bitset for
//! symmetric enclosures, and head/tail context buffers of `2k` characters.
//!
//! [`PartialState::combine_with`] is the monoid operation, parameterized by a
//! [`Judge`] so the algebra stays independent of concrete language rules;
//! associativity holds for every pure judge and is verified by property tests
//! in this module.

mod candidate;
mod compiled;
mod context;
mod scanner;

pub(crate) use candidate::{
    Candidate, EnclosureSlot, Judge, Judgment, PendingCandidate, PendingEnclosure, TerminatorKind,
};
pub(crate) use compiled::CompiledRules;
pub(crate) use context::{window_around, ContextBuf, CONTEXT_CHARS, WINDOW_CHARS};
pub(crate) use scanner::scan_chunk;

use crate::domain::types::DepthVec;
use smallvec::SmallVec;

/// Candidates per state; spills to the heap for large chunks.
pub(crate) type CandidateVec = SmallVec<[Candidate; 8]>;

/// Pending candidates per state; at most a handful of terminators fall within
/// `k` characters of a chunk edge in practice.
pub(crate) type PendingVec = SmallVec<[PendingCandidate; 4]>;

/// Pending enclosures per state; only suppressible enclosure characters near
/// a chunk edge become pending.
pub(crate) type PendingEncVec = SmallVec<[PendingEnclosure; 2]>;

/// Resolved enclosure toggles collected during one combine, in combined-state
/// coordinates.
type ToggleVec = SmallVec<[(usize, EnclosureSlot); 4]>;

/// Parsing state of a text span under the Δ-Stack Monoid algorithm.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PartialState {
    /// Linguistically confirmed candidates (B), sorted by offset.
    pub boundaries: CandidateVec,
    /// Unjudged candidates within `k` characters of a state edge (P), sorted
    /// by offset.
    pub pending: PendingVec,
    /// Suppression-undecided enclosure characters within `k` characters of a
    /// state edge (E), sorted by offset. Their depth/parity effect is
    /// excluded from `deltas`/`parity` and from every candidate until
    /// resolved.
    pub pending_enc: PendingEncVec,
    /// Net depth change per asymmetric enclosure type (Δ).
    pub deltas: DepthVec,
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
            pending_enc: PendingEncVec::new(),
            deltas: DepthVec::new(),
            parity: 0,
            head_ctx: ContextBuf::empty(),
            tail_ctx: ContextBuf::empty(),
            chunk_len: 0,
        }
    }

    /// Characters available strictly before byte offset `p`, saturating at
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

    /// Characters available from byte offset `p` (inclusive) to the span end,
    /// saturating at [`CONTEXT_CHARS`] (see [`Self::chars_before`]).
    fn chars_after(&self, p: usize) -> usize {
        let tail_start = self.chunk_len - self.tail_ctx.byte_len();
        if p >= tail_start {
            self.tail_ctx.as_str()[p - tail_start..].chars().count()
        } else {
            CONTEXT_CHARS
        }
    }

    /// True when the ±k window around `p` lies fully inside the span, i.e.
    /// the pending item at `p` can be resolved.
    fn window_available(&self, p: usize) -> bool {
        self.chars_before(p) >= WINDOW_CHARS && self.chars_after(p) >= WINDOW_CHARS
    }

    /// The monoid operation: state of `text(self) ++ text(other)`.
    ///
    /// Pending items whose ±k window becomes available are resolved here, on
    /// a window reconstructed from `self.tail_ctx ++ other.head_ctx`; the
    /// window's content equals the corresponding substring of the original
    /// text regardless of combine order, so any pure `judge` yields the same
    /// verdicts for every parenthesization. Enclosures resolve before
    /// candidates because a confirmed enclosure retroactively shifts the
    /// depth/parity of everything positioned after it.
    pub(crate) fn combine_with<J: Judge>(&self, other: &Self, judge: &J) -> Self {
        let mut acc = self.clone();
        acc.absorb(other, judge);
        acc
    }

    /// In-place [`Self::combine_with`]: `self` becomes the state of
    /// `text(self) ++ text(other)`. The driver folds the chunk states with
    /// this, so the accumulated candidates are never re-copied — one fold
    /// over the whole text does O(total items) work.
    pub(crate) fn absorb<J: Judge>(&mut self, other: &Self, judge: &J) {
        // Rebasing `other`'s items needs the left-hand totals as they were
        // before the merge.
        let left_len = self.chunk_len;
        let left_deltas = self.deltas.clone();
        let left_parity = self.parity;

        // The joint context window covers the byte range
        // [left_len - |tail|, left_len + |head_r|) of the combined span;
        // every item resolvable at this combine falls inside it
        // (docs/DELTA_STACK_ALGORITHM.md, "Window Availability").
        let mut joint = [0u8; 2 * CONTEXT_CHARS * 4];
        let lt = self.tail_ctx.as_str().as_bytes();
        let rh = other.head_ctx.as_str().as_bytes();
        joint[..lt.len()].copy_from_slice(lt);
        joint[lt.len()..lt.len() + rh.len()].copy_from_slice(rh);
        let joint_str = std::str::from_utf8(&joint[..lt.len() + rh.len()])
            .expect("context buffers hold valid UTF-8");
        let joint_start = left_len - self.tail_ctx.byte_len();

        // Merge totals, contexts, and length; window availability below must
        // see the combined state.
        if other.deltas.len() > self.deltas.len() {
            self.deltas.resize(other.deltas.len(), 0);
        }
        for (i, d) in other.deltas.iter().enumerate() {
            self.deltas[i] += d;
        }
        self.parity ^= other.parity;
        self.head_ctx = ContextBuf::compose_head(&self.head_ctx, &other.head_ctx);
        self.tail_ctx = ContextBuf::compose_tail(&self.tail_ctx, &other.tail_ctx);
        self.chunk_len += other.chunk_len;

        // Pending enclosures first: confirmed ones shift everything after
        // them, so their toggles must be known before candidates are placed.
        let mut toggles = ToggleVec::new();
        let prior_enc = std::mem::take(&mut self.pending_enc);
        let right_enc = other.pending_enc.iter().map(|pe| PendingEnclosure {
            local_offset: pe.local_offset + left_len,
            ..*pe
        });
        for pe in prior_enc.into_iter().chain(right_enc) {
            if self.window_available(pe.local_offset) {
                let (window, pos) = resolve_window(joint_str, joint_start, pe.local_offset);
                if !judge.suppress_enclosure(window, pos, pe.ch) {
                    toggles.push((pe.local_offset, pe.slot));
                }
            } else {
                self.pending_enc.push(pe);
            }
        }
        for &(_, slot) in &toggles {
            apply_slot_to_totals(&mut self.deltas, &mut self.parity, slot);
        }
        // The left-hand confirmed candidates never need the toggles: a
        // left-side enclosure resolving here lacked right context until now,
        // so it sits after every left-confirmed candidate (which had ≥ k
        // characters following it), and right-side toggles lie at or beyond
        // the seam entirely.

        // Right-hand confirmed candidates: rebase against the left snapshot,
        // then apply the toggles positioned before them. Confirmed offsets
        // grow strictly across the seam, so appending keeps the order.
        for c in &other.boundaries {
            let mut c = rebase_candidate(c, left_len, &left_deltas, left_parity);
            adjust_for_toggles(
                &mut c.local_depths,
                &mut c.local_parity,
                c.local_offset,
                &toggles,
            );
            self.boundaries.push(c);
        }

        // Pending candidates: adjust, then resolve or keep. Newly confirmed
        // ones sit near the seam and are merged into the sorted candidates.
        let prior_pending = std::mem::take(&mut self.pending);
        let right_pending = other
            .pending
            .iter()
            .map(|p| rebase_pending_candidate(p, left_len, &left_deltas, left_parity));
        for mut pc in prior_pending.into_iter().chain(right_pending) {
            adjust_for_toggles(
                &mut pc.local_depths,
                &mut pc.local_parity,
                pc.local_offset,
                &toggles,
            );
            if self.window_available(pc.local_offset) {
                let (window, pos) = resolve_window(joint_str, joint_start, pc.local_offset);
                if let Judgment::Boundary(flags) = judge.judge(window, pos, pc.kind) {
                    insert_sorted(&mut self.boundaries, pc.confirm(flags));
                }
            } else {
                self.pending.push(pc);
            }
        }
    }

    /// Resolves the remaining pending items with the knowledge that no more
    /// text is coming: missing left context resolves against the start of
    /// text, missing right context against the end (the window is clipped
    /// instead of completed). Called once by the driver after the final
    /// combine; sits outside the monoid. Enclosures resolve before
    /// candidates, as in combine.
    pub(crate) fn resolve_edges<J: Judge>(mut self, judge: &J) -> Self {
        let pending_enc = std::mem::take(&mut self.pending_enc);
        let mut toggles = ToggleVec::new();
        for pe in pending_enc {
            let p = pe.local_offset;
            // Pick the buffer whose coverage contains the clipped window:
            // items lacking left context sit within k characters of the text
            // start (window ⊆ head buffer), all others lack right context and
            // sit within k characters of the end (⊆ tail buffer).
            let (buf, buf_start) = if self.chars_before(p) < WINDOW_CHARS {
                (self.head_ctx.as_str(), 0)
            } else {
                let tail_start = self.chunk_len - self.tail_ctx.byte_len();
                debug_assert!(p >= tail_start, "pending enclosure outside both buffers");
                (self.tail_ctx.as_str(), tail_start)
            };
            let (window, pos) = window_around(buf, p - buf_start, WINDOW_CHARS);
            if !judge.suppress_enclosure(window, pos, pe.ch) {
                toggles.push((p, pe.slot));
            }
        }
        if !toggles.is_empty() {
            for &(_, slot) in &toggles {
                apply_slot_to_totals(&mut self.deltas, &mut self.parity, slot);
            }
            for c in &mut self.boundaries {
                adjust_for_toggles(
                    &mut c.local_depths,
                    &mut c.local_parity,
                    c.local_offset,
                    &toggles,
                );
            }
            for pc in &mut self.pending {
                adjust_for_toggles(
                    &mut pc.local_depths,
                    &mut pc.local_parity,
                    pc.local_offset,
                    &toggles,
                );
            }
        }

        let pending = std::mem::take(&mut self.pending);
        for pc in pending {
            let p = pc.local_offset;
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

/// The ±k window of a pending item resolved at a combine, drawn from the
/// joint `tail ++ head` context (byte range starts at `joint_start` in
/// combined coordinates).
fn resolve_window(joint_str: &str, joint_start: usize, p: usize) -> (&str, usize) {
    debug_assert!(
        p >= joint_start && p <= joint_start + joint_str.len(),
        "resolvable pending item must lie inside the joint window"
    );
    let (window, pos) = window_around(joint_str, p - joint_start, WINDOW_CHARS);
    debug_assert_eq!(
        window.chars().count(),
        2 * WINDOW_CHARS,
        "a window resolved at combine is never clipped (Window Availability)"
    );
    (window, pos)
}

/// Rebases a right-hand candidate to the combined state's origin, given the
/// left-hand span's pre-merge totals.
fn rebase_candidate(
    c: &Candidate,
    left_len: usize,
    left_deltas: &DepthVec,
    left_parity: u32,
) -> Candidate {
    Candidate {
        local_offset: c.local_offset + left_len,
        local_depths: rebase_depths(&c.local_depths, left_deltas),
        local_parity: c.local_parity ^ left_parity,
        flags: c.flags,
    }
}

/// Rebases a right-hand pending candidate (see [`rebase_candidate`]).
fn rebase_pending_candidate(
    p: &PendingCandidate,
    left_len: usize,
    left_deltas: &DepthVec,
    left_parity: u32,
) -> PendingCandidate {
    PendingCandidate {
        local_offset: p.local_offset + left_len,
        local_depths: rebase_depths(&p.local_depths, left_deltas),
        local_parity: p.local_parity ^ left_parity,
        kind: p.kind,
    }
}

fn rebase_depths(depths: &DepthVec, left_deltas: &DepthVec) -> DepthVec {
    let len = depths.len().max(left_deltas.len());
    (0..len)
        .map(|i| depths.get(i).copied().unwrap_or(0) + left_deltas.get(i).copied().unwrap_or(0))
        .collect()
}

/// Inserts a candidate into an offset-sorted vector, keeping it sorted.
fn insert_sorted(boundaries: &mut CandidateVec, c: Candidate) {
    let pos = boundaries.partition_point(|b| b.local_offset < c.local_offset);
    boundaries.insert(pos, c);
}

/// Applies a confirmed enclosure's effect to a state's depth/parity totals.
fn apply_slot_to_totals(deltas: &mut DepthVec, parity: &mut u32, slot: EnclosureSlot) {
    match slot {
        EnclosureSlot::Asym { index, delta } => {
            let i = index as usize;
            if deltas.len() <= i {
                deltas.resize(i + 1, 0);
            }
            deltas[i] += i32::from(delta);
        }
        EnclosureSlot::Sym { bit } => *parity ^= 1 << bit,
    }
}

/// Applies every confirmed enclosure positioned before `offset` to one
/// candidate's depth/parity.
fn adjust_for_toggles(
    depths: &mut DepthVec,
    parity: &mut u32,
    offset: usize,
    toggles: &[(usize, EnclosureSlot)],
) {
    for &(q, slot) in toggles {
        if q < offset {
            match slot {
                EnclosureSlot::Asym { index, delta } => {
                    let i = index as usize;
                    if depths.len() <= i {
                        depths.resize(i + 1, 0);
                    }
                    depths[i] += i32::from(delta);
                }
                EnclosureSlot::Sym { bit } => *parity ^= 1 << bit,
            }
        }
    }
}

#[cfg(test)]
mod tests;
