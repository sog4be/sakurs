//! Domain layer for the Delta-Stack Monoid algorithm
//!
//! This module contains the mathematical foundations and language-specific
//! logic for parallel sentence boundary detection using monoid structures.

pub mod enclosure;
pub mod enclosure_suppressor;
pub mod error;
pub mod language;
pub(crate) mod state;
pub mod types;

// Re-export from other modules
pub use enclosure::*;
pub use types::*;

// Re-export language module (contains BoundaryContext, BoundaryDecision)
pub use language::*;
