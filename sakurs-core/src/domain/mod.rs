//! Domain layer for the Delta-Stack Monoid algorithm
//!
//! This module contains the mathematical foundations and language-specific
//! logic for parallel sentence boundary detection using monoid structures.

pub mod language;
pub mod monoid;
pub mod state;

pub use language::*;
pub use monoid::*;
pub use state::*;
