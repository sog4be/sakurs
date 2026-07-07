//! Fixed-capacity context buffers and window helpers for deferred judgment.
//!
//! A partial state carries the first and last [`CONTEXT_CHARS`] characters of
//! the text span it covers, so that the ±[`WINDOW_CHARS`] window of a pending
//! candidate can be reconstructed at the combine that resolves it (see
//! `docs/DELTA_STACK_ALGORITHM.md`, "Window Availability").

/// Number of characters on each side of a candidate that suffices to decide
/// any linguistic rule (the judgment window `k`).
///
/// Language configurations must fit inside this window; configuration loading
/// asserts the requirement so that a config change cannot silently break
/// sequential equivalence.
pub(crate) const WINDOW_CHARS: usize = 32;

/// Capacity of the head/tail context buffers in characters (`2k`).
///
/// A pending candidate can sit up to `k` characters inside a state edge and
/// its window reaches `k` characters further, so the buffers hold twice the
/// judgment window.
pub(crate) const CONTEXT_CHARS: usize = 2 * WINDOW_CHARS;

/// Byte capacity: [`CONTEXT_CHARS`] characters of up to 4 UTF-8 bytes each.
const CONTEXT_BYTES: usize = CONTEXT_CHARS * 4;

/// A fixed-capacity UTF-8 buffer holding up to [`CONTEXT_CHARS`] characters.
#[derive(Clone)]
pub(crate) struct ContextBuf {
    bytes: [u8; CONTEXT_BYTES],
    len: u16,
    chars: u8,
}

impl ContextBuf {
    /// The empty buffer (identity element for composition).
    pub(crate) fn empty() -> Self {
        Self {
            bytes: [0; CONTEXT_BYTES],
            len: 0,
            chars: 0,
        }
    }

    /// Buffer holding the first `min(CONTEXT_CHARS, |text|)` characters.
    pub(crate) fn head_of(text: &str) -> Self {
        let end = fwd_chars(text, 0, CONTEXT_CHARS);
        Self::from_str(&text[..end])
    }

    /// Buffer holding the last `min(CONTEXT_CHARS, |text|)` characters.
    pub(crate) fn tail_of(text: &str) -> Self {
        let start = back_chars(text, text.len(), CONTEXT_CHARS);
        Self::from_str(&text[start..])
    }

    fn from_str(s: &str) -> Self {
        debug_assert!(s.len() <= CONTEXT_BYTES);
        let mut bytes = [0u8; CONTEXT_BYTES];
        bytes[..s.len()].copy_from_slice(s.as_bytes());
        Self {
            bytes,
            len: s.len() as u16,
            chars: s.chars().count() as u8,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len as usize])
            .expect("ContextBuf always holds valid UTF-8")
    }

    pub(crate) fn byte_len(&self) -> usize {
        self.len as usize
    }

    pub(crate) fn char_count(&self) -> usize {
        self.chars as usize
    }

    /// True when the buffer holds its full [`CONTEXT_CHARS`] characters, which
    /// implies it covers only a prefix/suffix of its span rather than all of it.
    fn is_full(&self) -> bool {
        self.char_count() == CONTEXT_CHARS
    }

    /// Head buffer of the concatenated span `left ++ right`, given the head
    /// buffers of both sides. Exact because a non-full head buffer covers its
    /// entire span.
    pub(crate) fn compose_head(left: &Self, right: &Self) -> Self {
        if left.is_full() {
            return left.clone();
        }
        let room = CONTEXT_CHARS - left.char_count();
        let take = fwd_chars(right.as_str(), 0, room);
        let mut out = left.clone();
        out.bytes[left.byte_len()..left.byte_len() + take]
            .copy_from_slice(&right.as_str().as_bytes()[..take]);
        out.len = (left.byte_len() + take) as u16;
        out.chars = (left.char_count() + right.as_str()[..take].chars().count()) as u8;
        out
    }

    /// Tail buffer of the concatenated span `left ++ right`, given the tail
    /// buffers of both sides. Exact because a non-full tail buffer covers its
    /// entire span.
    pub(crate) fn compose_tail(left: &Self, right: &Self) -> Self {
        if right.is_full() {
            return right.clone();
        }
        let room = CONTEXT_CHARS - right.char_count();
        let left_str = left.as_str();
        let start = back_chars(left_str, left_str.len(), room);
        let kept = &left_str[start..];
        let mut bytes = [0u8; CONTEXT_BYTES];
        bytes[..kept.len()].copy_from_slice(kept.as_bytes());
        bytes[kept.len()..kept.len() + right.byte_len()].copy_from_slice(right.as_str().as_bytes());
        Self {
            bytes,
            len: (kept.len() + right.byte_len()) as u16,
            chars: (kept.chars().count() + right.char_count()) as u8,
        }
    }
}

impl PartialEq for ContextBuf {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for ContextBuf {}

impl std::fmt::Debug for ContextBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContextBuf({:?})", self.as_str())
    }
}

/// Byte index `count` characters before `pos` (clamped at the text start).
/// `pos` must lie on a character boundary.
pub(crate) fn back_chars(text: &str, pos: usize, count: usize) -> usize {
    debug_assert!(text.is_char_boundary(pos));
    let mut idx = pos;
    let mut taken = 0;
    while taken < count && idx > 0 {
        idx -= 1;
        while idx > 0 && !text.is_char_boundary(idx) {
            idx -= 1;
        }
        taken += 1;
    }
    idx
}

/// Byte index `count` characters after `pos` (clamped at the text end).
/// `pos` must lie on a character boundary.
pub(crate) fn fwd_chars(text: &str, pos: usize, count: usize) -> usize {
    debug_assert!(text.is_char_boundary(pos));
    match text[pos..].char_indices().nth(count) {
        Some((i, _)) => pos + i,
        None => text.len(),
    }
}

/// The judgment window around a candidate: up to `k` characters before and
/// after byte position `pos`. Returns the window and the candidate's byte
/// offset inside it.
pub(crate) fn window_around(text: &str, pos: usize, k: usize) -> (&str, usize) {
    let start = back_chars(text, pos, k);
    let end = fwd_chars(text, pos, k);
    (&text[start..end], pos - start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_and_tail_of_short_text_cover_everything() {
        let text = "short.";
        let head = ContextBuf::head_of(text);
        let tail = ContextBuf::tail_of(text);
        assert_eq!(head.as_str(), text);
        assert_eq!(tail.as_str(), text);
        assert_eq!(head.char_count(), 6);
    }

    #[test]
    fn head_and_tail_of_long_text_truncate_to_capacity() {
        let text = "x".repeat(200);
        let head = ContextBuf::head_of(&text);
        let tail = ContextBuf::tail_of(&text);
        assert_eq!(head.char_count(), CONTEXT_CHARS);
        assert_eq!(tail.char_count(), CONTEXT_CHARS);
        assert_eq!(head.as_str(), &text[..CONTEXT_CHARS]);
        assert_eq!(tail.as_str(), &text[text.len() - CONTEXT_CHARS..]);
    }

    #[test]
    fn buffers_handle_multibyte_characters() {
        let text = "あ".repeat(100); // 3 bytes per char
        let head = ContextBuf::head_of(&text);
        let tail = ContextBuf::tail_of(&text);
        assert_eq!(head.char_count(), CONTEXT_CHARS);
        assert_eq!(head.byte_len(), CONTEXT_CHARS * 3);
        assert_eq!(tail.as_str(), "あ".repeat(CONTEXT_CHARS));
    }

    #[test]
    fn compose_matches_direct_computation() {
        // Various lengths around the capacity boundary, mixed widths.
        let pieces = ["", "a.", "あい。", &"x".repeat(70), &"うえ".repeat(40)];
        for l in pieces {
            for r in pieces {
                let joined = format!("{l}{r}");
                let head =
                    ContextBuf::compose_head(&ContextBuf::head_of(l), &ContextBuf::head_of(r));
                let tail =
                    ContextBuf::compose_tail(&ContextBuf::tail_of(l), &ContextBuf::tail_of(r));
                assert_eq!(head, ContextBuf::head_of(&joined), "head of {l:?} ++ {r:?}");
                assert_eq!(tail, ContextBuf::tail_of(&joined), "tail of {l:?} ++ {r:?}");
            }
        }
    }

    #[test]
    fn compose_is_associative() {
        let pieces = ["", "ab.", &"y".repeat(50), &"あ".repeat(40), "z"];
        for a in pieces {
            for b in pieces {
                for c in pieces {
                    let ha = ContextBuf::head_of(a);
                    let hb = ContextBuf::head_of(b);
                    let hc = ContextBuf::head_of(c);
                    let left = ContextBuf::compose_head(&ContextBuf::compose_head(&ha, &hb), &hc);
                    let right = ContextBuf::compose_head(&ha, &ContextBuf::compose_head(&hb, &hc));
                    assert_eq!(left, right, "head associativity for {a:?} {b:?} {c:?}");

                    let ta = ContextBuf::tail_of(a);
                    let tb = ContextBuf::tail_of(b);
                    let tc = ContextBuf::tail_of(c);
                    let left = ContextBuf::compose_tail(&ContextBuf::compose_tail(&ta, &tb), &tc);
                    let right = ContextBuf::compose_tail(&ta, &ContextBuf::compose_tail(&tb, &tc));
                    assert_eq!(left, right, "tail associativity for {a:?} {b:?} {c:?}");
                }
            }
        }
    }

    #[test]
    fn window_around_clips_at_text_edges() {
        let text = "abcdef.ghij";
        let pos = 7; // just after the dot
        let (w, p) = window_around(text, pos, 3);
        assert_eq!(w, "ef.ghi");
        assert_eq!(p, 3);

        let (w, p) = window_around(text, 1, 5);
        assert_eq!(w, "abcdef");
        assert_eq!(p, 1);

        let (w, p) = window_around(text, text.len(), 4);
        assert_eq!(w, "ghij");
        assert_eq!(p, 4);
    }

    #[test]
    fn window_around_multibyte() {
        let text = "あいう。えおか";
        let pos = "あいう。".len();
        let (w, p) = window_around(text, pos, 2);
        assert_eq!(w, "う。えお");
        assert_eq!(p, "う。".len());
    }
}
