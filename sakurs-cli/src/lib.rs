//! Sakurs CLI library
//!
//! This library provides the command-line interface for the Sakurs
//! sentence boundary detection system.

pub mod commands;
pub mod config;
pub mod error;
pub mod input;
pub mod output;
pub mod progress;

pub use error::{CliError, CliResult};
