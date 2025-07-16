use crate::domain::{
    enclosure::EnclosureChar,
    enclosure_suppressor::EnclosureSuppressor,
    error::DomainError,
    language::{
        config::{get_language_config, LanguageConfig},
        rules::{
            AbbreviationTrie, EllipsisRules, EnclosureMap, PatternContext, Suppressor,
            TerminatorRules,
        },
        traits::{
            AbbreviationResult, BoundaryContext, BoundaryDecision, LanguageRules, QuotationContext,
            QuotationDecision,
        },
    },
    BoundaryFlags,
};

/// Extract the next word from the following context (first alphabetic sequence)
fn extract_next_word(following_context: &str) -> Option<String> {
    let mut chars = following_context.chars().peekable();

    // Skip whitespace
    while let Some(ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }

    // Extract word characters
    let mut word = String::new();
    while let Some(ch) = chars.peek() {
        if ch.is_alphabetic() {
            word.push(chars.next().unwrap());
        } else {
            break;
        }
    }

    if word.is_empty() {
        None
    } else {
        Some(word)
    }
}

/// Check if a word is a sentence starter (typically capitalized)
/// This is a more conservative check that considers common sentence starters
fn is_sentence_starter(word: &str) -> bool {
    if let Some(first_char) = word.chars().next() {
        // Must be uppercase and not a common proper noun pattern
        if first_char.is_uppercase() {
            // Common sentence starters
            let common_starters = [
                "He",
                "She",
                "It",
                "They",
                "We",
                "I",
                "You",
                "This",
                "That",
                "These",
                "Those",
                "The",
                "A",
                "An",
                "There",
                "Here",
                "Now",
                "Then",
                "However",
                "But",
                "And",
                "So",
                "Therefore",
                "Moreover",
                "Furthermore",
                "Meanwhile",
                "Finally",
                "Also",
                "Additionally",
                "Nevertheless",
                "Nonetheless",
                "Consequently",
                "Hence",
                "Thus",
                "What",
                "When",
                "Where",
                "Why",
                "How",
                "Who",
                "Which",
                "Whose",
                "Whom",
            ];

            // Check if it's a common sentence starter
            common_starters.contains(&word)
        } else {
            false
        }
    } else {
        false
    }
}

/// Configurable language rules based on TOML configuration
pub struct ConfigurableLanguageRules {
    /// Language metadata
    code: String,
    name: String,

    /// Rule components
    terminator_rules: TerminatorRules,
    ellipsis_rules: EllipsisRules,
    abbreviation_trie: AbbreviationTrie,
    enclosure_map: EnclosureMap,
    suppressor: Suppressor,
}

impl ConfigurableLanguageRules {
    /// Create language rules from a language code
    pub fn from_code(code: &str) -> Result<Self, DomainError> {
        let config = get_language_config(code)?;
        Self::from_config(config)
    }

    /// Create language rules from configuration
    pub fn from_config(config: &LanguageConfig) -> Result<Self, DomainError> {
        // Build terminator rules
        let terminator_rules = TerminatorRules::new(
            config.terminators.chars.clone(),
            config
                .terminators
                .patterns
                .iter()
                .map(|p| (p.pattern.clone(), p.name.clone()))
                .collect(),
        );

        // Build ellipsis rules
        let ellipsis_rules = EllipsisRules::new(
            config.ellipsis.treat_as_boundary,
            config.ellipsis.patterns.clone(),
            config
                .ellipsis
                .context_rules
                .iter()
                .map(|r| (r.condition.clone(), r.boundary))
                .collect(),
            config
                .ellipsis
                .exceptions
                .iter()
                .map(|e| (e.regex.clone(), e.boundary))
                .collect(),
        )
        .map_err(DomainError::InvalidLanguageRules)?;

        // Build abbreviation trie
        let abbreviation_trie = AbbreviationTrie::from_categories(
            config.abbreviations.categories.clone(),
            false, // Case insensitive by default
        );

        // Build enclosure map
        let enclosure_map = EnclosureMap::new(
            config
                .enclosures
                .pairs
                .iter()
                .map(|p| (p.open, p.close, p.symmetric))
                .collect(),
        );

        // Build suppressor with regex patterns if available
        let suppressor = if config.suppression.regex_patterns.is_empty() {
            Suppressor::new(
                config
                    .suppression
                    .fast_patterns
                    .iter()
                    .map(|p| (p.char, p.line_start, p.before.clone(), p.after.clone()))
                    .collect(),
            )
        } else {
            Suppressor::with_regex_patterns(
                config
                    .suppression
                    .fast_patterns
                    .iter()
                    .map(|p| (p.char, p.line_start, p.before.clone(), p.after.clone()))
                    .collect(),
                config
                    .suppression
                    .regex_patterns
                    .iter()
                    .map(|p| (p.pattern.clone(), p.description.clone()))
                    .collect(),
            )
            .map_err(|e| {
                DomainError::InvalidLanguageRules(format!("Invalid regex pattern: {}", e))
            })?
        };

        Ok(Self {
            code: config.metadata.code.clone(),
            name: config.metadata.name.clone(),
            terminator_rules,
            ellipsis_rules,
            abbreviation_trie,
            enclosure_map,
            suppressor,
        })
    }
}

impl LanguageRules for ConfigurableLanguageRules {
    fn detect_sentence_boundary(&self, context: &BoundaryContext) -> BoundaryDecision {
        let ch = context.boundary_char;

        // Check if it's an ellipsis pattern
        if self.ellipsis_rules.is_ellipsis_pattern(context) {
            return self.ellipsis_rules.evaluate_boundary(context);
        }

        // Check if current character is part of an incomplete ellipsis pattern
        // For example, if we're at the first or second '.' of "..."
        if ch == '.' {
            // Check if we're followed by more dots (incomplete ellipsis)
            if let Some(next_ch) = context.following_context.chars().next() {
                if next_ch == '.' {
                    // We're part of an ellipsis pattern but not at the end
                    return BoundaryDecision::NotBoundary;
                }
            }

            // Check if we're preceded by dots (middle of ellipsis)
            if let Some(prev_ch) = context.preceding_context.chars().last() {
                if prev_ch == '.' {
                    // Check if we're followed by another dot
                    if let Some(next_ch) = context.following_context.chars().next() {
                        if next_ch == '.' {
                            // We're in the middle of "..."
                            return BoundaryDecision::NotBoundary;
                        }
                    }
                    // We might be at the end of ellipsis, let the ellipsis rules handle it
                }
            }
        }

        // Check if it's a terminator
        if self.terminator_rules.is_terminator(ch) {
            // Create pattern context for pattern matching
            let pattern_context = PatternContext {
                text: &context.text,
                position: context.position,
                current_char: ch,
                next_char: context.following_context.chars().next(),
                prev_char: context.preceding_context.chars().last(),
            };

            // Check for terminator patterns
            if let Some(_pattern) = self.terminator_rules.match_pattern(&pattern_context) {
                // Pattern terminators are always strong boundaries
                return BoundaryDecision::Boundary(BoundaryFlags::STRONG);
            }

            // Check if current character is part of a future pattern
            // This prevents creating boundaries at the first character of multi-character patterns
            let next_char = context.following_context.chars().next();
            if let Some(next) = next_char {
                // Check if current + next forms a known pattern
                let potential_pattern = format!("{}{}", ch, next);
                for (pattern_str, _) in self.terminator_rules.patterns() {
                    if pattern_str == &potential_pattern {
                        // This is the first character of a pattern, don't create boundary
                        return BoundaryDecision::NotBoundary;
                    }
                }
            }

            // Check for abbreviations
            // context.position is the byte offset BEFORE the terminator (period)
            // We check if there's an abbreviation ending at this position
            let abbr_result = self.process_abbreviation(&context.text, context.position);
            if abbr_result.is_abbreviation {
                // Check if the next word is a sentence starter (capitalized word)
                if let Some(next_word) = extract_next_word(&context.following_context) {
                    if is_sentence_starter(&next_word) {
                        // Abbreviation followed by sentence starter - create boundary
                        return BoundaryDecision::Boundary(BoundaryFlags::WEAK);
                    }
                }
                return BoundaryDecision::NotBoundary;
            }

            // Default terminator evaluation
            return self.terminator_rules.evaluate_single_terminator(context);
        }

        BoundaryDecision::NotBoundary
    }

    fn process_abbreviation(&self, text: &str, position: usize) -> AbbreviationResult {
        // Look for abbreviations ending before the period
        // position is the position of the period, so we check at position - 1
        if position > 0 {
            if let Some(abbr_match) = self.abbreviation_trie.find_at_position(text, position - 1) {
                // Check for word boundary at the start of the abbreviation
                let abbr_start = position - abbr_match.length;

                // Simple word boundary check: the character before the abbreviation should not be alphanumeric
                let has_word_boundary = if abbr_start == 0 {
                    true // Start of text is a valid boundary
                } else {
                    // Check the character before the abbreviation
                    text.chars()
                        .nth(abbr_start.saturating_sub(1))
                        .map(|ch| !ch.is_alphanumeric())
                        .unwrap_or(true)
                };

                if has_word_boundary {
                    AbbreviationResult {
                        is_abbreviation: true,
                        length: abbr_match.length,
                        confidence: 1.0, // High confidence for exact matches
                    }
                } else {
                    // Abbreviation found but not at word boundary
                    AbbreviationResult {
                        is_abbreviation: false,
                        length: 0,
                        confidence: 0.0,
                    }
                }
            } else {
                AbbreviationResult {
                    is_abbreviation: false,
                    length: 0,
                    confidence: 0.0,
                }
            }
        } else {
            AbbreviationResult {
                is_abbreviation: false,
                length: 0,
                confidence: 0.0,
            }
        }
    }

    fn handle_quotation(&self, _context: &QuotationContext) -> QuotationDecision {
        // Basic quotation handling - can be enhanced later
        QuotationDecision::QuoteStart
    }

    fn language_code(&self) -> &str {
        &self.code
    }

    fn language_name(&self) -> &str {
        &self.name
    }

    fn get_enclosure_char(&self, ch: char) -> Option<EnclosureChar> {
        self.enclosure_map.get_enclosure_char(ch)
    }

    fn get_enclosure_type_id(&self, ch: char) -> Option<usize> {
        self.enclosure_map.get_type_id(ch)
    }

    fn enclosure_type_count(&self) -> usize {
        self.enclosure_map.type_count()
    }

    fn enclosure_suppressor(&self) -> Option<&dyn EnclosureSuppressor> {
        Some(&self.suppressor)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_configurable_rules_creation() {
        // This test will work once we have the config files in place
        // For now, we'll test that the structure compiles correctly
    }
}
