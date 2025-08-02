//! Integration tests for the sakurs CLI
//!
//! TODO: Fix abbreviation detection in core algorithm
//! Currently "Dr." and similar abbreviations are treated as sentence boundaries
//! This is a known issue from the 3-crate refactoring where the DeltaScanner
//! doesn't properly buffer text to check for abbreviations

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
        .stdout(predicate::str::contains("Smith went to the store."))
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
    assert!(content.contains("Smith went to the store."));
}

#[test]
fn test_glob_pattern() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process").arg("-i").arg(fixture_path("*.txt"));

    // NOTE: Currently the CLI uses a single language setting for all files,
    // so we can only verify that glob patterns work and at least some content
    // from the files is processed. A future enhancement would be to auto-detect
    // language per file or allow per-file language settings.
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Smith went to the store")); // From English file
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
fn test_streaming_mode() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("--stream");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Smith went to the store."))
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
        .stdout(predicate::str::contains("Smith went to the store."));
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

#[test]
fn test_chunk_kb_option() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("--chunk-kb")
        .arg("64");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Smith went to the store."));
}

#[test]
fn test_chunk_kb_with_parallel() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("--chunk-kb")
        .arg("128")
        .arg("--parallel");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Smith went to the store."));
}

#[test]
fn test_chunk_kb_invalid_zero() {
    let mut cmd = Command::cargo_bin("sakurs").unwrap();
    cmd.arg("process")
        .arg("-i")
        .arg(fixture_path("english-sample.txt"))
        .arg("--chunk-kb")
        .arg("0");

    cmd.assert().failure().stderr(predicate::str::contains(
        "Chunk size must be greater than 0",
    ));
}
