//! Runtime tables for language rules
//!
//! All tables are allocation-free during lookup and optimized for cache locality.

pub mod abbreviation;
pub mod ellipsis;
pub mod enclosure;
pub mod suppression;
pub mod terminator;

pub use abbreviation::Trie;
pub use ellipsis::EllipsisSet;
pub use enclosure::EncTable;
pub use suppression::Suppresser;
pub use terminator::{DotTable, TermTable};
