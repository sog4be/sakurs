//! DFA-based pattern matcher for efficient multi-period abbreviation detection
//!
//! This module implements a deterministic finite automaton (DFA) for recognizing
//! patterns like "U.S.A.", "Ph.D.", etc. with O(1) state transitions.

use std::collections::HashMap;

/// DFA state identifier
type StateId = u8;

/// DFA transition on character class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CharClass {
    Letter,      // Alphabetic character
    Period,      // Period character
    Whitespace,  // Whitespace
    Other,       // Any other character
}

/// DFA state with transitions
#[derive(Debug, Clone)]
struct DfaState {
    /// Transitions to next states based on character class
    transitions: HashMap<CharClass, StateId>,
    /// Whether this state accepts (indicates pattern match)
    is_accepting: bool,
    /// Pattern position if accepting
    pattern_position: Option<usize>,
}

/// DFA for multi-period abbreviation patterns
pub struct AbbreviationDfa {
    /// States indexed by ID
    states: Vec<DfaState>,
    /// Start state
    start_state: StateId,
}

impl AbbreviationDfa {
    /// Create a new DFA for multi-period abbreviation detection
    pub fn new() -> Self {
        // Build DFA that recognizes patterns like:
        // - Single letter + period (e.g., "A.")
        // - Letter + period + letter + period (e.g., "U.S.")
        // - Letter + period + letter + period + letter + period (e.g., "U.S.A.")
        // - Two letters + period (e.g., "Ph.")
        // - Two letters + period + letter + period (e.g., "Ph.D.")
        
        let mut states = Vec::new();
        let mut transitions = HashMap::new();
        
        // State 0: Start state
        transitions.insert(CharClass::Letter, 1);
        transitions.insert(CharClass::Period, 0);
        transitions.insert(CharClass::Whitespace, 0);
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: false,
            pattern_position: None,
        });
        
        // State 1: After first letter
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 2);
        transitions.insert(CharClass::Period, 3);
        transitions.insert(CharClass::Whitespace, 0);
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: false,
            pattern_position: None,
        });
        
        // State 2: After two letters
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 0); // Reset on 3+ letters
        transitions.insert(CharClass::Period, 4);
        transitions.insert(CharClass::Whitespace, 0);
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: false,
            pattern_position: None,
        });
        
        // State 3: After single letter + period (accepting)
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 5);
        transitions.insert(CharClass::Period, 0);
        transitions.insert(CharClass::Whitespace, 3); // Stay in accepting state
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: true,
            pattern_position: Some(0), // At the period
        });
        
        // State 4: After two letters + period (accepting)
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 6);
        transitions.insert(CharClass::Period, 0);
        transitions.insert(CharClass::Whitespace, 4); // Stay in accepting state
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: true,
            pattern_position: Some(0), // At the period
        });
        
        // State 5: After letter + period + letter
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 0); // Reset on 2+ letters
        transitions.insert(CharClass::Period, 7);
        transitions.insert(CharClass::Whitespace, 0);
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: false,
            pattern_position: None,
        });
        
        // State 6: After two letters + period + letter
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 0); // Reset on 2+ letters
        transitions.insert(CharClass::Period, 8);
        transitions.insert(CharClass::Whitespace, 0);
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: false,
            pattern_position: None,
        });
        
        // State 7: After letter + period + letter + period (accepting)
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 9);
        transitions.insert(CharClass::Period, 0);
        transitions.insert(CharClass::Whitespace, 7); // Stay in accepting state
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: true,
            pattern_position: Some(1), // At second period
        });
        
        // State 8: After two letters + period + letter + period (accepting)
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 0);
        transitions.insert(CharClass::Period, 0);
        transitions.insert(CharClass::Whitespace, 8); // Stay in accepting state
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: true,
            pattern_position: Some(1), // At second period
        });
        
        // State 9: After letter + period + letter + period + letter
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 0); // Reset on 2+ letters
        transitions.insert(CharClass::Period, 10);
        transitions.insert(CharClass::Whitespace, 0);
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: false,
            pattern_position: None,
        });
        
        // State 10: After letter + period + letter + period + letter + period (accepting)
        let mut transitions = HashMap::new();
        transitions.insert(CharClass::Letter, 0);
        transitions.insert(CharClass::Period, 0);
        transitions.insert(CharClass::Whitespace, 10); // Stay in accepting state
        transitions.insert(CharClass::Other, 0);
        states.push(DfaState {
            transitions,
            is_accepting: true,
            pattern_position: Some(2), // At third period
        });
        
        Self {
            states,
            start_state: 0,
        }
    }
    
    /// Classify a character for DFA transitions
    #[inline]
    fn classify_char(ch: char) -> CharClass {
        if ch.is_alphabetic() {
            CharClass::Letter
        } else if ch == '.' {
            CharClass::Period
        } else if ch.is_whitespace() {
            CharClass::Whitespace
        } else {
            CharClass::Other
        }
    }
    
    /// Check if we're in a multi-period abbreviation pattern
    /// Returns (is_pattern, period_position) where period_position indicates
    /// which period in the pattern we're at (0 = first, 1 = second, etc.)
    pub fn is_multi_period_pattern(&self, text: &str, position: usize) -> (bool, Option<usize>) {
        if position == 0 || position >= text.len() {
            return (false, None);
        }
        
        // We'll scan backwards from the position to find pattern start
        let mut state = self.start_state;
        let mut chars_before: Vec<char> = Vec::new();
        
        // Collect characters before position (up to reasonable limit)
        let start = position.saturating_sub(10); // Max pattern length
        for ch in text[start..position].chars() {
            chars_before.push(ch);
        }
        
        // Process characters through DFA
        for &ch in &chars_before {
            let char_class = Self::classify_char(ch);
            if let Some(&next_state) = self.states[state as usize].transitions.get(&char_class) {
                state = next_state;
            } else {
                state = self.start_state;
            }
        }
        
        // Check if we're at current position (should be a period)
        if position < text.len() {
            if let Some(ch) = text[position..].chars().next() {
                if ch == '.' {
                    // Process the period
                    let char_class = CharClass::Period;
                    if let Some(&next_state) = self.states[state as usize].transitions.get(&char_class) {
                        state = next_state;
                        
                        // Check if this state accepts
                        let state_info = &self.states[state as usize];
                        if state_info.is_accepting {
                            // Now check if pattern continues forward
                            let mut forward_state = state;
                            let mut can_continue = false;
                            
                            // Look ahead for continuation pattern
                            for ch in text[position + 1..].chars().take(5) {
                                let char_class = Self::classify_char(ch);
                                if let Some(&next) = self.states[forward_state as usize].transitions.get(&char_class) {
                                    forward_state = next;
                                    if char_class == CharClass::Letter && forward_state != self.start_state {
                                        can_continue = true;
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                            
                            return (can_continue, state_info.pattern_position);
                        }
                    }
                }
            }
        }
        
        (false, None)
    }
}

impl Default for AbbreviationDfa {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dfa_single_letter_abbreviation() {
        let dfa = AbbreviationDfa::new();
        
        // Test "U.S.A."
        let text = "U.S.A.";
        assert_eq!(dfa.is_multi_period_pattern(text, 1), (true, Some(0))); // After U.
        assert_eq!(dfa.is_multi_period_pattern(text, 3), (true, Some(1))); // After U.S.
        assert_eq!(dfa.is_multi_period_pattern(text, 5), (false, Some(2))); // After U.S.A.
    }
    
    #[test]
    fn test_dfa_two_letter_abbreviation() {
        let dfa = AbbreviationDfa::new();
        
        // Test "Ph.D."
        let text = "Ph.D.";
        assert_eq!(dfa.is_multi_period_pattern(text, 2), (true, Some(0))); // After Ph.
        assert_eq!(dfa.is_multi_period_pattern(text, 4), (false, Some(1))); // After Ph.D.
    }
    
    #[test]
    fn test_dfa_no_pattern() {
        let dfa = AbbreviationDfa::new();
        
        // Test regular sentence
        let text = "Hello world. This is a test.";
        assert_eq!(dfa.is_multi_period_pattern(text, 11), (false, None)); // After "world."
    }
}