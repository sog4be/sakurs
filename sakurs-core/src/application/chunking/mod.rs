//! Text chunking for parallel processing.
//!
//! Chunks are strictly contiguous, borrowed slices of the input snapped
//! forward to UTF-8 character boundaries — no copying, no overlap, no
//! word-boundary search. Deferred judgment makes cut placement irrelevant to
//! correctness (see `docs/DELTA_STACK_ALGORITHM.md`), so the only constraint
//! is slice validity.

/// Splits `text` into contiguous spans of roughly `chunk_size` bytes, each
/// end snapped forward to the next character boundary. `chunk_size` is
/// clamped to at least one byte; the final span may be shorter.
pub(crate) fn chunk_spans(text: &str, chunk_size: usize) -> Vec<&str> {
    let chunk_size = chunk_size.max(1);
    let mut spans = Vec::with_capacity(text.len() / chunk_size + 1);
    let mut start = 0;
    while start < text.len() {
        let mut end = (start + chunk_size).min(text.len());
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        spans.push(&text[start..end]);
        start = end;
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_are_contiguous_and_cover_the_text() {
        let text = "abcdefあいうえおxyz".repeat(7);
        for chunk_size in [1, 2, 3, 5, 16, 64, 1024] {
            let spans = chunk_spans(&text, chunk_size);
            assert_eq!(spans.concat(), text, "chunk_size={chunk_size}");
            assert!(spans.iter().all(|s| !s.is_empty()));
        }
    }

    #[test]
    fn multibyte_boundaries_are_snapped_forward() {
        let text = "あいうえお"; // 3 bytes per char
        let spans = chunk_spans(text, 4); // lands mid-character
        assert_eq!(spans, vec!["あい", "うえ", "お"]);
    }

    #[test]
    fn empty_text_yields_no_spans() {
        assert!(chunk_spans("", 16).is_empty());
    }

    #[test]
    fn zero_chunk_size_is_clamped() {
        assert_eq!(chunk_spans("ab", 0), vec!["a", "b"]);
    }
}
