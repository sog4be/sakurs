//! Adaptive processing strategies for optimized sentence segmentation
//!
//! This module provides different processing strategies that are automatically
//! selected based on input characteristics for optimal performance.
//!
//! # Overview
//!
//! The adaptive processing system analyzes input text characteristics and selects
//! the most efficient processing strategy:
//!
//! - **Sequential**: For small texts (<75KB) where parallel overhead isn't worth it
//! - **Parallel**: For medium to large texts (≥75KB) with dynamic thread allocation
//!
//! # Example
//!
//! ```rust
//! use sakurs_core::processing::AdaptiveProcessor;
//! use sakurs_core::domain::language::EnglishLanguageRules;
//! use std::sync::Arc;
//!
//! let rules = Arc::new(EnglishLanguageRules::new());
//! let processor = AdaptiveProcessor::new(rules);
//!
//! let text = "This is a test. It has multiple sentences!";
//! let boundaries = processor.process(text).unwrap();
//! ```

pub mod adaptive;
pub mod parallel;
pub mod selector;
pub mod sequential;
pub mod strategy;

pub use adaptive::AdaptiveProcessor;
pub use selector::StrategySelector;
pub use strategy::{InputCharacteristics, ProcessingConfig, ProcessingStrategy};
