//! Pure boundary detection logic

use crate::domain::state::PartialState;

/// Context for boundary analysis
#[derive(Clone, Debug)]
pub struct BoundaryContext {
    /// Text content around the potential boundary
    pub text_before: String,
    pub text_after: String,
    /// Position in the original text
    pub position: usize,
    /// Character at the boundary position
    pub boundary_char: char,
    /// Current enclosure depth
    pub enclosure_depth: i32,
}

/// Represents a potential sentence boundary (trait version)
#[derive(Clone, Debug)]
pub struct BoundaryCandidateInfo {
    /// Position in the text
    pub position: usize,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Context at this position
    pub context: BoundaryContext,
    /// Type of boundary marker
    pub marker_type: BoundaryMarkerType,
}

/// Types of boundary markers
#[derive(Clone, Debug, PartialEq)]
pub enum BoundaryMarkerType {
    /// Period (.)
    Period,
    /// Question mark (?)
    Question,
    /// Exclamation mark (!)
    Exclamation,
    /// Other punctuation
    Other(char),
}

/// Decision about a boundary candidate
#[derive(Clone, Debug, PartialEq)]
pub enum BoundaryDecision {
    /// Confirmed sentence boundary
    Confirmed { confidence: f32 },
    /// Rejected as boundary
    Rejected { reason: RejectionReason },
    /// Needs more context to decide
    Pending,
}

/// Reasons for rejecting a boundary
#[derive(Clone, Debug, PartialEq)]
pub enum RejectionReason {
    /// Part of an abbreviation
    Abbreviation,
    /// Inside quotes or parentheses
    InsideEnclosure,
    /// Not followed by appropriate spacing/capitalization
    InvalidFollowing,
    /// Other language-specific reason
    LanguageSpecific(String),
}

/// Pure boundary detection logic
pub trait BoundaryAnalyzer: Send + Sync {
    /// Analyze a potential boundary position
    fn analyze_candidate(&self, context: &BoundaryContext) -> BoundaryCandidateInfo;

    /// Evaluate a boundary candidate with accumulated state
    fn evaluate_boundary(
        &self,
        candidate: &BoundaryCandidateInfo,
        state: &PartialState,
    ) -> BoundaryDecision;

    /// Check if a character could be a sentence boundary
    fn is_potential_boundary(&self, ch: char) -> bool {
        matches!(ch, '.' | '?' | '!' | '。' | '？' | '！' | '…' | '‥')
    }
}
