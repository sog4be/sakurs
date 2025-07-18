//! CLI command implementations

use clap::Subcommand;

pub mod process;

/// Available CLI commands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Process text files for sentence boundary detection
    Process(process::ProcessArgs),

    /// List available components
    List {
        #[command(subcommand)]
        subcommand: ListCommands,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands_debug_format() {
        // Test Process command with minimal args
        let process_cmd = Commands::Process(process::ProcessArgs {
            input: vec!["test.txt".to_string()],
            output: None,
            format: process::OutputFormat::Text,
            language: Some(process::Language::English),
            language_config: None,
            language_code: None,
            parallel: false,
            adaptive: false,
            threads: None,
            chunk_kb: None,
            quiet: false,
            verbose: 0,
            stream: false,
            stream_chunk_mb: 10,
        });

        let debug_str = format!("{:?}", process_cmd);
        assert!(debug_str.contains("Process"));
        assert!(debug_str.contains("test.txt"));

        // Test List command
        let list_cmd = Commands::List {
            subcommand: ListCommands::Languages,
        };

        let debug_str = format!("{:?}", list_cmd);
        assert!(debug_str.contains("List"));
        assert!(debug_str.contains("Languages"));
    }

    #[test]
    fn test_list_commands_variants() {
        // Test Languages variant
        let languages = ListCommands::Languages;
        let debug_str = format!("{:?}", languages);
        assert!(debug_str.contains("Languages"));

        // Test Formats variant
        let formats = ListCommands::Formats;
        let debug_str = format!("{:?}", formats);
        assert!(debug_str.contains("Formats"));
    }

    #[test]
    fn test_enum_variants_completeness() {
        // Ensure all Commands variants are covered
        let process_cmd = Commands::Process(process::ProcessArgs {
            input: vec!["test.txt".to_string()],
            output: None,
            format: process::OutputFormat::Text,
            language: Some(process::Language::English),
            language_config: None,
            language_code: None,
            parallel: false,
            adaptive: false,
            threads: None,
            chunk_kb: None,
            quiet: false,
            verbose: 0,
            stream: false,
            stream_chunk_mb: 10,
        });

        let list_cmd = Commands::List {
            subcommand: ListCommands::Languages,
        };

        // Verify all variants can be matched
        match process_cmd {
            Commands::Process(_) => (),
            Commands::List { .. } => panic!("Should be Process"),
        }

        match list_cmd {
            Commands::Process(_) => panic!("Should be List"),
            Commands::List { .. } => (),
        }
    }

    #[test]
    fn test_list_commands_completeness() {
        // Test all ListCommands variants
        match ListCommands::Languages {
            ListCommands::Languages => (),
            ListCommands::Formats => panic!("Should be Languages"),
        }

        match ListCommands::Formats {
            ListCommands::Languages => panic!("Should be Formats"),
            ListCommands::Formats => (),
        }
    }
}
