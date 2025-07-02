//! CLI command implementations

use clap::Subcommand;

pub mod process;

/// Available CLI commands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Process text files for sentence boundary detection
    Process(process::ProcessArgs),

    /// Analyze text without processing (statistics, benchmarks)
    #[command(visible_alias = "analyse")]
    Analyze {
        /// Show statistics about the text
        #[arg(long)]
        stats: bool,

        /// Run performance benchmark
        #[arg(long)]
        benchmark: bool,

        /// Input file to analyze
        input: String,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommands,
    },

    /// List available components
    List {
        #[command(subcommand)]
        subcommand: ListCommands,
    },
}

/// Configuration subcommands
#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Generate a configuration template
    Generate,

    /// Validate a configuration file
    Validate {
        /// Configuration file to validate
        file: String,
    },
}

/// List subcommands
#[derive(Debug, Subcommand)]
pub enum ListCommands {
    /// List available language rules
    Languages,

    /// List available output formats
    Formats,
}
