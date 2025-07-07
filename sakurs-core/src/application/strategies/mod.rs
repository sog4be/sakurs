//! Text processing strategies
//!
//! This module contains various strategies for processing text based on
//! input characteristics and system resources.

pub mod adaptive;
pub mod parallel;
pub mod selector;
pub mod sequential;
pub mod streaming;
pub mod traits;

pub use adaptive::AdaptiveStrategy;
pub use parallel::ParallelStrategy;
pub use selector::StrategySelector;
pub use sequential::SequentialStrategy;
pub use streaming::{BoundaryDetector, StreamingBuffer, StreamingState, StreamingStrategy};
pub use traits::{
    InputCharacteristics, ProcessingConfig, ProcessingStrategy, StrategyInput, StrategyOutput,
    StrategySelection, StrategyType,
};
