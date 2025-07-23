# Adding New Languages to Sakurs

This guide explains how to add support for new languages to Sakurs using the configurable language rules system.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
  - [Option 1: External Configuration](#option-1-external-configuration-recommended-for-testing)
  - [Option 2: Built-in Configuration](#option-2-built-in-configuration-for-contributing)
- [Configuration File Structure](#configuration-file-structure)
- [Configuration Sections Explained](#configuration-sections-explained)
  - [Metadata](#metadata-required)
  - [Terminators](#terminators-required)
  - [Ellipsis](#ellipsis-optional)
  - [Enclosures](#enclosures-optional)
  - [Suppression](#suppression-optional)
  - [Abbreviations](#abbreviations-optional)
- [Registering Your Language](#registering-your-language)
- [Testing Your Configuration](#testing-your-configuration)
  - [Testing External Configurations](#testing-external-configurations)
  - [Unit Tests](#unit-tests)
  - [Integration Tests](#integration-tests)
- [Language-Specific Considerations](#language-specific-considerations)
  - [Character Sets](#character-sets)
  - [Right-to-Left Languages](#right-to-left-languages)
  - [Complex Scripts](#complex-scripts)
- [Performance Tips](#performance-tips)
- [Examples](#examples)
  - [Minimal Configuration](#minimal-configuration)
  - [Full-Featured Configuration](#full-featured-configuration)
- [Troubleshooting](#troubleshooting)
  - [Configuration doesn't load](#configuration-doesnt-load)
  - [Incorrect sentence detection](#incorrect-sentence-detection)
  - [Performance issues](#performance-issues)
- [Contributing Your Language](#contributing-your-language)
- [Need Help?](#need-help)

## Overview

Sakurs uses a TOML-based configuration system that makes adding new languages straightforward. Language rules are defined declaratively, without requiring any Rust code changes for most use cases.

## Quick Start

You have two options for adding language support:

### Option 1: External Configuration (Recommended for Testing)

1. Generate a configuration template: `sakurs generate-config --language-code {code} --output {file}.toml`
2. Edit the configuration file to define your language rules
3. Validate it: `sakurs validate --language-config {file}.toml`
4. Use it: `sakurs process -i text.txt --language-config {file}.toml`

### Option 2: Built-in Configuration (For Contributing)

1. Create a TOML configuration file in `sakurs-core/configs/languages/`
2. Register it in the language loader
3. Test your configuration and submit a PR

## Configuration File Structure

Create a new file at `sakurs-core/configs/languages/{language_code}.toml`:

```toml
# Metadata section - required
[metadata]
code = "de"        # ISO 639-1 language code
name = "German"    # Human-readable language name

# Sentence terminators - required
[terminators]
chars = [".", "!", "?"]  # Basic sentence-ending punctuation

# Optional: Multi-character patterns
patterns = [
    { pattern = "!?", name = "surprised_question" },
    { pattern = "?!", name = "questioning_exclamation" }
]

# Ellipsis handling - optional but recommended
[ellipsis]
treat_as_boundary = true        # Default behavior
patterns = ["...", "…", "...."] # Patterns to recognize

# Context rules for smarter decisions
context_rules = [
    { condition = "followed_by_capital", boundary = true },
    { condition = "followed_by_lowercase", boundary = false }
]

# Regex-based exceptions
exceptions = [
    { regex = "\\betc\\.\\.\\.", boundary = false }
]

# Paired delimiters - optional but recommended
[enclosures]
pairs = [
    { open = "(", close = ")" },
    { open = "[", close = "]" },
    { open = "{", close = "}" },
    { open = "„", close = """, comment = "German quotes" },
    { open = "‚", close = "'", comment = "German single quotes" },
    { open = '"', close = '"', symmetric = true }
]

# Fast suppression patterns - optional
[suppression]
fast_patterns = [
    # Suppress boundaries for contractions
    { char = "'", before = "alpha", after = "alpha" },
    # List items at line start
    { char = ")", line_start = true, before = "alnum" }
]

# Abbreviations - highly recommended
[abbreviations]
# Group abbreviations by category for better organization
common = ["z.B", "d.h", "bzw", "ca", "evtl", "ggf", "inkl", "max", "min"]
titles = ["Dr", "Prof", "Dipl.-Ing", "Mag", "Hr", "Fr"]
locations = ["Str", "Pl", "Weg"]
months = ["Jan", "Feb", "Mär", "Apr", "Mai", "Jun", "Jul", "Aug", "Sep", "Okt", "Nov", "Dez"]
```

## Configuration Sections Explained

### Metadata (Required)
- `code`: ISO 639-1 two-letter language code
- `name`: Human-readable language name

### Terminators (Required)
- `chars`: Array of single characters that end sentences
- `patterns`: Optional multi-character patterns with names

### Ellipsis (Optional)
Controls how ellipsis patterns are handled:
- `treat_as_boundary`: Default behavior (true = sentence boundary)
- `patterns`: Strings to recognize as ellipsis
- `context_rules`: Context-based decisions
- `exceptions`: Regex patterns for special cases

### Enclosures (Optional)
Defines paired delimiters that should not contain sentence boundaries:
- `open`/`close`: The delimiter characters
- `symmetric`: Set to true for quotes that use the same character
- `comment`: Optional description

### Suppression (Optional)
High-performance pattern matching for common suppressions:
- `char`: The character to match
- `before`/`after`: Character class (`alpha`, `alnum`, `whitespace`, or specific char)
- `line_start`: Only match at line beginning

### Abbreviations (Optional)
Lists of known abbreviations that don't end sentences. Group them logically for maintainability.

## Registering Your Language

After creating the configuration file, register it in `sakurs-core/src/domain/language/config/loader.rs`:

```rust
let embedded_configs = [
    embed_language_config!("en", "../../../../configs/languages/english.toml"),
    embed_language_config!("ja", "../../../../configs/languages/japanese.toml"),
    // Add your language here:
    embed_language_config!("de", "../../../../configs/languages/german.toml"),
];
```

## Testing Your Configuration

### Testing External Configurations

Before submitting a built-in configuration, test it as an external config:

```bash
# Generate template
sakurs generate-config --language-code test --output test-lang.toml

# Edit the configuration...

# Validate syntax and structure
sakurs validate --language-config test-lang.toml

# Test with sample text
echo "Test sentence. Another one!" | sakurs process -i - --language-config test-lang.toml

# Test with real files
sakurs process -i sample.txt --language-config test-lang.toml
```

### Unit Tests

For built-in configurations, add tests to verify your configuration loads correctly:

```rust
#[test]
fn test_german_config_loads() {
    let config = get_language_config("de").expect("German config should load");
    assert_eq!(config.metadata.code, "de");
    assert_eq!(config.metadata.name, "German");
}
```

### Integration Tests

Create test cases for common patterns in your language:

```rust
#[test]
fn test_german_abbreviations() {
    let processor = SentenceProcessor::with_language("de").unwrap();
    let text = "Das ist z.B. ein Beispiel.";
    let output = processor.process(Input::from_text(text)).unwrap();
    assert_eq!(output.boundaries.len(), 1); // Only one sentence
}
```

## Language-Specific Considerations

### Character Sets
- Sakurs handles UTF-8 natively
- No special configuration needed for non-ASCII characters
- Ensure your TOML file is saved as UTF-8

### Right-to-Left Languages
- Currently requires additional implementation work
- Contact maintainers for RTL language support

### Complex Scripts
- Languages with complex writing systems may need custom rules
- The configuration system handles most cases, but some may require code changes

## Performance Tips

1. **Abbreviations**: Use the Trie-based lookup for best performance
2. **Suppression patterns**: Prefer `fast_patterns` over regex when possible
3. **Enclosures**: List most common pairs first
4. **Patterns**: Keep regex patterns simple and specific

## Examples

### Minimal Configuration

```toml
[metadata]
code = "xx"
name = "Example"

[terminators]
chars = [".", "!", "?"]
```

### Full-Featured Configuration

See `english.toml` or `japanese.toml` for comprehensive examples.

## Troubleshooting

### Configuration doesn't load
- Check TOML syntax with a validator
- Ensure file path in loader.rs is correct
- Verify the language code matches between file and loader

### Incorrect sentence detection
- Add missing abbreviations
- Adjust context rules
- Check enclosure definitions
- Review suppression patterns

### Performance issues
- Minimize regex usage
- Group related abbreviations
- Use fast_patterns where possible

## Contributing Your Language

1. Follow the configuration guidelines above
2. Add comprehensive tests
3. Include example texts that demonstrate correct behavior
4. Submit a pull request with:
   - The TOML configuration file
   - Updates to loader.rs
   - Tests for your language
   - Example usage in documentation

## Need Help?

- Check existing language configurations for examples
- Open an issue for language-specific questions
- Join discussions about language support improvements