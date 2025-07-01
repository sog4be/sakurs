//! Domain layer for the Delta-Stack Monoid algorithm
//!
//! This module contains the pure mathematical foundations for parallel
//! sentence boundary detection using monoid structures.

pub mod monoid;
pub mod state;

pub use monoid::*;
pub use state::*;
