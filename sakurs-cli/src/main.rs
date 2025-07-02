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
        Commands::Config { subcommand } => execute_config(subcommand),
        Commands::List { subcommand } => execute_list(subcommand),
    }
}

/// Execute config subcommands
fn execute_config(subcommand: ConfigCommands) -> Result<()> {
    match subcommand {
        ConfigCommands::Generate => {
            let config = sakurs_cli::config::CliConfig::default();
            let toml =
                toml::to_string_pretty(&config).context("Failed to serialize default config")?;
            println!("{toml}");
            Ok(())
        }
        ConfigCommands::Validate { file } => {
            use std::fs;

            // Read the config file
            let content = fs::read_to_string(&file)
                .with_context(|| format!("Failed to read config file: {file}"))?;

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
                    eprintln!("Error: {e}");

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
    use std::io::Write;
    use tempfile::NamedTempFile;

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
        assert!(help_str.contains("config"));
        assert!(help_str.contains("list"));
    }

    #[test]
    fn test_execute_config_generate() {
        // Test config generation
        let result = execute_config(ConfigCommands::Generate);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_config_validate_valid_file() -> Result<()> {
        // Create a valid config file
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
[processing]
default_language = "english"
detect_abbreviations = true
strict_punctuation = false

[output]
default_format = "text"
include_metadata = false
pretty_json = true

[performance]
parallel_threshold_mb = 1
chunk_size_kb = 256
worker_threads = 0
"#
        )?;

        let result = execute_config(ConfigCommands::Validate {
            file: temp_file.path().to_string_lossy().to_string(),
        });
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_execute_config_validate_missing_file() {
        // Test with non-existent file
        let result = execute_config(ConfigCommands::Validate {
            file: "nonexistent-config-file.toml".to_string(),
        });
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to read config file"));
    }

    // Note: This test is commented out because it calls std::process::exit(1)
    // which causes the test runner to exit. In a production application, you might
    // want to refactor the error handling to return errors instead of exiting.

    // #[test]
    // fn test_execute_config_validate_invalid_toml() -> Result<()> {
    //     // This test would call std::process::exit(1) and cannot be tested directly
    //     Ok(())
    // }

    #[test]
    fn test_execute_config_validate_with_warnings() -> Result<()> {
        // Create config with invalid values that should trigger warnings
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
[processing]
default_language = "unsupported_language"
detect_abbreviations = true
strict_punctuation = false

[output]
default_format = "unsupported_format"
include_metadata = false
pretty_json = true

[performance]
parallel_threshold_mb = 1
chunk_size_kb = 0
worker_threads = 0
"#
        )?;

        let result = execute_config(ConfigCommands::Validate {
            file: temp_file.path().to_string_lossy().to_string(),
        });
        // Should still pass validation but show warnings
        assert!(result.is_ok());
        Ok(())
    }

    // Note: These tests are commented out because they call std::process::exit(1)
    // which causes the test runner to exit. To properly test these scenarios,
    // the error handling would need to be refactored to return errors instead.

    // #[test]
    // fn test_execute_config_validate_missing_fields() -> Result<()> {
    //     // This test would call std::process::exit(1) and cannot be tested directly
    //     Ok(())
    // }

    // #[test]
    // fn test_execute_config_validate_type_errors() -> Result<()> {
    //     // This test would call std::process::exit(1) and cannot be tested directly
    //     Ok(())
    // }

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

        // Test all config subcommands
        assert!(execute_config(ConfigCommands::Generate).is_ok());

        // Test all list subcommands
        assert!(execute_list(ListCommands::Languages).is_ok());
        assert!(execute_list(ListCommands::Formats).is_ok());
    }

    #[test]
    fn test_config_validation_edge_cases() -> Result<()> {
        // Test minimal valid config
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            r#"
[processing]
default_language = "japanese"
detect_abbreviations = false
strict_punctuation = true

[output]
default_format = "json"
include_metadata = true
pretty_json = false

[performance]
parallel_threshold_mb = 100
chunk_size_kb = 1024
worker_threads = 8
"#
        )?;

        let result = execute_config(ConfigCommands::Validate {
            file: temp_file.path().to_string_lossy().to_string(),
        });
        assert!(result.is_ok());

        // Test config with markdown format
        let mut temp_file2 = NamedTempFile::new()?;
        writeln!(
            temp_file2,
            r#"
[processing]
default_language = "english"
detect_abbreviations = true
strict_punctuation = false

[output]
default_format = "markdown"
include_metadata = false
pretty_json = true

[performance]
parallel_threshold_mb = 50
chunk_size_kb = 512
worker_threads = 4
"#
        )?;

        let result2 = execute_config(ConfigCommands::Validate {
            file: temp_file2.path().to_string_lossy().to_string(),
        });
        assert!(result2.is_ok());

        Ok(())
    }

    #[test]
    fn test_config_file_paths() {
        // Test various file path formats
        let invalid_paths = vec![
            "/nonexistent/path/config.toml",
            "~/nonexistent/config.toml",
            "./missing/config.toml",
            "../missing.toml",
        ];

        for path in invalid_paths {
            let result = execute_config(ConfigCommands::Validate {
                file: path.to_string(),
            });
            assert!(result.is_err(), "Should fail for path: {}", path);
        }
    }
}
