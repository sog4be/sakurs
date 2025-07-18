//! Generate config command implementation

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

/// Arguments for the generate-config command
#[derive(Debug, Args)]
pub struct GenerateConfigArgs {
    /// Language code for the new configuration
    #[arg(short = 'l', long, value_name = "CODE", required = true)]
    pub language_code: String,

    /// Output file path
    #[arg(short, long, value_name = "FILE", required = true)]
    pub output: PathBuf,
}

impl GenerateConfigArgs {
    /// Execute the generate-config command
    pub fn execute(&self) -> Result<()> {
        use std::fs;

        println!("Generating language configuration template...");
        println!("  Language code: {}", self.language_code);
        println!("  Output file: {}", self.output.display());

        // Generate template configuration
        let template = self.generate_template();

        // Write to file
        fs::write(&self.output, template)
            .with_context(|| format!("Failed to write to {}", self.output.display()))?;

        println!("✓ Configuration template generated successfully!");
        println!();
        println!("Next steps:");
        println!("1. Edit the configuration file to customize language rules");
        println!("2. Validate your configuration:");
        println!(
            "   sakurs validate --language-config {}",
            self.output.display()
        );
        println!("3. Use it for processing:");
        println!(
            "   sakurs process -i input.txt --language-config {}",
            self.output.display()
        );

        Ok(())
    }

    /// Generate template configuration content
    fn generate_template(&self) -> String {
        format!(
            r#"# Language configuration for {}

[metadata]
code = "{}"
name = "Custom Language"

# Sentence terminator characters
[terminators]
# Basic sentence-ending punctuation
chars = [".", "!", "?"]

# Multi-character terminator patterns (optional)
patterns = [
    # Example: {{ pattern = "!?", name = "surprised_question" }}
]

# Ellipsis handling
[ellipsis]
# Whether ellipsis should be treated as sentence boundary by default
treat_as_boundary = true

# Ellipsis patterns to recognize
patterns = ["...", "…"]

# Context-based rules for ellipsis
context_rules = [
    {{ condition = "followed_by_capital", boundary = true }},
    {{ condition = "followed_by_lowercase", boundary = false }}
]

# Exception patterns (regex) - patterns where ellipsis should NOT be a boundary
exceptions = [
    # Example: {{ regex = "\\b(um|uh|er)\\.\\.\\.", boundary = false }}
]

# Enclosure pairs (quotes, parentheses, etc.)
[enclosures]
pairs = [
    {{ open = "(", close = ")" }},
    {{ open = "[", close = "]" }},
    {{ open = "{{", close = "}}" }},
    {{ open = '"', close = '"', symmetric = true }},
    {{ open = "'", close = "'", symmetric = true }}
]

# Suppression rules for preventing false boundaries
[suppression]
# Fast pattern matching for enclosure suppression
fast_patterns = [
    # Apostrophes in contractions
    {{ char = "'", before = "alpha", after = "alpha" }},
    # List items at line start: 1) item
    {{ char = ")", line_start = true, before = "alnum" }}
]

# Regex patterns for more complex suppression (optional)
regex_patterns = [
    # Example: {{ pattern = "\\d+'\\d+\"", description = "Feet and inches like 5'9\"" }}
]

# Abbreviations organized by category
[abbreviations]
# Add your abbreviations here, organized by category
# Category names are arbitrary - choose what makes sense for your language

titles = ["Dr", "Mr", "Mrs", "Ms", "Prof"]
academic = ["Ph.D", "M.D", "B.A", "M.A"]
business = ["Inc", "Corp", "Ltd", "LLC", "Co"]
common = ["etc", "vs", "e.g", "i.e"]

# Add more categories as needed:
# geographic = ["St", "Ave", "Blvd"]
# measurement = ["oz", "lb", "kg", "km"]
# month = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
"#,
            self.language_code, self.language_code
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_config_args_debug() {
        let args = GenerateConfigArgs {
            language_code: "fr".to_string(),
            output: PathBuf::from("french.toml"),
        };

        let debug_str = format!("{:?}", args);
        assert!(debug_str.contains("GenerateConfigArgs"));
        assert!(debug_str.contains("fr"));
        assert!(debug_str.contains("french.toml"));
    }

    #[test]
    fn test_generate_template() {
        let args = GenerateConfigArgs {
            language_code: "test".to_string(),
            output: PathBuf::from("test.toml"),
        };

        let template = args.generate_template();
        assert!(template.contains("code = \"test\""));
        assert!(template.contains("[metadata]"));
        assert!(template.contains("[terminators]"));
        assert!(template.contains("[abbreviations]"));
    }

    #[test]
    fn test_execute_success() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_config.toml");

        let args = GenerateConfigArgs {
            language_code: "test".to_string(),
            output: output_path.clone(),
        };

        assert!(args.execute().is_ok());
        assert!(output_path.exists());

        // Verify the generated file contains expected content
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("code = \"test\""));
    }
}
