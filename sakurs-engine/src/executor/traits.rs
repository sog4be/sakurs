//! Executor trait for pluggable execution strategies

use crate::error::Result;
use sakurs_core::{Boundary, LanguageRules};

/// Pluggable execution strategy for sentence boundary detection
pub trait Executor: Send + Sync {
    /// Process text and return sentence boundaries
    /// 
    /// # Arguments
    /// * `text` - The input text to process
    /// * `rules` - Language-specific rules for boundary detection
    /// 
    /// # Returns
    /// Vector of boundaries sorted by byte offset
    fn process(
        &self,
        text: &str,
        rules: &dyn LanguageRules,
    ) -> Result<Vec<Boundary>>;
    
    /// Get a human-readable name for this executor
    fn name(&self) -> &'static str;
    
    /// Check if this executor supports parallel processing
    fn is_parallel(&self) -> bool {
        false
    }
}