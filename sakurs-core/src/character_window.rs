//! Character window for efficient context management
//!
//! This module provides O(1) character context access to eliminate
//! the O(n²) performance issues caused by repeated text scanning.

/// Sliding window of characters for efficient context access
///
/// Maintains a 5-character window: [prev_prev, prev, current, next, next_next]
/// All operations are O(1) to ensure linear algorithm performance.
#[derive(Debug, Clone)]
pub struct CharacterWindow {
    /// 5-character sliding window for better lookahead
    chars: [Option<char>; 5],
    /// Current byte position in text
    byte_pos: usize,
    /// Current character index (0-based)
    char_index: usize,
}

impl CharacterWindow {
    /// Create new character window at the beginning of text
    pub fn new() -> Self {
        Self {
            chars: [None; 5],
            byte_pos: 0,
            char_index: 0,
        }
    }

    /// Advance window by one character with lookahead
    ///
    /// This is the core O(1) operation that replaces expensive text scanning.
    ///
    /// # Arguments
    /// * `current_char` - The character at current position
    /// * `next_char` - Optional lookahead character (pos+1)
    /// * `next_next_char` - Optional second lookahead character (pos+2)
    pub fn advance(&mut self, current_char: char, next_char: Option<char>, next_next_char: Option<char>) {
        // Shift window left by one position (O(1))
        self.chars[0] = self.chars[1]; // prev_prev = prev
        self.chars[1] = self.chars[2]; // prev = current
        self.chars[2] = Some(current_char); // current = new_char
        self.chars[3] = next_char; // next = lookahead
        self.chars[4] = next_next_char; // next_next = second lookahead

        // Update positions
        self.byte_pos += current_char.len_utf8();
        self.char_index += 1;
    }

    /// Get character before previous (position - 2)
    pub fn prev_prev_char(&self) -> Option<char> {
        self.chars[0]
    }

    /// Get previous character (position - 1)
    pub fn prev_char(&self) -> Option<char> {
        self.chars[1]
    }

    /// Get current character (position)
    pub fn current_char(&self) -> Option<char> {
        self.chars[2]
    }

    /// Get next character (position + 1)
    pub fn next_char(&self) -> Option<char> {
        self.chars[3]
    }
    
    /// Get character after next (position + 2)
    pub fn next_next_char(&self) -> Option<char> {
        self.chars[4]
    }

    /// Get current byte position
    pub fn byte_position(&self) -> usize {
        self.byte_pos
    }

    /// Get current character index
    pub fn char_index(&self) -> usize {
        self.char_index
    }

    /// Check if we're at the start of text
    pub fn is_at_start(&self) -> bool {
        self.char_index == 0
    }

    /// Get position of current character's start (byte position - char len)
    pub fn current_char_start_pos(&self) -> usize {
        if let Some(ch) = self.current_char() {
            self.byte_pos - ch.len_utf8()
        } else {
            self.byte_pos
        }
    }

    /// Check if previous character is a specific character
    pub fn prev_char_is(&self, target: char) -> bool {
        self.prev_char() == Some(target)
    }

    /// Check if next character is a specific character
    pub fn next_char_is(&self, target: char) -> bool {
        self.next_char() == Some(target)
    }

    /// Check if we're at line start (beginning or after newline)
    pub fn is_at_line_start(&self) -> bool {
        self.is_at_start() || self.prev_char_is('\n')
    }

    /// Get a tuple of (prev, current, next) for common pattern matching
    pub fn context_triple(&self) -> (Option<char>, Option<char>, Option<char>) {
        (self.prev_char(), self.current_char(), self.next_char())
    }
    
    /// Look ahead n positions from current (0 = current, 1 = next, etc.)
    /// Note: Only supports up to 2 positions ahead due to window size
    pub fn peek_char(&self, n: usize) -> Option<char> {
        match n {
            0 => self.current_char(),
            1 => self.next_char(),
            2 => self.next_next_char(),
            _ => None, // Beyond window
        }
    }
}

impl Default for CharacterWindow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_window_basic() {
        let mut window = CharacterWindow::new();

        // Initially empty
        assert_eq!(window.prev_char(), None);
        assert_eq!(window.current_char(), None);
        assert_eq!(window.next_char(), None);
        assert_eq!(window.char_index(), 0);
        assert!(window.is_at_start());

        // Advance with 'a', next is 'b', next_next is 'c'
        window.advance('a', Some('b'), Some('c'));
        assert_eq!(window.prev_char(), None);
        assert_eq!(window.current_char(), Some('a'));
        assert_eq!(window.next_char(), Some('b'));
        assert_eq!(window.next_next_char(), Some('c'));
        assert_eq!(window.char_index(), 1);
        assert!(!window.is_at_start());

        // Advance with 'b', next is 'c', next_next is 'd'
        window.advance('b', Some('c'), Some('d'));
        assert_eq!(window.prev_char(), Some('a'));
        assert_eq!(window.current_char(), Some('b'));
        assert_eq!(window.next_char(), Some('c'));
        assert_eq!(window.next_next_char(), Some('d'));
        assert_eq!(window.char_index(), 2);
    }

    #[test]
    fn test_character_window_sliding() {
        let mut window = CharacterWindow::new();

        // Build: "abcd"
        window.advance('a', Some('b'), Some('c'));
        window.advance('b', Some('c'), Some('d'));
        window.advance('c', Some('d'), None);
        window.advance('d', None, None);

        // Window should be: [b, c, d, None, None]
        assert_eq!(window.prev_prev_char(), Some('b'));
        assert_eq!(window.prev_char(), Some('c'));
        assert_eq!(window.current_char(), Some('d'));
        assert_eq!(window.next_char(), None);
        assert_eq!(window.next_next_char(), None);
    }

    #[test]
    fn test_byte_position_tracking() {
        let mut window = CharacterWindow::new();

        assert_eq!(window.byte_position(), 0);

        // ASCII character
        window.advance('a', None, None);
        assert_eq!(window.byte_position(), 1);

        // Multi-byte UTF-8 character (3 bytes)
        window.advance('あ', None, None);
        assert_eq!(window.byte_position(), 4);
    }

    #[test]
    fn test_context_helpers() {
        let mut window = CharacterWindow::new();

        window.advance('a', Some('\n'), Some('b'));
        assert!(!window.is_at_line_start());
        assert!(window.next_char_is('\n'));

        window.advance('\n', Some('b'), None);
        assert!(!window.is_at_line_start()); // Current is '\n', not after it

        window.advance('b', None, None);
        assert!(window.is_at_line_start()); // Previous was '\n'
    }

    #[test]
    fn test_context_triple() {
        let mut window = CharacterWindow::new();

        window.advance('a', Some('b'), Some('c'));
        window.advance('b', Some('c'), Some('d'));

        let (prev, current, next) = window.context_triple();
        assert_eq!(prev, Some('a'));
        assert_eq!(current, Some('b'));
        assert_eq!(next, Some('c'));
        assert_eq!(window.next_next_char(), Some('d'));
    }
}