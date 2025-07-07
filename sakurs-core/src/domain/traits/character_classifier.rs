//! Character classification for sentence boundary detection

/// Classification of characters for boundary detection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterClass {
    /// Sentence-ending punctuation
    SentenceTerminal,
    /// Opening delimiter (quote, parenthesis, etc.)
    DelimiterOpen,
    /// Closing delimiter
    DelimiterClose,
    /// Whitespace character
    Whitespace,
    /// Alphabetic character
    Alphabetic,
    /// Numeric character
    Numeric,
    /// Other punctuation
    OtherPunctuation,
    /// Other character type
    Other,
}

/// Pure character classification logic
pub trait CharacterClassifier: Send + Sync {
    /// Classify a character
    fn classify(&self, ch: char) -> CharacterClass;

    /// Check if character is a sentence terminal
    fn is_sentence_terminal(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::SentenceTerminal)
    }

    /// Check if character is an opening delimiter
    fn is_delimiter_open(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::DelimiterOpen)
    }

    /// Check if character is a closing delimiter
    fn is_delimiter_close(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::DelimiterClose)
    }

    /// Check if character is whitespace
    fn is_whitespace(&self, ch: char) -> bool {
        matches!(self.classify(ch), CharacterClass::Whitespace)
    }

    /// Get the matching closing delimiter for an opening delimiter
    fn get_matching_delimiter(&self, open: char) -> Option<char> {
        match open {
            '(' => Some(')'),
            '[' => Some(']'),
            '{' => Some('}'),
            '"' => Some('"'),
            '\'' => Some('\''),
            '「' => Some('」'),
            '『' => Some('』'),
            '（' => Some('）'),
            '［' => Some('］'),
            '｛' => Some('｝'),
            '〔' => Some('〕'),
            '【' => Some('】'),
            '〈' => Some('〉'),
            '《' => Some('》'),
            _ => None,
        }
    }
}
