//! Sequential execution strategy

use crate::{
    error::Result,
    executor::{ExecutionMode, Executor},
};
use sakurs_core::{emit_push, Boundary, DeltaScanner, LanguageRules};

/// Sequential single-threaded executor
#[derive(Debug, Clone)]
pub struct SequentialExecutor;

impl Executor for SequentialExecutor {
    fn process<R: LanguageRules>(&self, text: &str, rules: &R) -> Result<Vec<Boundary>> {
        let mut boundaries = Vec::new();
        let mut scanner = DeltaScanner::new(rules)?;

        // Process each character
        for ch in text.chars() {
            scanner.step(ch, &mut emit_push(&mut boundaries))?;
        }

        Ok(boundaries)
    }

    fn mode(&self) -> ExecutionMode {
        ExecutionMode::Sequential
    }
}
