//! Integration tests for the sakurs CLI

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to get the path to a test fixture
fn fixture_path(name: &str) -> String {
    format!("tests/fixtures/{}", name)
}

#[test]
fn test_process_english_text() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dr. Smith went to the store."))
        .stdout(predicate::str::contains("He bought some milk and eggs."));
}

#[test]
fn test_process_japanese_text() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("japanese-sample.txt"))
        .arg("-l")
        .arg("japanese");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("これはテストです。"))
        .stdout(predicate::str::contains(
            "日本語の文章を正しく分割できるか確認しています。",
        ));
}

#[test]
fn test_json_output() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("-f")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("]"))
        .stdout(predicate::str::contains("\"text\""))
        .stdout(predicate::str::contains("\"offset\""));
}

#[test]
fn test_markdown_output() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("japanese-sample.txt"))
        .arg("-l")
        .arg("japanese")
        .arg("-f")
        .arg("markdown");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("1. "))
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("*Total sentences:"));
}

#[test]
fn test_output_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let output_file = temp_dir.path().join("output.txt");

    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("-o")
        .arg(&output_file);

    cmd.assert().success();

    // Check that file was created and contains expected content
    let content = fs::read_to_string(&output_file).unwrap();
    assert!(content.contains("Dr. Smith went to the store."));
}

#[test]
fn test_glob_pattern() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process").arg("-i").arg(fixture_path("*.txt"));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dr. Smith")) // From English file
        .stdout(predicate::str::contains("これはテストです")); // From Japanese file
}

#[test]
fn test_invalid_file() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process").arg("-i").arg("nonexistent.txt");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No files found"));
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("sentence boundary detection"));
}

#[test]
fn test_list_languages() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("list").arg("languages");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("english"))
        .stdout(predicate::str::contains("japanese"));
}

#[test]
fn test_config_generate() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("config").arg("generate");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("[processing]"))
        .stdout(predicate::str::contains("[output]"))
        .stdout(predicate::str::contains("[performance]"));
}

#[test]
fn test_streaming_mode() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("--stream");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dr. Smith went to the store."))
        .stdout(predicate::str::contains("He bought some milk and eggs."));
}

#[test]
fn test_streaming_with_custom_chunk_size() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("--stream")
        .arg("--stream-chunk-mb")
        .arg("1");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dr. Smith went to the store."));
}

#[test]
fn test_streaming_japanese() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("japanese-sample.txt"))
        .arg("-l")
        .arg("japanese")
        .arg("--stream");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("これはテストです。"))
        .stdout(predicate::str::contains(
            "日本語の文章を正しく分割できるか確認しています。",
        ));
}
