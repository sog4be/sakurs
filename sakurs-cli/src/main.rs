//! Sakurs CLI - Command-line interface for sentence boundary detection
//!
//! This CLI provides a high-performance, user-friendly interface for the
//! Sakurs sentence boundary detection system based on the Δ-Stack Monoid algorithm.

use anyhow::{Context, Result};
use clap::Parser;
use sakurs_cli::commands::{Commands, ConfigCommands, ListCommands};
use sakurs_cli::CliResult;

/// Sakurs - High-performance sentence boundary detection
///
/// A parallel text processing tool based on the Δ-Stack Monoid algorithm
/// for accurate sentence segmentation in multiple languages.
#[derive(Debug, Parser)]
#[command(
    name = "sakurs",
    version,
    author,
    about,
    long_about = None,
    arg_required_else_help = true
)]
struct Cli {
    /// The command to execute
    #[command(subcommand)]
    command: Commands,
}

fn main() -> CliResult<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    match cli.command {
        Commands::Process(args) => args.execute(),
        Commands::Analyze {
            stats,
            benchmark,
            input,
        } => execute_analyze(stats, benchmark, &input),
        Commands::Config { subcommand } => execute_config(subcommand),
        Commands::List { subcommand } => execute_list(subcommand),
    }
}

/// Execute the analyze command
fn execute_analyze(stats: bool, benchmark: bool, input: &str) -> Result<()> {
    if !stats && !benchmark {
        anyhow::bail!("Please specify --stats or --benchmark");
    }

    println!("Analyzing file: {}", input);

    if stats {
        println!("Statistics analysis not yet implemented");
    }

    if benchmark {
        println!("Benchmark analysis not yet implemented");
    }

    Ok(())
}

/// Execute config subcommands
fn execute_config(subcommand: ConfigCommands) -> Result<()> {
    match subcommand {
        ConfigCommands::Generate => {
            let config = sakurs_cli::config::CliConfig::default();
            let toml =
                toml::to_string_pretty(&config).context("Failed to serialize default config")?;
            println!("{}", toml);
            Ok(())
        }
        ConfigCommands::Validate { file } => {
            println!("Validating config file: {}", file);
            // TODO: Implement config validation
            Ok(())
        }
    }
}

/// Execute list subcommands
fn execute_list(subcommand: ListCommands) -> Result<()> {
    match subcommand {
        ListCommands::Languages => {
            println!("Available languages:");
            println!("  - english (English language rules)");
            println!("  - japanese (Japanese language rules)");
            Ok(())
        }
        ListCommands::Formats => {
            println!("Available output formats:");
            println!("  - text (Plain text, one sentence per line)");
            println!("  - json (JSON array with sentence metadata)");
            println!("  - markdown (Markdown formatted output)");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        // Verify that the CLI structure is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_help_output() {
        // Test that help can be generated without panic
        let mut cmd = Cli::command();
        let help = cmd.render_help();
        assert!(!help.to_string().is_empty());
    }
}
