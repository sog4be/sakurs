//! Language configuration for sentence boundary detection.
//!
//! Languages are defined by TOML configuration files (see
//! `configs/languages/`) that are compiled into the judgment oracles of the
//! deferred-judgment pipeline (`domain::state`). The [`config`] module
//! provides the configuration schema, the embedded bundled languages, and
//! loading from external files.

pub mod config;
