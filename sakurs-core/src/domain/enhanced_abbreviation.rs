//! Enhanced abbreviation tracking for improved cross-chunk handling
//!
//! This module extends the basic abbreviation state to include more context
//! that helps with accurate boundary detection at chunk boundaries.

use crate::domain::{
    language::LanguageRules,
    types::{AbbreviationState, BoundaryCandidate, BoundaryFlags},
};

/// Extended abbreviation context for chunk boundaries
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedAbbreviationContext {
    /// Basic abbreviation state
    pub basic_state: AbbreviationState,
    
    /// Text content near the abbreviation (for context)
    pub trailing_context: Option<String>,
    
    /// Position of the last period in the chunk
    pub last_period_position: Option<usize>,
    
    /// Whether we detected a multi-period pattern
    pub has_multi_period_pattern: bool,
    
    /// Confidence score for abbreviation detection
    pub confidence: f32,
}

impl ExtendedAbbreviationContext {
    /// Create a new extended context
    pub fn new() -> Self {
        Self {
            basic_state: AbbreviationState::default(),
            trailing_context: None,
            last_period_position: None,
            has_multi_period_pattern: false,
            confidence: 0.0,
        }
    }
    
    /// Create from basic abbreviation state
    pub fn from_basic(basic: AbbreviationState) -> Self {
        let confidence = if basic.dangling_dot { 0.8 } else { 0.0 };
        Self {
            basic_state: basic,
            trailing_context: None,
            last_period_position: None,
            has_multi_period_pattern: false,
            confidence,
        }
    }
    
    /// Update with period information
    pub fn update_period(&mut self, position: usize, is_multi_period: bool) {
        self.last_period_position = Some(position);
        if is_multi_period {
            self.has_multi_period_pattern = true;
            self.confidence = 0.95; // High confidence for multi-period patterns
        }
    }
    
    /// Set trailing context (last 20-30 chars of chunk)
    pub fn set_trailing_context(&mut self, text: &str, from_position: usize) {
        let context_size = 30;
        let start = from_position.saturating_sub(context_size);
        if start < text.len() && from_position <= text.len() {
            self.trailing_context = Some(text[start..from_position].to_string());
        }
    }
    
    /// Check if this context suggests a cross-chunk abbreviation
    pub fn suggests_cross_chunk_abbreviation(&self) -> bool {
        // If we have a dangling dot with high confidence
        if self.basic_state.dangling_dot && self.confidence > 0.7 {
            return true;
        }
        
        // If we detected a multi-period pattern that might continue
        if self.has_multi_period_pattern && self.basic_state.dangling_dot {
            return true;
        }
        
        false
    }
}

impl Default for ExtendedAbbreviationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced boundary candidate with abbreviation context
#[derive(Debug, Clone)]
pub struct EnhancedBoundaryCandidate {
    /// Basic boundary candidate
    pub candidate: BoundaryCandidate,
    
    /// Extended abbreviation context at this boundary
    pub abbreviation_context: ExtendedAbbreviationContext,
    
    /// Distance from chunk boundary (for prioritization)
    pub distance_from_edge: usize,
}

impl EnhancedBoundaryCandidate {
    /// Create from basic candidate
    pub fn from_candidate(
        candidate: BoundaryCandidate,
        chunk_length: usize,
    ) -> Self {
        // Calculate distance from nearest chunk edge
        let distance_from_start = candidate.local_offset;
        let distance_from_end = chunk_length.saturating_sub(candidate.local_offset);
        let distance_from_edge = distance_from_start.min(distance_from_end);
        
        Self {
            candidate,
            abbreviation_context: ExtendedAbbreviationContext::new(),
            distance_from_edge,
        }
    }
    
    /// Check if this boundary is near a chunk edge
    pub fn is_near_edge(&self, threshold: usize) -> bool {
        self.distance_from_edge < threshold
    }
    
    /// Adjust confidence based on abbreviation context
    pub fn adjusted_confidence(&self) -> f32 {
        let base_confidence = if self.candidate.flags.contains(BoundaryFlags::STRONG) {
            0.9
        } else {
            0.5
        };
        
        // Reduce confidence if abbreviation context suggests continuation
        if self.abbreviation_context.suggests_cross_chunk_abbreviation() {
            base_confidence * 0.3
        } else {
            base_confidence
        }
    }
}

/// Chunk boundary analyzer for improved cross-chunk handling
pub struct ChunkBoundaryAnalyzer {
    /// Size of the overlap region to analyze
    overlap_size: usize,
    
    /// Minimum confidence threshold for boundaries near edges
    edge_confidence_threshold: f32,
}

impl ChunkBoundaryAnalyzer {
    /// Create a new analyzer
    pub fn new(overlap_size: usize) -> Self {
        Self {
            overlap_size,
            edge_confidence_threshold: 0.7,
        }
    }
    
    /// Analyze boundaries near chunk edges
    pub fn analyze_edge_boundaries(
        &self,
        candidates: &[BoundaryCandidate],
        chunk_length: usize,
        language_rules: &dyn LanguageRules,
        chunk_text: &str,
    ) -> Vec<EnhancedBoundaryCandidate> {
        candidates.iter()
            .filter_map(|candidate| {
                let enhanced = EnhancedBoundaryCandidate::from_candidate(
                    candidate.clone(),
                    chunk_length,
                );
                
                // Only enhance boundaries near edges
                if enhanced.is_near_edge(self.overlap_size) {
                    let mut result = enhanced;
                    
                    // Add abbreviation context if near a period
                    if candidate.local_offset > 0 {
                        let check_pos = candidate.local_offset.saturating_sub(1);
                        if check_pos < chunk_text.len() {
                            if let Some(ch) = chunk_text[check_pos..].chars().next() {
                                if ch == '.' {
                                    // Check for abbreviation
                                    let abbr_result = language_rules.process_abbreviation(
                                        chunk_text,
                                        check_pos,
                                    );
                                    
                                    if abbr_result.is_abbreviation {
                                        result.abbreviation_context.basic_state.dangling_dot = true;
                                        result.abbreviation_context.confidence = abbr_result.confidence;
                                        result.abbreviation_context.set_trailing_context(
                                            chunk_text,
                                            candidate.local_offset,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    
                    Some(result)
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Merge boundary information from adjacent chunks
    pub fn merge_chunk_boundaries(
        &self,
        left_boundaries: &[EnhancedBoundaryCandidate],
        right_boundaries: &[EnhancedBoundaryCandidate],
        left_abbr_state: &AbbreviationState,
        right_abbr_state: &AbbreviationState,
    ) -> Vec<BoundaryCandidate> {
        let mut merged = Vec::new();
        
        // Check for cross-chunk abbreviation pattern
        let has_cross_chunk_abbr = left_abbr_state.dangling_dot && right_abbr_state.head_alpha;
        
        // Process left chunk boundaries
        for boundary in left_boundaries {
            if boundary.is_near_edge(self.overlap_size) && has_cross_chunk_abbr {
                // Skip boundaries that are likely part of cross-chunk abbreviation
                if boundary.adjusted_confidence() < self.edge_confidence_threshold {
                    continue;
                }
            }
            merged.push(boundary.candidate.clone());
        }
        
        // Process right chunk boundaries (these would need offset adjustment in practice)
        for boundary in right_boundaries {
            if !boundary.is_near_edge(self.overlap_size) || !has_cross_chunk_abbr {
                merged.push(boundary.candidate.clone());
            }
        }
        
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::DepthVec;
    
    #[test]
    fn test_extended_abbreviation_context() {
        let mut ctx = ExtendedAbbreviationContext::new();
        ctx.update_period(100, true);
        
        assert!(ctx.has_multi_period_pattern);
        assert_eq!(ctx.confidence, 0.95);
        assert!(ctx.suggests_cross_chunk_abbreviation());
    }
    
    #[test]
    fn test_enhanced_boundary_candidate() {
        let candidate = BoundaryCandidate {
            local_offset: 95,
            local_depths: DepthVec::from_vec(vec![0]),
            flags: BoundaryFlags::STRONG,
        };
        
        let enhanced = EnhancedBoundaryCandidate::from_candidate(candidate, 100);
        assert_eq!(enhanced.distance_from_edge, 5);
        assert!(enhanced.is_near_edge(10));
    }
    
    #[test]
    fn test_chunk_boundary_analyzer() {
        let analyzer = ChunkBoundaryAnalyzer::new(20);
        
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
        
        let left_boundaries = vec![];
        let right_boundaries = vec![];
        
        let merged = analyzer.merge_chunk_boundaries(
            &left_boundaries,
            &right_boundaries,
            &left_abbr,
            &right_abbr,
        );
        
        assert_eq!(merged.len(), 0); // No boundaries to merge in this test
    }
}