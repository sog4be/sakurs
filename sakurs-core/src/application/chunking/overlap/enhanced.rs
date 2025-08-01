//! Enhanced overlap chunk processing with improved abbreviation handling
//!
//! This module extends the basic overlap chunking to better handle
//! abbreviations and sentence boundaries at chunk boundaries.

use crate::{
    application::chunking::{overlap::types::*, TextChunk},
    domain::{
        enhanced_abbreviation::{ChunkBoundaryAnalyzer, ExtendedAbbreviationContext},
        language::LanguageRules,
        types::{AbbreviationState, PartialState},
    },
};
use std::collections::HashMap;

/// Enhanced overlap processor with abbreviation tracking
pub struct EnhancedOverlapProcessor {
    /// Base overlap size
    overlap_size: usize,
    
    /// Extended context size for abbreviation detection
    extended_context_size: usize,
    
    /// Chunk boundary analyzer
    boundary_analyzer: ChunkBoundaryAnalyzer,
    
    /// Minimum confidence for edge boundaries
    min_edge_confidence: f32,
}

impl EnhancedOverlapProcessor {
    /// Create a new enhanced processor
    pub fn new(overlap_size: usize) -> Self {
        Self {
            overlap_size,
            extended_context_size: 50, // Larger context for abbreviations
            boundary_analyzer: ChunkBoundaryAnalyzer::new(overlap_size),
            min_edge_confidence: 0.8,
        }
    }
    
    /// Process chunk transition with enhanced abbreviation handling
    pub fn process_chunk_transition(
        &self,
        left_chunk: &TextChunk,
        right_chunk: &TextChunk,
        left_state: &PartialState,
        right_state: &PartialState,
        language_rules: &dyn LanguageRules,
    ) -> ChunkTransitionState {
        // Extract overlap regions
        let left_suffix = self.extract_suffix(left_chunk);
        let right_prefix = self.extract_prefix(right_chunk);
        
        // Build extended context
        let extended_context = format!("{}{}", left_suffix, right_prefix);
        
        // Analyze abbreviation patterns
        let abbreviation_analysis = self.analyze_abbreviation_transition(
            &left_suffix,
            &right_prefix,
            &left_state.abbreviation,
            &right_state.abbreviation,
            language_rules,
        );
        
        // Process boundaries in overlap region
        let boundary_adjustments = self.process_overlap_boundaries(
            left_state,
            right_state,
            &extended_context,
            &abbreviation_analysis,
            language_rules,
        );
        
        // Create a new transition state with enhanced abbreviation info
        let mut state = ChunkTransitionState::new(right_chunk.index);
        
        // Add abbreviation pattern if detected
        if abbreviation_analysis.has_cross_chunk_abbreviation {
            state.ending_patterns.push(PartialPattern {
                text: abbreviation_analysis.abbreviation_text.clone().unwrap_or_default(),
                expected_continuations: vec![],
                pattern_type: PatternType::Abbreviation,
            });
            state.add_pattern_confidence(
                "cross_chunk_abbreviation".to_string(),
                abbreviation_analysis.confidence,
            );
        }
        
        state
    }
    
    /// Extract suffix from chunk for overlap analysis
    fn extract_suffix(&self, chunk: &TextChunk) -> String {
        let content_len = chunk.content.len();
        let start = content_len.saturating_sub(self.extended_context_size);
        
        if start < content_len {
            // Find valid UTF-8 boundary
            let mut actual_start = start;
            while actual_start < content_len && !chunk.content.is_char_boundary(actual_start) {
                actual_start += 1;
            }
            chunk.content[actual_start..].to_string()
        } else {
            String::new()
        }
    }
    
    /// Extract prefix from chunk for overlap analysis
    fn extract_prefix(&self, chunk: &TextChunk) -> String {
        let end = self.extended_context_size.min(chunk.content.len());
        
        if end > 0 {
            // Find valid UTF-8 boundary
            let mut actual_end = end;
            while actual_end > 0 && !chunk.content.is_char_boundary(actual_end) {
                actual_end -= 1;
            }
            chunk.content[..actual_end].to_string()
        } else {
            String::new()
        }
    }
    
    /// Analyze abbreviation patterns at chunk transition
    fn analyze_abbreviation_transition(
        &self,
        left_suffix: &str,
        right_prefix: &str,
        left_abbr: &AbbreviationState,
        right_abbr: &AbbreviationState,
        language_rules: &dyn LanguageRules,
    ) -> AbbreviationTransitionAnalysis {
        let mut analysis = AbbreviationTransitionAnalysis::default();
        
        // Check basic cross-chunk abbreviation
        if left_abbr.dangling_dot && right_abbr.head_alpha {
            analysis.has_cross_chunk_abbreviation = true;
            
            // Try to extract the full abbreviation text
            if let Some(dot_pos) = left_suffix.rfind('.') {
                let abbr_start = left_suffix[..dot_pos]
                    .rfind(|c: char| !c.is_alphabetic())
                    .map(|p| p + 1)
                    .unwrap_or(0);
                
                if abbr_start < dot_pos {
                    let abbr_text = &left_suffix[abbr_start..=dot_pos];
                    analysis.abbreviation_text = Some(abbr_text.to_string());
                    
                    // Check if it's a known abbreviation
                    let check_result = language_rules.process_abbreviation(
                        left_suffix,
                        dot_pos,
                    );
                    analysis.confidence = check_result.confidence;
                }
            }
        }
        
        // Check for multi-period patterns spanning chunks
        let combined = format!("{}{}", left_suffix, right_prefix);
        if combined.contains("...") || combined.matches('.').count() > 2 {
            analysis.has_multi_period_pattern = true;
        }
        
        // Check if right chunk starts with a sentence starter
        if let Some(first_word) = &right_abbr.first_word {
            analysis.following_word = Some(first_word.clone());
            // The sentence starter check is done by the language rules
        }
        
        analysis
    }
    
    /// Process boundaries in overlap region
    fn process_overlap_boundaries(
        &self,
        left_state: &PartialState,
        right_state: &PartialState,
        extended_context: &str,
        abbreviation_analysis: &AbbreviationTransitionAnalysis,
        _language_rules: &dyn LanguageRules,
    ) -> Vec<BoundaryAdjustment> {
        let mut adjustments = Vec::new();
        
        // Check boundaries near the end of left chunk
        for candidate in &left_state.boundary_candidates {
            let near_end = left_state.chunk_length.saturating_sub(candidate.local_offset) 
                < self.overlap_size;
            
            if near_end && abbreviation_analysis.has_cross_chunk_abbreviation {
                // This boundary might be invalid due to cross-chunk abbreviation
                adjustments.push(BoundaryAdjustment {
                    original_position: candidate.local_offset,
                    adjusted_position: None, // Remove the boundary
                    reason: format!(
                        "Cross-chunk abbreviation detected: {:?}",
                        abbreviation_analysis.abbreviation_text
                    ),
                });
            }
        }
        
        // Check boundaries near the start of right chunk
        for candidate in &right_state.boundary_candidates {
            if candidate.local_offset < self.overlap_size {
                // Boundary near chunk start might need adjustment
                if abbreviation_analysis.has_cross_chunk_abbreviation {
                    // Keep this boundary if it's after a sentence starter
                    if abbreviation_analysis.following_word.is_some() {
                        // Boundary is valid - abbreviation followed by sentence starter
                        continue;
                    } else {
                        // Remove boundary - part of abbreviation
                        adjustments.push(BoundaryAdjustment {
                            original_position: candidate.local_offset,
                            adjusted_position: None,
                            reason: "Part of cross-chunk abbreviation".to_string(),
                        });
                    }
                }
            }
        }
        
        adjustments
    }
}

/// Analysis results for abbreviation transition
#[derive(Debug, Default)]
struct AbbreviationTransitionAnalysis {
    /// Whether a cross-chunk abbreviation was detected
    has_cross_chunk_abbreviation: bool,
    
    /// The detected abbreviation text (if any)
    abbreviation_text: Option<String>,
    
    /// Confidence score for the abbreviation
    confidence: f32,
    
    /// Whether a multi-period pattern was detected
    has_multi_period_pattern: bool,
    
    /// The word following the abbreviation (if any)
    following_word: Option<String>,
}

/// Extended chunk transition state with abbreviation info
#[derive(Debug, Clone)]
pub struct EnhancedChunkTransitionState {
    /// Base transition state
    pub base_state: ChunkTransitionState,
    
    /// Extended abbreviation context for left chunk
    pub left_abbreviation_context: ExtendedAbbreviationContext,
    
    /// Extended abbreviation context for right chunk
    pub right_abbreviation_context: ExtendedAbbreviationContext,
    
    /// Merged boundary candidates after resolution
    pub resolved_boundaries: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enhanced_overlap_processor() {
        let processor = EnhancedOverlapProcessor::new(20);
        
        let left_chunk = TextChunk {
            content: "This is Dr.".to_string(),
            start_offset: 0,
            end_offset: 11,
            has_prefix_overlap: false,
            has_suffix_overlap: true,
            index: 0,
            total_chunks: 2,
        };
        
        let right_chunk = TextChunk {
            content: " Smith arrived.".to_string(),
            start_offset: 11,
            end_offset: 26,
            has_prefix_overlap: true,
            has_suffix_overlap: false,
            index: 1,
            total_chunks: 2,
        };
        
        assert_eq!(processor.extract_suffix(&left_chunk), "This is Dr.");
        assert_eq!(processor.extract_prefix(&right_chunk), " Smith arrived.");
    }
    
    #[test]
    fn test_abbreviation_transition_analysis() {
        let processor = EnhancedOverlapProcessor::new(20);
        
        let left_abbr = AbbreviationState {
            dangling_dot: true,
            head_alpha: false,
            first_word: None,
        };
        
        let right_abbr = AbbreviationState {
            dangling_dot: false,
            head_alpha: true,
            first_word: Some("Smith".to_string()),
        };
        
        let analysis = processor.analyze_abbreviation_transition(
            "Dr.",
            " Smith",
            &left_abbr,
            &right_abbr,
            &crate::domain::language::MockLanguageRules::english(),
        );
        
        assert!(analysis.has_cross_chunk_abbreviation);
        assert_eq!(analysis.following_word, Some("Smith".to_string()));
    }
}