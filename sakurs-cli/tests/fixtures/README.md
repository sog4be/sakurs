# Test Fixtures

This directory contains test data files used for CLI integration tests.

## Files

- `english-sample.txt` - English text with various punctuation patterns, abbreviations, and quotations
- `japanese-sample.txt` - Japanese text with full-width punctuation and quotation marks

## Usage

These files are used by:
- CLI integration tests (`sakurs-cli/tests/cli_integration.rs`)
- Manual testing during development

## Adding New Fixtures

When adding new test fixtures:
1. Keep files small (< 1KB) for version control
2. Include edge cases relevant to sentence boundary detection
3. Document the purpose of each fixture file
4. Consider both positive and negative test cases