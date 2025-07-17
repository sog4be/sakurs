use crate::domain::enclosure::{EnclosureChar, EnclosureType};
use std::collections::HashMap;

/// Enclosure pair definition
#[derive(Debug, Clone)]
pub struct EnclosurePairDef {
    pub open: char,
    pub close: char,
    pub symmetric: bool,
    pub type_id: usize,
}

/// Maps characters to their enclosure types and manages type IDs
#[derive(Debug, Clone)]
pub struct EnclosureMap {
    /// Character to enclosure info mapping
    char_map: HashMap<char, EnclosureInfo>,
    /// Total number of enclosure types
    type_count: usize,
    /// Pairs for reference
    pairs: Vec<EnclosurePairDef>,
}

#[derive(Debug, Clone)]
struct EnclosureInfo {
    enclosure_type: EnclosureType,
    is_opening: bool,
    type_id: usize,
    is_symmetric: bool,
}

impl EnclosureMap {
    /// Create a new enclosure map from configuration pairs
    pub fn new(pairs: Vec<(char, char, bool)>) -> Self {
        let mut char_map = HashMap::new();
        let mut pair_defs = Vec::new();

        // Assign type IDs based on array order
        for (idx, (open, close, symmetric)) in pairs.into_iter().enumerate() {
            let enclosure_type = Self::determine_enclosure_type(open, close);

            if symmetric {
                // For symmetric quotes, the open/close behavior is context-dependent
                char_map.insert(
                    open,
                    EnclosureInfo {
                        enclosure_type,
                        is_opening: true, // Default to opening for symmetric quotes
                        type_id: idx,
                        is_symmetric: true,
                    },
                );

                if open != close {
                    char_map.insert(
                        close,
                        EnclosureInfo {
                            enclosure_type,
                            is_opening: true, // Also defaults to opening for symmetric quotes
                            type_id: idx,
                            is_symmetric: true,
                        },
                    );
                }
            } else {
                // For asymmetric pairs, opening and closing are distinct
                char_map.insert(
                    open,
                    EnclosureInfo {
                        enclosure_type,
                        is_opening: true,
                        type_id: idx,
                        is_symmetric: false,
                    },
                );

                char_map.insert(
                    close,
                    EnclosureInfo {
                        enclosure_type,
                        is_opening: false,
                        type_id: idx,
                        is_symmetric: false,
                    },
                );
            }

            pair_defs.push(EnclosurePairDef {
                open,
                close,
                symmetric,
                type_id: idx,
            });
        }

        Self {
            char_map,
            type_count: pair_defs.len(),
            pairs: pair_defs,
        }
    }

    /// Get enclosure character information
    pub fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        self.char_map.get(&ch).map(|info| EnclosureChar {
            enclosure_type: info.enclosure_type,
            is_opening: info.is_opening,
            is_symmetric: info.is_symmetric,
        })
    }

    /// Get the type ID for an enclosure character
    pub fn get_type_id(&self, ch: char) -> Option<usize> {
        self.char_map.get(&ch).map(|info| info.type_id)
    }

    /// Get the total number of enclosure types
    pub fn type_count(&self) -> usize {
        self.type_count
    }

    /// Get all pairs (for debugging/inspection)
    pub fn pairs(&self) -> &[EnclosurePairDef] {
        &self.pairs
    }

    /// Determine the enclosure type based on the characters
    fn determine_enclosure_type(open: char, close: char) -> EnclosureType {
        match (open, close) {
            ('"', '"') | ('\u{201C}', '\u{201D}') => EnclosureType::DoubleQuote,
            ('\'', '\'') | ('\u{2018}', '\u{2019}') => EnclosureType::SingleQuote,
            ('(', ')') | ('（', '）') => EnclosureType::Parenthesis,
            ('[', ']') | ('［', '］') => EnclosureType::SquareBracket,
            ('{', '}') => EnclosureType::CurlyBrace,
            ('「', '」') => EnclosureType::JapaneseQuote,
            ('『', '』') => EnclosureType::JapaneseDoubleQuote,
            ('〈', '〉') => EnclosureType::JapaneseAngleBracket,
            ('《', '》') => EnclosureType::JapaneseDoubleAngleBracket,
            ('【', '】') => EnclosureType::JapaneseLenticularBracket,
            ('〔', '〕') => EnclosureType::JapaneseTortoiseShellBracket,
            _ => EnclosureType::Custom(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enclosure_map_creation() {
        let pairs = vec![('(', ')', false), ('[', ']', false), ('"', '"', true)];

        let map = EnclosureMap::new(pairs);

        assert_eq!(map.type_count(), 3);

        // Test parentheses
        let paren_open = map.get_enclosure_char('(').unwrap();
        assert!(paren_open.is_opening);
        assert_eq!(map.get_type_id('('), Some(0));

        let paren_close = map.get_enclosure_char(')').unwrap();
        assert!(!paren_close.is_opening);
        assert_eq!(map.get_type_id(')'), Some(0));

        // Test symmetric quotes
        let quote = map.get_enclosure_char('"').unwrap();
        assert!(quote.is_opening); // Default to opening for symmetric
        assert_eq!(map.get_type_id('"'), Some(2));
    }

    #[test]
    fn test_japanese_enclosures() {
        let pairs = vec![
            ('「', '」', false),
            ('『', '』', false),
            ('（', '）', false),
        ];

        let map = EnclosureMap::new(pairs);

        // Test Japanese quote
        let ja_quote_open = map.get_enclosure_char('「').unwrap();
        assert!(ja_quote_open.is_opening);
        assert!(matches!(
            ja_quote_open.enclosure_type,
            EnclosureType::JapaneseQuote
        ));

        let ja_quote_close = map.get_enclosure_char('」').unwrap();
        assert!(!ja_quote_close.is_opening);
        assert!(matches!(
            ja_quote_close.enclosure_type,
            EnclosureType::JapaneseQuote
        ));

        // Verify they have the same type ID
        assert_eq!(map.get_type_id('「'), map.get_type_id('」'));
    }

    #[test]
    fn test_nonexistent_character() {
        let pairs = vec![('(', ')', false)];
        let map = EnclosureMap::new(pairs);

        assert!(map.get_enclosure_char('x').is_none());
        assert!(map.get_type_id('x').is_none());
    }
}
