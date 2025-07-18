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
        use sakurs_core::domain::language::ConfigurableLanguageRules;
        use sakurs_core::LanguageRules;

        println!(
            "Validating language configuration: {}",
            self.language_config.display()
        );

        // Try to load and validate the configuration
        match ConfigurableLanguageRules::from_file(&self.language_config, None) {
            Ok(rules) => {
                println!("✓ Configuration is valid!");
                println!("  Language code: {}", rules.language_code());
                println!("  Language name: {}", rules.language_name());
                Ok(())
            }
            Err(e) => {
                println!("✗ Configuration is invalid!");
                println!("  Error: {e}");
                Err(anyhow::anyhow!("Validation failed: {}", e))
            }
        }
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
    fn test_validate_valid_config() {
        let toml_content = r#"
[metadata]
code = "test"
name = "Test Language"

[terminators]
chars = ["."]

[ellipsis]
patterns = []

[enclosures]
pairs = []

[suppression]

[abbreviations]
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", toml_content).unwrap();

        let args = ValidateArgs {
            language_config: temp_file.path().to_path_buf(),
        };

        assert!(args.execute().is_ok());
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
