//! Streaming processing for handling very large files with bounded memory usage
//!
//! This module provides streaming-based text processing that can handle files
//! larger than available RAM by processing them in chunks while maintaining
//! sentence boundary accuracy.

pub mod buffer;
pub mod detector;
pub mod state;
pub mod strategy;

pub use strategy::StreamingStrategy;
