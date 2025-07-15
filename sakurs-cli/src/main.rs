//! Sakurs CLI - Command-line interface for sentence boundary detection
//!
//! This CLI provides a high-performance, user-friendly interface for the
//! Sakurs sentence boundary detection system based on the Δ-Stack Monoid algorithm.

use anyhow::Result;
use clap::Parser;
use sakurs_cli::commands::{Commands, ListCommands};
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
        Commands::List { subcommand } => execute_list(subcommand),
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
        let help_str = help.to_string();
        assert!(!help_str.is_empty());
        assert!(help_str.contains("sakurs"));
        assert!(help_str.contains("process"));
        assert!(help_str.contains("list"));
    }

    #[test]
    fn test_execute_list_languages() {
        let result = execute_list(ListCommands::Languages);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_list_formats() {
        let result = execute_list(ListCommands::Formats);
        assert!(result.is_ok());
    }

    #[test]
    fn test_main_command_dispatch() {
        // Test that command dispatch logic works for each variant
        // Note: We can't easily test main() directly since it calls parse(),
        // but we can test the execute functions directly

        // Test all list subcommands
        assert!(execute_list(ListCommands::Languages).is_ok());
        assert!(execute_list(ListCommands::Formats).is_ok());
    }
}
