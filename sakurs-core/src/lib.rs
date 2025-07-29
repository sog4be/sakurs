//! Core Delta-Stack Monoid algorithm
//!
//! This crate provides the mathematical foundations for parallel sentence
//! boundary detection with zero external dependencies. It can operate in
//! no_std environments with alloc support.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod delta_stack;
pub mod error;
pub mod traits;
pub mod types;

// Re-export core types that work in no_std
pub use delta_stack::{DeltaScanner, DeltaVec, PartialState, ENCLOSURE_MAX};
pub use error::CoreError;
pub use traits::LanguageRules;
pub use types::{Boundary, BoundaryKind, Class};

// Re-export alloc-dependent functions
#[cfg(feature = "alloc")]
pub use delta_stack::{emit_commit_if_depth0, emit_push, reduce_deltas, run, scan_chunk};
