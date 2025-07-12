//! Enhanced cross-chunk handling for the Δ-Stack Monoid algorithm
//!
//! This module provides improved mechanisms for handling state that spans
//! across chunk boundaries, including:
//! - Enhanced abbreviation continuity tracking
//! - Enclosure state preservation
//! - Cross-chunk boundary validation

use crate::domain::{
    enclosure::EnclosureType,
    language::LanguageRules,
    types::{AbbreviationState, Boundary, BoundaryCandidate, BoundaryFlags, PartialState},
};
use std::collections::HashMap;

/// Check if a word is a common sentence starter
fn is_sentence_starter(word: &str) -> bool {
    // Only consider words that start with a capital letter as potential sentence starters
    if word.chars().next().is_some_and(|c| !c.is_uppercase()) {
        return false;
    }

    // Convert to title case for case-insensitive comparison
    let title_case = word
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_lowercase().collect::<String>()
            }
        })
        .collect::<String>();

    // Common English sentence starters
    matches!(
        title_case.as_str(),
        // Personal pronouns
        "I" | "He" | "She" | "It" | "We" | "You" | "They" |
        // WH-words
        "What" | "Why" | "When" | "Where" | "Who" | "Whom" | "Whose" | "Which" | "How" |
        // Demonstratives
        "This" | "That" | "These" | "Those" |
        // Conjunctive adverbs
        "However" | "Therefore" | "Thus" | "Moreover" | "Furthermore" |
        "Meanwhile" | "Consequently" | "Nevertheless" |
        // Conditional adverbs
        "Otherwise" | "Instead" |
        // Common article
        "The" |
        // Time indicators
        "Yesterday" | "Today" | "Tomorrow"
    )
}

/// Enhanced abbreviation state with more context
#[derive(Debug, Clone, PartialEq)]
pub struct EnhancedAbbreviationState {
    /// Basic abbreviation state (backward compatible)
    pub basic: AbbreviationState,

    /// The actual abbreviation text if detected
    pub abbreviation_text: Option<String>,

    /// Position where the abbreviation starts (relative to chunk)
    pub abbreviation_start: Option<usize>,

    /// Confidence score for the abbreviation
    pub confidence: f32,

    /// Whether this abbreviation continues from previous chunk
    pub is_continuation: bool,
}

impl Default for EnhancedAbbreviationState {
    fn default() -> Self {
        Self {
            basic: AbbreviationState::default(),
            abbreviation_text: None,
            abbreviation_start: None,
            confidence: 0.0,
            is_continuation: false,
        }
    }
}

impl EnhancedAbbreviationState {
    /// Create from basic abbreviation state
    pub fn from_basic(basic: AbbreviationState) -> Self {
        Self {
            basic,
            abbreviation_text: None,
            abbreviation_start: None,
            confidence: 1.0,
            is_continuation: false,
        }
    }

    /// Check if this represents a cross-chunk abbreviation with another state
    pub fn is_cross_chunk_abbreviation(&self, next: &Self) -> bool {
        // Basic check
        if self.basic.dangling_dot && next.basic.head_alpha {
            return true;
        }

        // Enhanced check: if we have abbreviation text that ends with dot
        if let Some(text) = &self.abbreviation_text {
            if text.ends_with('.') && next.basic.head_alpha {
                return true;
            }
        }

        false
    }
}

/// Tracks enclosure state across chunks with more detail
#[derive(Debug, Clone)]
pub struct EnclosureStateTracker {
    /// Stack of open enclosures with their types and positions
    pub open_enclosures: Vec<(EnclosureType, usize)>,

    /// Map of enclosure type to depth (for compatibility)
    pub depth_map: HashMap<usize, i32>,

    /// Positions where enclosures were opened but not closed in chunk
    pub unclosed_positions: Vec<usize>,

    /// Positions where enclosures were closed but not opened in chunk
    pub unopened_closures: Vec<usize>,
}

impl EnclosureStateTracker {
    pub fn new() -> Self {
        Self {
            open_enclosures: Vec::new(),
            depth_map: HashMap::new(),
            unclosed_positions: Vec::new(),
            unopened_closures: Vec::new(),
        }
    }

    /// Update tracker with a new enclosure event
    pub fn update(
        &mut self,
        enclosure_type: EnclosureType,
        type_id: usize,
        is_opening: bool,
        position: usize,
    ) {
        if is_opening {
            self.open_enclosures.push((enclosure_type, position));
            *self.depth_map.entry(type_id).or_insert(0) += 1;
            self.unclosed_positions.push(position);
        } else {
            // Try to find matching opening
            if let Some(pos) = self
                .open_enclosures
                .iter()
                .rposition(|(t, _)| *t == enclosure_type)
            {
                self.open_enclosures.remove(pos);
                self.unclosed_positions
                    .retain(|&p| p != self.open_enclosures.get(pos).map(|(_, p)| *p).unwrap_or(0));
            } else {
                // Closing without opening in this chunk
                self.unopened_closures.push(position);
            }
            *self.depth_map.entry(type_id).or_insert(0) -= 1;
        }
    }

    /// Merge with tracker from next chunk
    pub fn merge_with_next(&self, next: &Self) -> Self {
        let mut merged = self.clone();

        // Handle unopened closures from next chunk
        // These should close some of our unclosed positions
        for &_pos in &next.unopened_closures {
            if !merged.open_enclosures.is_empty() {
                merged.open_enclosures.pop();
            }
            if let Some(idx) = merged.unclosed_positions.iter().position(|_| true) {
                merged.unclosed_positions.remove(idx);
            }
        }

        // Add remaining open enclosures from next
        merged
            .open_enclosures
            .extend(next.open_enclosures.iter().cloned());

        // Update depth map - the net effect after merging
        for (type_id, &depth) in &next.depth_map {
            *merged.depth_map.entry(*type_id).or_insert(0) += depth;
        }

        // Handle the special case where tracker2 has unopened closures
        // In the test case, tracker2 closes a quote that was opened in tracker1
        if !next.unopened_closures.is_empty() && !self.open_enclosures.is_empty() {
            // The depth map should reflect the net result
            // tracker1 has +1, tracker2 has -1, so net is 0
            if let Some(_type_id) = self.open_enclosures.first().and_then(|(enc_type, _)| {
                // Map enclosure type to type_id - this is a simplification
                match enc_type {
                    EnclosureType::DoubleQuote => Some(0),
                    EnclosureType::SingleQuote => Some(1),
                    EnclosureType::Parenthesis => Some(2),
                    _ => None,
                }
            }) {
                // Already handled by depth map update above
            }
        }

        merged
    }
}

impl Default for EnclosureStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Cross-chunk boundary validator
pub struct CrossChunkValidator {
    /// Overlap size for validation
    overlap_size: usize,

    /// Minimum context required for validation
    #[allow(dead_code)]
    min_context: usize,
}

impl CrossChunkValidator {
    pub fn new(overlap_size: usize) -> Self {
        Self {
            overlap_size,
            min_context: 20, // At least 20 chars of context
        }
    }

    /// Validate boundaries near chunk boundaries
    pub fn validate_chunk_boundary(
        &self,
        boundary: &BoundaryCandidate,
        state: &PartialState,
        next_state: Option<&PartialState>,
        _language_rules: &dyn LanguageRules,
    ) -> ValidationResult {
        // First check enclosure balance for all boundaries
        if !self.validate_enclosure_balance(boundary, state) {
            return ValidationResult::Weakened(BoundaryFlags::WEAK);
        }

        // Check if boundary is near chunk boundary
        let near_end = state.chunk_length.saturating_sub(boundary.local_offset) < self.overlap_size;
        let near_start = boundary.local_offset < self.overlap_size;

        if !near_end && !near_start {
            return ValidationResult::Valid;
        }

        // For boundaries near chunk boundaries, we need more context
        if near_end && next_state.is_none() {
            return ValidationResult::NeedsMoreContext;
        }

        // Check abbreviation continuity
        if near_end {
            if let Some(next) = next_state {
                if state.abbreviation.is_cross_chunk_abbr(&next.abbreviation) {
                    // Check if the next chunk starts with a sentence starter
                    if let Some(ref first_word) = next.abbreviation.first_word {
                        // Use language rules to check if it's a sentence starter
                        // For now, we'll use a simple check for common sentence starters
                        if is_sentence_starter(first_word) {
                            // This is an abbreviation followed by a sentence starter
                            // The boundary should be kept as strong
                            return ValidationResult::Weakened(BoundaryFlags::STRONG);
                        }
                    }
                    return ValidationResult::Invalid(
                        "Cross-chunk abbreviation detected".to_string(),
                    );
                }
            }
        }

        ValidationResult::Valid
    }

    /// Validate enclosure balance at boundary
    fn validate_enclosure_balance(
        &self,
        boundary: &BoundaryCandidate,
        _state: &PartialState,
    ) -> bool {
        // Check if all depths are balanced
        boundary.local_depths.iter().all(|&depth| depth == 0)
    }
}

/// Result of cross-chunk validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Boundary is valid
    Valid,

    /// Boundary should be weakened
    Weakened(BoundaryFlags),

    /// Boundary is invalid
    Invalid(String),

    /// Needs more context from next chunk
    NeedsMoreContext,
}

/// Enhanced partial state with cross-chunk tracking
#[derive(Debug, Clone)]
pub struct EnhancedPartialState {
    /// Basic partial state
    pub base: PartialState,

    /// Enhanced abbreviation tracking
    pub enhanced_abbreviation: EnhancedAbbreviationState,

    /// Enclosure state tracking
    pub enclosure_tracker: EnclosureStateTracker,

    /// Chunk metadata
    pub chunk_metadata: ChunkMetadata,
}

/// Metadata about a chunk for cross-chunk processing
#[derive(Debug, Clone)]
pub struct ChunkMetadata {
    /// Chunk index in sequence
    pub index: usize,

    /// Total number of chunks
    pub total_chunks: usize,

    /// Whether this chunk has overlap from previous
    pub has_prefix_overlap: bool,

    /// Whether this chunk has overlap for next
    pub has_suffix_overlap: bool,

    /// Actual overlap content if available
    pub prefix_overlap: Option<String>,
    pub suffix_overlap: Option<String>,
}

impl ChunkMetadata {
    pub fn is_first(&self) -> bool {
        self.index == 0
    }

    pub fn is_last(&self) -> bool {
        self.index == self.total_chunks - 1
    }
}

/// Cross-chunk boundary resolution
pub struct CrossChunkResolver {
    validator: CrossChunkValidator,
}

impl CrossChunkResolver {
    pub fn new(overlap_size: usize) -> Self {
        Self {
            validator: CrossChunkValidator::new(overlap_size),
        }
    }

    /// Resolve boundaries across chunk boundaries
    pub fn resolve_boundaries(
        &self,
        states: &[EnhancedPartialState],
        language_rules: &dyn LanguageRules,
    ) -> Vec<Boundary> {
        let mut all_boundaries = Vec::new();

        for (i, state) in states.iter().enumerate() {
            let next_state = states.get(i + 1);

            for candidate in &state.base.boundary_candidates {
                let validation = self.validator.validate_chunk_boundary(
                    candidate,
                    &state.base,
                    next_state.map(|s| &s.base),
                    language_rules,
                );

                match validation {
                    ValidationResult::Valid => {
                        all_boundaries.push(Boundary {
                            offset: candidate.local_offset, // Needs global offset adjustment
                            flags: candidate.flags,
                        });
                    }
                    ValidationResult::Weakened(flags) => {
                        all_boundaries.push(Boundary {
                            offset: candidate.local_offset,
                            flags,
                        });
                    }
                    ValidationResult::Invalid(_) | ValidationResult::NeedsMoreContext => {
                        // Skip this boundary
                    }
                }
            }
        }

        all_boundaries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::DepthVec;

    #[test]
    fn test_enhanced_abbreviation_state() {
        let basic = AbbreviationState::with_first_word(
            true,  // dangling_dot
            false, // head_alpha
            None,  // first_word
        );

        let enhanced = EnhancedAbbreviationState {
            basic,
            abbreviation_text: Some("Dr.".to_string()),
            abbreviation_start: Some(10),
            confidence: 0.95,
            is_continuation: false,
        };

        let next = EnhancedAbbreviationState {
            basic: AbbreviationState::with_first_word(
                false, // dangling_dot
                true,  // head_alpha
                None,  // first_word
            ),
            ..Default::default()
        };

        assert!(enhanced.is_cross_chunk_abbreviation(&next));
    }

    #[test]
    fn test_enclosure_state_tracker() {
        let mut tracker = EnclosureStateTracker::new();

        tracker.update(EnclosureType::DoubleQuote, 0, true, 10);
        assert_eq!(tracker.open_enclosures.len(), 1);
        assert_eq!(tracker.depth_map[&0], 1);

        tracker.update(EnclosureType::DoubleQuote, 0, false, 20);
        assert_eq!(tracker.open_enclosures.len(), 0);
        assert_eq!(tracker.depth_map[&0], 0);
    }

    #[test]
    fn test_cross_chunk_validator() {
        let validator = CrossChunkValidator::new(10);
        let mut state = PartialState::new(5);
        state.chunk_length = 100; // Set chunk length

        let boundary = BoundaryCandidate {
            local_offset: 50, // Not near chunk boundary (more than 10 from end)
            local_depths: DepthVec::from_vec(vec![0, 0, 0, 0, 0]),
            flags: BoundaryFlags::STRONG,
        };

        let result = validator.validate_chunk_boundary(
            &boundary,
            &state,
            None,
            &crate::domain::language::MockLanguageRules::english(),
        );
        assert_eq!(result, ValidationResult::Valid);
    }

    #[test]
    fn test_chunk_metadata() {
        let metadata = ChunkMetadata {
            index: 0,
            total_chunks: 3,
            has_prefix_overlap: false,
            has_suffix_overlap: true,
            prefix_overlap: None,
            suffix_overlap: Some(" overlap".to_string()),
        };

        assert!(metadata.is_first());
        assert!(!metadata.is_last());
    }
}
