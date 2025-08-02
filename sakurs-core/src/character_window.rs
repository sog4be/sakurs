//! Character window for efficient context management
//!
//! This module provides O(1) character context access to eliminate
//! the O(n²) performance issues caused by repeated text scanning.

/// Sliding window of characters for efficient context access
///
/// Maintains an 11-character window: [prev_5, prev_4, prev_3, prev_2, prev_1, current, next_1, next_2, next_3, next_4, next_5]
/// All operations are O(1) to ensure linear algorithm performance.
#[derive(Debug, Clone)]
pub struct CharacterWindow {
    /// 11-character sliding window for extended context
    /// Layout: [prev_5, prev_4, prev_3, prev_2, prev_1, current, next_1, next_2, next_3, next_4, next_5]
    /// Indices:   [0]     [1]     [2]     [3]     [4]      [5]      [6]     [7]     [8]     [9]    [10]
    chars: [Option<char>; 11],
    /// Current byte position in text
    byte_pos: usize,
    /// Current character index (0-based)
    char_index: usize,
}

impl CharacterWindow {
    /// Index of the current character in the window
    const CURRENT_POS: usize = 5;

    /// Create new character window at the beginning of text
    pub fn new() -> Self {
        Self {
            chars: [None; 11],
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
    pub fn advance(&mut self, current_char: char, lookahead_chars: &[Option<char>]) {
        // Shift window left by one position (O(1))
        for i in 0..10 {
            self.chars[i] = self.chars[i + 1];
        }

        // Set current character
        self.chars[Self::CURRENT_POS] = Some(current_char);

        // Set lookahead characters (up to 5)
        for (i, &ch) in lookahead_chars.iter().enumerate().take(5) {
            self.chars[Self::CURRENT_POS + 1 + i] = ch;
        }

        // Update positions
        self.byte_pos += current_char.len_utf8();
        self.char_index += 1;
    }

    /// Get character at offset from current position
    pub fn char_at_offset(&self, offset: isize) -> Option<char> {
        let index = (Self::CURRENT_POS as isize + offset) as usize;
        if index < self.chars.len() {
            self.chars[index]
        } else {
            None
        }
    }

    /// Get character before previous (position - 2)
    pub fn prev_prev_char(&self) -> Option<char> {
        self.chars[Self::CURRENT_POS - 2]
    }

    /// Get previous character (position - 1)
    pub fn prev_char(&self) -> Option<char> {
        self.chars[Self::CURRENT_POS - 1]
    }

    /// Get current character (position)
    pub fn current_char(&self) -> Option<char> {
        self.chars[Self::CURRENT_POS]
    }

    /// Get next character (position + 1)
    pub fn next_char(&self) -> Option<char> {
        self.chars[Self::CURRENT_POS + 1]
    }

    /// Get character after next (position + 2)
    pub fn next_next_char(&self) -> Option<char> {
        self.chars[Self::CURRENT_POS + 2]
    }

    /// Get character at position - n (for abbreviation detection)
    pub fn prev_char_at(&self, n: usize) -> Option<char> {
        if n > 0 && n <= 5 {
            self.chars[Self::CURRENT_POS - n]
        } else {
            None
        }
    }

    /// Get character at position + n (for sentence starter detection)
    pub fn next_char_at(&self, n: usize) -> Option<char> {
        if n > 0 && n <= 5 {
            self.chars[Self::CURRENT_POS + n]
        } else {
            None
        }
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
    /// Note: Now supports up to 5 positions ahead with extended window
    pub fn peek_char(&self, n: usize) -> Option<char> {
        if n <= 5 {
            self.char_at_offset(n as isize)
        } else {
            None
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

        // Advance with 'a', lookahead is ['b', 'c']
        window.advance('a', &[Some('b'), Some('c')]);
        assert_eq!(window.prev_char(), None);
        assert_eq!(window.current_char(), Some('a'));
        assert_eq!(window.next_char(), Some('b'));
        assert_eq!(window.next_next_char(), Some('c'));
        assert_eq!(window.char_index(), 1);
        assert!(!window.is_at_start());

        // Advance with 'b', lookahead is ['c', 'd']
        window.advance('b', &[Some('c'), Some('d')]);
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
        window.advance('a', &[Some('b'), Some('c')]);
        window.advance('b', &[Some('c'), Some('d')]);
        window.advance('c', &[Some('d')]);
        window.advance('d', &[]);

        // Window should now contain more history due to 11-char size
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
        window.advance('a', &[]);
        assert_eq!(window.byte_position(), 1);

        // Multi-byte UTF-8 character (3 bytes)
        window.advance('あ', &[]);
        assert_eq!(window.byte_position(), 4);
    }

    #[test]
    fn test_context_helpers() {
        let mut window = CharacterWindow::new();

        window.advance('a', &[Some('\n'), Some('b')]);
        assert!(!window.is_at_line_start());
        assert!(window.next_char_is('\n'));

        window.advance('\n', &[Some('b')]);
        assert!(!window.is_at_line_start()); // Current is '\n', not after it

        window.advance('b', &[]);
        assert!(window.is_at_line_start()); // Previous was '\n'
    }

    #[test]
    fn test_context_triple() {
        let mut window = CharacterWindow::new();

        window.advance('a', &[Some('b'), Some('c'), Some('d')]);
        window.advance('b', &[Some('c'), Some('d')]);

        let (prev, current, next) = window.context_triple();
        assert_eq!(prev, Some('a'));
        assert_eq!(current, Some('b'));
        assert_eq!(next, Some('c'));
        assert_eq!(window.next_next_char(), Some('d'));
    }

    #[test]
    fn test_extended_window() {
        let mut window = CharacterWindow::new();

        // Build "Prof.X" step by step
        window.advance(
            'P',
            &[Some('r'), Some('o'), Some('f'), Some('.'), Some('X')],
        );
        window.advance('r', &[Some('o'), Some('f'), Some('.'), Some('X')]);
        window.advance('o', &[Some('f'), Some('.'), Some('X')]);
        window.advance('f', &[Some('.'), Some('X')]);
        window.advance('.', &[Some('X')]);

        // At the dot, we should be able to see back to 'P'
        assert_eq!(window.current_char(), Some('.'));
        assert_eq!(window.prev_char_at(1), Some('f'));
        assert_eq!(window.prev_char_at(2), Some('o'));
        assert_eq!(window.prev_char_at(3), Some('r'));
        assert_eq!(window.prev_char_at(4), Some('P'));
        assert_eq!(window.prev_char_at(5), None); // Beyond window

        // And forward
        assert_eq!(window.next_char_at(1), Some('X'));

        // Test char_at_offset
        assert_eq!(window.char_at_offset(-4), Some('P'));
        assert_eq!(window.char_at_offset(-3), Some('r'));
        assert_eq!(window.char_at_offset(-2), Some('o'));
        assert_eq!(window.char_at_offset(-1), Some('f'));
        assert_eq!(window.char_at_offset(0), Some('.'));
        assert_eq!(window.char_at_offset(1), Some('X'));
    }
}
