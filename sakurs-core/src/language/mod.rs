//! Language-specific rules for sentence boundary detection
//!
//! This module provides a high-performance, data-driven system for
//! language-specific sentence boundary detection rules.

// Sub-modules (only interface is public)
pub mod interface;

#[cfg(feature = "alloc")]
pub(crate) mod config;
#[cfg(feature = "std")]
pub(crate) mod loader;
#[cfg(feature = "alloc")]
pub(crate) mod runtime;
#[cfg(feature = "alloc")]
pub(crate) mod tables;

// Re-export only the public interface
pub use interface::*;

// Re-export loader function for convenience
#[cfg(feature = "std")]
pub use loader::get_rules;

// Re-export simple rules for no_std
#[cfg(not(feature = "std"))]
pub use loader::get_simple_rules;
