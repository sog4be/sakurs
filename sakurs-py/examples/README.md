# Sakurs Python Examples

This directory contains examples demonstrating how to use the sakurs Python package for sentence boundary detection.

## Examples

### 1. Basic Usage (`basic_usage.py`)
- Simple sentence splitting
- Working with different languages
- Using the `SentenceSplitter` class for multiple texts
- Handling edge cases

### 2. Performance Tuning (`performance_tuning.py`)
- Configuring chunk size (`chunk_kb`) and thread count (`threads`)
- Comparing execution modes (`sequential`, `parallel`, `adaptive`)
- Benchmarking different settings
- Tips for optimal performance

### 3. Error Handling (`error_handling.py`)
- Handling unsupported/invalid language codes
- Dealing with edge case inputs (empty, whitespace-only, unpunctuated text)
- Implementing a language-fallback pattern
- Using performance parameters (`threads`, `execution_mode`)

### 4. Custom Language Configuration (`custom_language.py`)
- Building a `LanguageConfig` programmatically (`MetadataConfig`, `TerminatorConfig`,
  `EllipsisConfig`, `EnclosureConfig`, `SuppressionConfig`, `AbbreviationConfig`)
- Defining custom abbreviations and enclosure rules
- Splitting text with `sakurs.split(text, language_config=config)`

### 5. Streaming (`streaming_demo.py`)
- `iter_split()` for responsive, incremental iteration over in-memory text
- `split_large_file()` for true memory-efficient processing of large files

## Running the Examples

First, make sure sakurs is installed:

```bash
pip install sakurs
```

Then run any example:

```bash
python basic_usage.py
python performance_tuning.py
python error_handling.py
python custom_language.py
python streaming_demo.py
```

## Key Takeaways

1. **Simple API**: Most users only need `sakurs.split(text)`
2. **Language Support**: Currently supports English and Japanese, plus any language defined by
   a custom TOML/`LanguageConfig`
3. **Performance**: Reuse `SentenceSplitter` instances for better performance
4. **Configuration**: Tune `chunk_kb` and `threads` based on your use case
5. **Error Handling**: The library handles edge cases gracefully