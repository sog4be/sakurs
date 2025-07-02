//! CLI command implementations

use clap::Subcommand;

pub mod process;

/// Available CLI commands
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Process text files for sentence boundary detection
    Process(process::ProcessArgs),

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
            language: process::Language::English,
            parallel: false,
            config: None,
            quiet: false,
            verbose: 0,
            stream: false,
            stream_chunk_mb: 10,
        });

        let debug_str = format!("{:?}", process_cmd);
        assert!(debug_str.contains("Process"));
        assert!(debug_str.contains("test.txt"));

        // Test Config command
        let config_cmd = Commands::Config {
            subcommand: ConfigCommands::Generate,
        };

        let debug_str = format!("{:?}", config_cmd);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("Generate"));

        // Test List command
        let list_cmd = Commands::List {
            subcommand: ListCommands::Languages,
        };

        let debug_str = format!("{:?}", list_cmd);
        assert!(debug_str.contains("List"));
        assert!(debug_str.contains("Languages"));
    }

    #[test]
    fn test_config_commands_variants() {
        // Test Generate variant
        let generate = ConfigCommands::Generate;
        let debug_str = format!("{:?}", generate);
        assert!(debug_str.contains("Generate"));

        // Test Validate variant
        let validate = ConfigCommands::Validate {
            file: "config.toml".to_string(),
        };
        let debug_str = format!("{:?}", validate);
        assert!(debug_str.contains("Validate"));
        assert!(debug_str.contains("config.toml"));

        // Test with different file paths
        let validate_path = ConfigCommands::Validate {
            file: "/path/to/my-config.toml".to_string(),
        };
        let debug_str = format!("{:?}", validate_path);
        assert!(debug_str.contains("/path/to/my-config.toml"));
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
            language: process::Language::English,
            parallel: false,
            config: None,
            quiet: false,
            verbose: 0,
            stream: false,
            stream_chunk_mb: 10,
        });

        let config_cmd = Commands::Config {
            subcommand: ConfigCommands::Generate,
        };

        let list_cmd = Commands::List {
            subcommand: ListCommands::Languages,
        };

        // Verify all variants can be matched
        match process_cmd {
            Commands::Process(_) => (),
            Commands::Config { .. } => panic!("Should be Process"),
            Commands::List { .. } => panic!("Should be Process"),
        }

        match config_cmd {
            Commands::Process(_) => panic!("Should be Config"),
            Commands::Config { .. } => (),
            Commands::List { .. } => panic!("Should be Config"),
        }

        match list_cmd {
            Commands::Process(_) => panic!("Should be List"),
            Commands::Config { .. } => panic!("Should be List"),
            Commands::List { .. } => (),
        }
    }

    #[test]
    fn test_config_commands_completeness() {
        // Test all ConfigCommands variants
        match ConfigCommands::Generate {
            ConfigCommands::Generate => (),
            ConfigCommands::Validate { .. } => panic!("Should be Generate"),
        }

        let validate_cmd = ConfigCommands::Validate {
            file: "test".to_string(),
        };
        match validate_cmd {
            ConfigCommands::Generate => panic!("Should be Validate"),
            ConfigCommands::Validate { .. } => (),
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
