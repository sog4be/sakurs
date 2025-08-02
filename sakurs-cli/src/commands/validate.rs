//! Validate command implementation

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Arguments for the validate command
#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Path to language configuration file to validate
    #[arg(short = 'c', long, value_name = "FILE", required = true)]
    pub language_config: PathBuf,
}

impl ValidateArgs {
    /// Execute the validate command
    pub fn execute(&self) -> Result<()> {
        println!(
            "Validating language configuration: {}",
            self.language_config.display()
        );

        // TODO: Custom language configuration validation is not yet supported in the new architecture
        println!(
            "✗ Custom language configuration validation is not yet supported in this version."
        );
        println!("  Only built-in languages (en, ja) are currently available.");
        Err(anyhow::anyhow!(
            "Custom language configuration is not yet supported"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_args_debug() {
        let args = ValidateArgs {
            language_config: PathBuf::from("test.toml"),
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("ValidateArgs"));
        assert!(debug_str.contains("test.toml"));
    }

    #[test]
    fn test_validate_not_supported() {
        let toml_content = r#"
[metadata]
code = "test"
name = "Test Language"

[terminators]
chars = ["."]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();

        let args = ValidateArgs {
            language_config: temp_file.path().to_path_buf(),
        };

        // Currently validation is not supported
        assert!(args.execute().is_err());
    }

    #[test]
    fn test_validate_invalid_config() {
        let toml_content = r#"
[metadata]
code = ""
name = "Test"

[terminators]
chars = ["."]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();

        let args = ValidateArgs {
            language_config: temp_file.path().to_path_buf(),
        };

        assert!(args.execute().is_err());
    }
}
