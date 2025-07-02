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
            use std::fs;

            // Read the config file
            let content = fs::read_to_string(&file)
                .with_context(|| format!("Failed to read config file: {}", file))?;

            // Try to parse it
            match toml::from_str::<sakurs_cli::config::CliConfig>(&content) {
                Ok(config) => {
                    println!("✅ Config file is valid!");
                    println!();

                    // Show loaded configuration
                    println!("Loaded configuration:");
                    println!("  Processing:");
                    println!(
                        "    - Default language: {}",
                        config.processing.default_language
                    );
                    println!(
                        "    - Detect abbreviations: {}",
                        config.processing.detect_abbreviations
                    );
                    println!(
                        "    - Strict punctuation: {}",
                        config.processing.strict_punctuation
                    );
                    println!();
                    println!("  Output:");
                    println!("    - Default format: {}", config.output.default_format);
                    println!("    - Include metadata: {}", config.output.include_metadata);
                    println!("    - Pretty JSON: {}", config.output.pretty_json);
                    println!();
                    println!("  Performance:");
                    println!(
                        "    - Parallel threshold: {} MB",
                        config.performance.parallel_threshold_mb
                    );
                    println!("    - Chunk size: {} KB", config.performance.chunk_size_kb);
                    println!(
                        "    - Worker threads: {}",
                        if config.performance.worker_threads == 0 {
                            "auto".to_string()
                        } else {
                            config.performance.worker_threads.to_string()
                        }
                    );

                    // Validate specific settings
                    if config.performance.chunk_size_kb == 0 {
                        println!();
                        println!(
                            "⚠️  Warning: chunk_size_kb is 0, which is invalid. Use at least 1."
                        );
                    }

                    if !["english", "japanese"]
                        .contains(&config.processing.default_language.as_str())
                    {
                        println!();
                        println!("⚠️  Warning: '{}' is not a supported language. Available: english, japanese", 
                            config.processing.default_language);
                    }

                    if !["text", "json", "markdown"]
                        .contains(&config.output.default_format.as_str())
                    {
                        println!();
                        println!("⚠️  Warning: '{}' is not a supported format. Available: text, json, markdown", 
                            config.output.default_format);
                    }

                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Config file is invalid!");
                    eprintln!();
                    eprintln!("Error: {}", e);

                    // Try to provide helpful hints based on common errors
                    if e.to_string().contains("missing field") {
                        eprintln!();
                        eprintln!("💡 Hint: Make sure all required fields are present.");
                        eprintln!("   Run 'sakurs config generate' to see a complete example.");
                    } else if e.to_string().contains("invalid type") {
                        eprintln!();
                        eprintln!("💡 Hint: Check that all values have the correct type.");
                        eprintln!(
                            "   Numbers should not be quoted, booleans should be true/false."
                        );
                    }

                    std::process::exit(1);
                }
            }
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
