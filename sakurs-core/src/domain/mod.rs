//! Domain layer for the Delta-Stack Monoid algorithm
//!
//! This module contains the mathematical foundations and language-specific
//! logic for parallel sentence boundary detection using monoid structures.

pub mod enclosure;
pub mod language;
pub mod monoid;
pub mod prefix_sum;
pub mod reduce;
pub mod state;
pub mod traits;
pub mod types;

pub use enclosure::*;
pub use language::*;
pub use monoid::*;
pub use prefix_sum::*;
pub use reduce::*;
pub use state::*;
pub use traits::*;
pub use types::*;
