# Δ-Stack v2 Design: Deferred Boundary Judgment

Status: **draft for review** (Phase 3 / P0 of the v0.2.0 core redesign)
Scope: state model v2, combine/reduce rules, judgment window derivation, scanner outline.
Non-goals: SIMD, streaming API redesign, new languages, symmetric-quote depth ≥ 2.

## 1. Problem

In v0.1.2 the scan phase asks the language rules to *decide* each boundary while
scanning a chunk (`TextParser::scan_chunk_with_context` →
`LanguageRules::detect_sentence_boundary`). The decision consumes context around
the terminator (preceding/following windows, abbreviation lookback, ellipsis
regexes). When a chunk boundary falls inside that context, the window is
truncated and the decision can differ from the single-chunk run — breaking the
central claim that chunked/parallel processing is bit-exact with sequential
processing. Two tests pin this today:

- `tests/chunk_invariance.rs` (generated-English proptest, `#[ignore]`)
- `tests/chunking_regressions.rs::abbreviation_decision_at_exact_chunk_edge` (`#[ignore]`)

v2 restores the monoid property by the following principle:

> **The scanner never decides.** Every decision is a pure function of a
> fixed-size window around the candidate. Candidates whose window is fully
> inside the chunk are judged during scan; candidates within `k` characters of
> a chunk edge are carried as *pending* in the partial state and judged at
> `combine`, when the missing context arrives.

## 2. Judgment window `k`

### 2.1 Context inventory (v0.1.2, en/ja)

Backward = characters needed before the terminator; forward = after it.

| Feature | Where (v0.1.2) | Backward | Forward | Notes |
|---|---|---|---|---|
| Terminator patterns (`!?`, `？！`) | `terminator.rs` | 0 | 1 | longest pattern is 2 chars |
| Decimal-point check | `terminator.rs:128` | 1 | 1 | digits around `.` |
| Ellipsis patterns (`...`, `…`) | `ellipsis.rs:85` | 3 | 0 | longest pattern 3 chars |
| Ellipsis context rules | `ellipsis.rs:175` | 0 | 10 | next non-space char case |
| Ellipsis regex exceptions | `ellipsis.rs:152` | 20 bytes | 20 bytes | fixed ±20-byte slice |
| Abbreviation lookup | `abbreviation.rs:118` | 21 | 0 | reverse scan `take(21)`; longest TOML entry is 5 (`D.D.S`) |
| Sentence starter match | `configurable.rs:336` | 0 | 15 | longest en starter 13 (`Unfortunately`) + space + word end |
| Enclosure suppression | `suppression.rs` | 3 | 3 | fast patterns |
| Line-start detection | `parser/mod.rs:339` | 11 | 0 | only compared against threshold 10, so decidable after 11 chars |
| First-word capture (`AbbreviationState`) | `parser/mod.rs:106` | — | — | computed but never read by reduce; **dropped in v2** |

Two latent defects surface from the inventory (both fixed by v2, both produce
*documented* output diffs):

1. **Long sentence starters can never match.** The starter is extracted from
   `following_context`, which is 10 chars, but the longest configured starter
   is 13 chars. Any starter longer than 9 chars is silently unmatchable in
   v0.1.2.
2. **The ellipsis exception window is byte-based** (±20 bytes), so its reach
   varies with UTF-8 width. v2 switches to a character-based window.

### 2.2 Derivation

```
k_backward = max(21 abbrev, 20 ellipsis-regex, 11 line-start, 10 context, 3 suppression) = 21
k_forward  = max(15 starter, 20 ellipsis-regex, 10 context, 3 suppression, 1 pattern)    = 20
```

**We fix `k = 32` characters, symmetric.** Rationale: covers both directions
with margin for near-term config growth (e.g. a 30-char starter still fits by
raising `k` without structural change), stays cache-friendly (window ≤ 64
chars ≤ 256 bytes), and keeps one constant instead of two.

`k` is asserted, not assumed: loading a language config computes that config's
required window (longest abbreviation + 1, longest starter + 2, regex window,
…) and rejects configs exceeding `k`. A future TOML addition that would break
the invariant fails loudly at load time.

Offsets remain byte offsets; `k` counts characters (UTF-8 safe by
construction).

## 3. State model v2

```rust
/// Fixed-capacity UTF-8 buffer holding up to 2k characters (≤ 256 bytes).
struct ContextBuf { bytes: [u8; 256], len: u8, chars: u8 }

struct PartialState {
    /// Linguistically confirmed candidates. Only the structural
    /// (depth/parity) check remains for reduce.
    boundaries: BoundaryVec,                     // Vec<CandidateBoundary>
    /// Candidates within k chars of a state edge; judged at combine.
    pending: SmallVec<[PendingCandidate; 4]>,
    /// Asymmetric enclosures: (net, min) per type — unchanged.
    deltas: DeltaVec,
    /// Symmetric enclosures: XOR toggle bit per type (Z/2Z monoid).
    parity: u32,
    /// First min(2k, len) characters of the state's text.
    head_ctx: ContextBuf,
    /// Last min(2k, len) characters of the state's text.
    tail_ctx: ContextBuf,
    /// Byte length of the text this state covers.
    chunk_len: usize,
}

struct CandidateBoundary {
    local_offset: usize,     // bytes from state start (position AFTER terminator)
    local_depths: DepthVec,  // asymmetric depths relative to state start
    local_parity: u32,       // symmetric parity relative to state start
    flags: BoundaryFlags,
}

struct PendingCandidate {
    candidate: CandidateBoundary,
    kind: TerminatorKind,    // what to re-judge (terminator char/pattern class)
}
```

Changes vs v0.1.2:

- **`AbbreviationState` is deleted.** `dangling_dot`/`head_alpha` were a
  special-case protocol for exactly one cross-chunk pattern; `first_word` is
  dead (computed, never read). The pending mechanism subsumes all of it.
- **Symmetric enclosures move out of `DeltaVec` into a parity bitset.** v0.1.2
  reuses the depth counters for symmetric quotes and applies `% 2` in reduce;
  that works but overloads the meaning of `net` and makes `min` meaningless
  for those slots. A `u32` XOR is trivially associative, costs nothing to
  combine, and supports up to 32 symmetric types (en uses 2, ja uses 0–2).
  Enclosure type IDs are assigned so symmetric types get parity bits and
  asymmetric types get delta slots.
- **`head_ctx`/`tail_ctx` capacity is `2k`, not `k`.** See §4.3 — a pending
  candidate's ±k window must be reconstructable at the combine that resolves
  it, and the candidate itself can sit up to `k` inside the edge.

### 3.1 Scan-time classification

For a candidate at byte offset `p` in a chunk of byte length `n` (character
distances used for the k-comparisons):

- `chars_before(p) ≥ k` and `chars_after(p) ≥ k` → judge now via `judge()`
  on a zero-copy `&str` window; push to `boundaries` if positive.
- otherwise → push to `pending` (no judgment attempted).

The same rule means a single-chunk run and a multi-chunk run execute the same
code path — the only difference is *when* `judge()` fires, never *on what
input*.

## 4. Combine

`combine(L, R)` produces the state of `text(L) ++ text(R)`:

1. **Rebase R's candidates**: for every candidate (confirmed and pending) of R:
   `local_offset += L.chunk_len`, `local_depths[i] += L.deltas[i].net`,
   `local_parity ^= L.parity`. This keeps the invariant "candidate
   depths/parity are relative to the state's own start".
2. **Deltas**: `deltas[i] = (L.net + R.net, min(L.min, L.net + R.min))` (classic).
3. **Parity**: `parity = L.parity XOR R.parity`.
4. **Resolve pending** (see §4.2): try to judge every pending candidate whose
   ±k window is now fully available; judged candidates move to `boundaries`
   (or are dropped); the rest stay pending.
5. **Contexts**: `head_ctx = first 2k chars of (L.head_ctx ++ R.head_ctx)`
   (the concatenation is exact because `L.head_ctx` is all of L whenever L is
   shorter than 2k); symmetrically
   `tail_ctx = last 2k chars of (L.tail_ctx ++ R.tail_ctx)`.
6. `chunk_len = L.chunk_len + R.chunk_len`.

### 4.1 Pending invariant

> A pending candidate of a state S is always within `k` characters of S's
> start (missing left context) or S's end (missing right context), or both.

True at scan time by §3.1. Preserved by combine: a candidate that stops being
within `k` of both edges after concatenation has, by definition, ≥ k
characters on each side inside the combined text — combine step 4 judges it
then, so it never survives as pending. Conversely a candidate still pending
after step 4 is still within `k` of an edge of the combined state.

### 4.2 Window availability at resolve time

The window needed to judge a pending candidate is `[p − k, p + k]` (character
distances). At the combine where the candidate first has both sides available,
that window is always contained in `L.tail_ctx ++ R.head_ctx`:

- For a right-missing candidate of L: `p` is within `k` of L's end (pending
  invariant), so `[p − k, p]` lies within the last `2k` chars of L
  (`⊆ L.tail_ctx`) and `[p, p + k]` needs at most `k` chars of R
  (`⊆ R.head_ctx`).
- For a left-missing candidate of R: symmetric.
- If the other side is *still* too short (e.g. R shorter than the missing
  forward span), the candidate keeps pending; the invariant plus the `2k`
  capacity guarantee its already-seen side still fits inside the combined
  state's `tail_ctx`/`head_ctx` (worst case: distance from edge < k, window
  reach < k more ⇒ < 2k total).

This is the reason `ContextBuf` holds `2k` characters, not `k`.

### 4.3 Associativity

Sketch (full proof is deliverable R6, to be written into
`DELTA_STACK_ALGORITHM.md`; a proptest checks associativity directly):

- deltas: `(net, min)` combine is the classical associative bracket reduction.
- parity: XOR is associative.
- offsets/depth rebasing: composition of shifts is associative.
- contexts: "concatenate, then take first/last `2k` chars" is associative
  because truncation commutes with further concatenation on the kept side.
- pending resolution: `judge()` is a pure function of the original text's
  substring `[p − k, p + k]`, which is independent of the combine order. For
  any parenthesization, a candidate is judged at the first combine whose
  span covers its window (or survives to edge resolution); §4.2 shows the
  window content available there is exactly that substring. Hence every order
  yields the same judgment for every candidate, and the final `boundaries`
  sets are equal.

Identity element: the empty state (empty contexts, zero deltas/parity,
`chunk_len = 0`, no candidates).

## 5. Edge resolution and reduce

After the last combine (or for a single chunk, immediately after scan), the
driver calls `resolve_edges(state)` once:

- pending candidates missing *left* context are judged with the text-start
  semantics (empty preceding context — BOF),
- pending candidates missing *right* context are judged with the text-end
  semantics (empty following context — EOF).

`resolve_edges` sits outside the monoid (it is knowledge that "no more text is
coming"), mirroring how v0.1.2 already treats the final chunk specially.

The reduce predicate is **unchanged from v0.1.2** (PR #237 semantics):

- asymmetric type `i`: candidate is outside iff `cumulative_net[i] + local_depth[i] ≤ 0`
  (clamped — unmatched closers never suppress the rest of the document),
- symmetric type `b`: outside iff `(cumulative_parity XOR local_parity) bit b == 0`,
- boundary ownership stays `(start, end]` per chunk; global offset =
  prefix offset + `local_offset`.

The prefix phase stays the sequential O(P) scan introduced in v0.1.2, extended
to accumulate `parity` (XOR) alongside deltas.

## 6. Judgment function

```rust
/// Pure. `window` is the candidate's ±k chars (clipped at BOF/EOF);
/// `pos` is the terminator's byte offset inside `window`.
fn judge(window: &str, pos: usize, kind: TerminatorKind, rules: &CompiledRules) -> Judgment;

enum Judgment { Boundary(BoundaryFlags), NotBoundary }
```

- Replaces `detect_sentence_boundary(&BoundaryContext)`;
  `BoundaryContext { text: String, … }` and its per-terminator full-chunk copy
  (`parser/mod.rs:286`) disappear.
- `BoundaryDecision::NeedsMoreContext` disappears: the window is guaranteed
  sufficient by construction (§2.2 assert), except at pending time where
  deferral is structural, not a rule decision.
- All v0.1.2 sub-rules (decimal, ellipsis patterns + context rules + regex
  exceptions, abbreviation via reverse trie, starters, suppression) are
  reformulated as window-relative checks. The TOML schema is **unchanged**.

## 7. Scanner outline (input to 3-B, summarized)

One pass, zero allocation per character:

- character classification via a 128-entry ASCII table + small fallback map
  built from the config (terminator / enclosure-open / enclosure-close /
  symmetric / dot / alpha classes);
- a ring buffer of the last `k` characters (char + byte offset), which also
  answers line-start queries (a newline within the last 11 chars decides;
  none ⇒ not a line start, since 11 < k);
- abbreviation matching as a reverse-trie state machine: advance on
  alphabetic/dot characters, reset otherwise, so "is the token ending here an
  abbreviation?" is O(1) when a `.` is hit — no backward scan;
- terminator hit → §3.1 classification → inline `judge()` on a borrowed
  window or a `pending` push.

Depth/parity updates and candidate collection are the only per-character work
outside terminators, which is what the ≥200 MB/s single-thread target (3-F)
rests on.

## 8. Behavioral diffs vs v0.1.2 (to document in CHANGELOG)

| # | Change | Direction |
|---|---|---|
| 1 | Chunk-edge decisions become identical to single-chunk (the two `#[ignore]` tests pass) | bug fix, the point of v2 |
| 2 | Sentence starters longer than 9 chars start matching (§2.1-1) | more boundaries after abbreviations, en only |
| 3 | Ellipsis exception window: ±20 bytes → ±20 chars (§2.1-2) | regex exceptions apply slightly more often in non-ASCII text |
| 4 | `AbbreviationState`, `BoundaryContext`, `BoundaryDecision::NeedsMoreContext`, `Config.overlap_size` removed; `LanguageRules` reshaped (3-E) | breaking API, collected in v0.2.0 |

TOML language configs are untouched.

## 9. Open questions (for review)

1. `k = 32`: comfortable margin vs. 24 (tighter cache) — any reason to expect
   longer starters/abbreviations in the planned de/fr/es/pt/it configs? 32
   seems safe; raising later is a constant bump plus re-derived assert.
2. Parity capacity `u32` (32 symmetric types per language) — assumed ample.
3. `PendingCandidate` inline capacity (`SmallVec<[_; 4]>`): at most a handful
   of terminators fall within `k` of an edge in practice; 4 avoids heap in the
   common case.
4. Naming: keep `PartialState` (public term in docs) or rename to
   `ChunkState`? Proposal: keep `PartialState`.
