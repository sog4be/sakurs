//! Input handling module

pub mod file_reader;
pub mod glob_resolver;

pub use file_reader::FileReader;
pub use glob_resolver::resolve_patterns;
