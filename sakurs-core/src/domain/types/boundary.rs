//! Boundary-related value objects

/// Represents a confirmed sentence boundary
#[derive(Clone, Debug, PartialEq)]
pub struct ConfirmedBoundary {
    /// Position in the text
    pub position: usize,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

impl ConfirmedBoundary {
    /// Create a new confirmed boundary
    pub fn new(position: usize, confidence: f32) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&confidence),
            "Confidence must be between 0.0 and 1.0"
        );
        Self {
            position,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    /// Create a high-confidence boundary
    pub fn high_confidence(position: usize) -> Self {
        Self::new(position, 1.0)
    }

    /// Create a medium-confidence boundary
    pub fn medium_confidence(position: usize) -> Self {
        Self::new(position, 0.75)
    }
}

/// Context for abbreviation detection
#[derive(Clone, Debug, Default)]
pub struct AbbreviationContext {
    /// Whether we're currently in an abbreviation
    pub in_abbreviation: bool,
    /// The abbreviation text if any
    pub current_abbreviation: Option<String>,
    /// Position where abbreviation started
    pub start_position: Option<usize>,
}

impl AbbreviationContext {
    /// Create new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking an abbreviation
    pub fn start_abbreviation(&mut self, text: String, position: usize) {
        self.in_abbreviation = true;
        self.current_abbreviation = Some(text);
        self.start_position = Some(position);
    }

    /// End abbreviation tracking
    pub fn end_abbreviation(&mut self) {
        self.in_abbreviation = false;
        self.current_abbreviation = None;
        self.start_position = None;
    }
}
