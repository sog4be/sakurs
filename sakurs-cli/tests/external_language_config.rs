//! Integration tests for external language configuration

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Test generating language configuration template
#[test]
fn test_generate_config_command() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test_lang.toml");

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "generate-config",
        "--language-code",
        "test",
        "--output",
        output_path.to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "Configuration template generated successfully",
    ));

    // Verify file was created
    assert!(output_path.exists());

    // Verify content
    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("code = \"test\""));
    assert!(content.contains("[metadata]"));
    assert!(content.contains("[terminators]"));
    assert!(content.contains("[abbreviations]"));
}

/// Test validating a valid configuration
#[test]
fn test_validate_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("valid.toml");

    // Create a valid configuration
    let config_content = r#"
[metadata]
code = "test"
name = "Test Language"

[terminators]
chars = [".", "!", "?"]

[ellipsis]
patterns = ["...", "…"]

[enclosures]
pairs = [
    { open = "(", close = ")" },
    { open = "[", close = "]" }
]

[suppression]

[abbreviations]
common = ["Mr", "Mrs", "Dr"]
"#;

    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "validate",
        "--language-config",
        config_path.to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Configuration is valid"))
    .stdout(predicate::str::contains("Language code: test"));
}

/// Test validating an invalid configuration
#[test]
fn test_validate_invalid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("invalid.toml");

    // Create an invalid configuration (empty language code)
    let config_content = r#"
[metadata]
code = ""
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

    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "validate",
        "--language-config",
        config_path.to_str().unwrap(),
    ])
    .assert()
    .failure()
    .stdout(predicate::str::contains("Configuration is invalid"))
    .stdout(predicate::str::contains("Language code is required"));
}

/// Test processing text with external language configuration
#[test]
fn test_process_with_external_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("custom.toml");
    let input_path = temp_dir.path().join("input.txt");

    // Create a custom language configuration
    let config_content = r#"
[metadata]
code = "custom"
name = "Custom Language"

[terminators]
chars = [".", "!", "?"]
patterns = [
    { pattern = "!!", name = "double_exclamation" }
]

[ellipsis]
patterns = ["..."]
treat_as_boundary = false

[enclosures]
pairs = [
    { open = "(", close = ")" },
    { open = '"', close = '"', symmetric = true }
]

[suppression]
fast_patterns = [
    { char = "'", before = "alpha", after = "alpha" }
]

[abbreviations]
titles = ["Dr", "Mr", "Mrs", "Ms"]
common = ["etc", "vs"]
"#;

    fs::write(&config_path, config_content).unwrap();

    // Create test input
    let input_text = "Hello Dr. Smith! How are you? I'm fine!!";
    fs::write(&input_path, input_text).unwrap();

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "process",
        "-i",
        input_path.to_str().unwrap(),
        "--language-config",
        config_path.to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("Hello Dr. Smith!"))
    .stdout(predicate::str::contains("How are you?"))
    .stdout(predicate::str::contains("I'm fine!"));
}

/// Test process command with language code override
#[test]
fn test_process_with_language_code_override() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("base.toml");
    let input_path = temp_dir.path().join("input.txt");

    // Create a configuration with one language code
    let config_content = r#"
[metadata]
code = "base"
name = "Base Language"

[terminators]
chars = ["."]

[ellipsis]
patterns = []

[enclosures]
pairs = []

[suppression]

[abbreviations]
"#;

    fs::write(&config_path, config_content).unwrap();

    // Create test input
    let input_text = "Hello world. How are you.";
    fs::write(&input_path, input_text).unwrap();

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "process",
        "-i",
        input_path.to_str().unwrap(),
        "--language-config",
        config_path.to_str().unwrap(),
        "--language-code",
        "override",
    ])
    .assert()
    .success();
}

/// Test that --language and --language-config are mutually exclusive
#[test]
fn test_language_options_mutual_exclusion() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("dummy.toml");

    // Create dummy config
    fs::write(&config_path, "[metadata]\ncode = \"test\"\nname = \"Test\"").unwrap();

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "process",
        "-i",
        "-",
        "--language",
        "english",
        "--language-config",
        config_path.to_str().unwrap(),
    ])
    .write_stdin("test")
    .assert()
    .failure()
    .stderr(predicate::str::contains("cannot be used with"));
}

/// Test processing from stdin with external config
#[test]
fn test_process_stdin_with_external_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("stdin_test.toml");

    // Create a minimal configuration
    let config_content = r#"
[metadata]
code = "test"
name = "Test Language"

[terminators]
chars = [".", "!"]

[ellipsis]
patterns = []

[enclosures]
pairs = []

[suppression]

[abbreviations]
"#;

    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "process",
        "-i",
        "-",
        "--language-config",
        config_path.to_str().unwrap(),
    ])
    .write_stdin("First sentence. Second sentence!")
    .assert()
    .success()
    .stdout(predicate::str::contains("First sentence."))
    .stdout(predicate::str::contains("Second sentence!"));
}

/// Test JSON output with external config
#[test]
fn test_json_output_with_external_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("json_test.toml");

    // Create configuration
    let config_content = r#"
[metadata]
code = "json_test"
name = "JSON Test Language"

[terminators]
chars = ["."]

[ellipsis]
patterns = []

[enclosures]
pairs = []

[suppression]

[abbreviations]
"#;

    fs::write(&config_path, config_content).unwrap();

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.args(&[
        "process",
        "-i",
        "-",
        "-f",
        "json",
        "--language-config",
        config_path.to_str().unwrap(),
    ])
    .write_stdin("Test.")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"text\": \"Test.\""))
    .stdout(predicate::str::contains("\"offset\": 0"))
    .stdout(predicate::str::contains("\"length\": 5"));
}
