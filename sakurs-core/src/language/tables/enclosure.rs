//! Enclosure (bracket/quote) mapping with O(1) lookup
//!
//! Handles paired delimiters with support for symmetric quotes.

use crate::language::interface::EnclosureInfo;
use std::collections::HashMap;

/// Enclosure character mapping table
#[derive(Debug, Clone)]
pub struct EncTable {
    /// Direct character -> EnclosureInfo mapping
    map: HashMap<char, EnclosureInfo>,
    /// Maximum type_id in use
    max_type_id: u8,
}

impl EncTable {
    /// Create from pairs configuration
    pub fn new(pairs: Vec<(char, char, bool)>) -> Self {
        let mut map = HashMap::new();
        let mut type_id = 0u8;

        for (open, close, symmetric) in pairs {
            if type_id == 255 {
                // Skip if we exceed u8 capacity
                break;
            }

            if symmetric {
                // Symmetric quotes - both chars map to same type with symmetric flag
                map.insert(
                    open,
                    EnclosureInfo {
                        type_id,
                        delta: 0, // Context-dependent
                        symmetric: true,
                    },
                );
                if open != close {
                    map.insert(
                        close,
                        EnclosureInfo {
                            type_id,
                            delta: 0,
                            symmetric: true,
                        },
                    );
                }
            } else {
                // Asymmetric pairs
                map.insert(
                    open,
                    EnclosureInfo {
                        type_id,
                        delta: 1,
                        symmetric: false,
                    },
                );
                map.insert(
                    close,
                    EnclosureInfo {
                        type_id,
                        delta: -1,
                        symmetric: false,
                    },
                );
            }

            type_id += 1;
        }

        Self {
            map,
            max_type_id: type_id.saturating_sub(1),
        }
    }

    /// Look up enclosure info for character
    #[inline]
    pub fn get(&self, ch: char) -> Option<EnclosureInfo> {
        self.map.get(&ch).copied()
    }

    /// Get maximum type ID
    pub fn max_type_id(&self) -> u8 {
        self.max_type_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asymmetric_pairs() {
        let table = EncTable::new(vec![
            ('(', ')', false),
            ('[', ']', false),
            ('{', '}', false),
        ]);

        // Opening brackets
        let info = table.get('(').unwrap();
        assert_eq!(info.type_id, 0);
        assert_eq!(info.delta, 1);
        assert!(!info.symmetric);

        // Closing brackets
        let info = table.get(')').unwrap();
        assert_eq!(info.type_id, 0);
        assert_eq!(info.delta, -1);
        assert!(!info.symmetric);
    }

    #[test]
    fn test_symmetric_quotes() {
        let table = EncTable::new(vec![('"', '"', true), ('\'', '\'', true)]);

        let info = table.get('"').unwrap();
        assert_eq!(info.type_id, 0);
        assert_eq!(info.delta, 0); // Context-dependent
        assert!(info.symmetric);
    }
}
