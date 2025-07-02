//! Domain layer for the Delta-Stack Monoid algorithm
//!
//! This module contains the mathematical foundations and language-specific
//! logic for parallel sentence boundary detection using monoid structures.

pub mod enclosure;
pub mod language;
pub mod monoid;
pub mod parser;
pub mod state;

pub use enclosure::*;
pub use language::*;
pub use monoid::*;
pub use parser::*;
pub use state::*;
