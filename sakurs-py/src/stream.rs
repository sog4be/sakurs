//! Streaming functionality for processing large texts

use crate::exceptions::InternalError;
use crate::input::PyInput;
use crate::iterator::SentenceIterator;
use crate::language_config::LanguageConfig;
use pyo3::prelude::*;
use pyo3::types::{PyIterator, PyString};
use sakurs_core::{Config, SentenceProcessor};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

/// Default chunk size for streaming (10MB)
#[allow(dead_code)]
const DEFAULT_CHUNK_SIZE_MB: usize = 10;

/// Default overlap size for streaming (1KB)
#[allow(dead_code)]
const DEFAULT_OVERLAP_SIZE: usize = 1024;

/// Create a streaming sentence iterator from various input types
#[allow(clippy::too_many_arguments)]
pub fn create_stream_iterator(
    py: Python,
    input: &Bound<'_, PyAny>,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    chunk_size_mb: usize,
    overlap_size: usize,
    encoding: &str,
    preserve_whitespace: bool,
) -> PyResult<SentenceIterator> {
    // Build processor configuration
    let (mut config_builder, custom_rules) = if let Some(lang_config) = language_config {
        // Use custom language configuration
        let core_config = lang_config.to_core_config(py)?;

        use sakurs_core::domain::language::ConfigurableLanguageRules;
        use std::sync::Arc;

        let language_rules = ConfigurableLanguageRules::from_config(&core_config)
            .map_err(|e| InternalError::ConfigurationError(e.to_string()))?;
        let language_rules_arc: Arc<dyn sakurs_core::domain::language::LanguageRules> =
            Arc::new(language_rules);

        (
            Config::builder()
                .language("en")
                .map_err(|e| InternalError::ConfigurationError(e.to_string()))?,
            Some(language_rules_arc),
        )
    } else {
        // Use built-in language
        let lang_code = match language.unwrap_or("en").to_lowercase().as_str() {
            "en" | "english" => "en",
            "ja" | "japanese" => "ja",
            _ => {
                return Err(InternalError::UnsupportedLanguage(
                    language.unwrap_or("unknown").to_string(),
                )
                .into())
            }
        };
        (
            Config::builder()
                .language(lang_code)
                .map_err(|e| InternalError::ConfigurationError(e.to_string()))?,
            None,
        )
    };

    // Configure for streaming
    let chunk_size_bytes = chunk_size_mb * 1024 * 1024;
    config_builder = config_builder
        .chunk_size(chunk_size_bytes)
        .overlap_size(overlap_size)
        .threads(Some(1)); // Sequential for streaming

    let config = config_builder
        .build()
        .map_err(|e| InternalError::ConfigurationError(e.to_string()))?;

    // Create processor
    let processor = if let Some(rules) = custom_rules {
        SentenceProcessor::with_custom_rules(config, rules)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    } else {
        SentenceProcessor::with_config(config)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    };

    // Create iterator
    let iterator = SentenceIterator::new_internal(preserve_whitespace);

    // Determine input type and start streaming
    let py_input = PyInput::from_py_object(py, input)?;

    match py_input {
        PyInput::Text(text) => {
            // For text input, process in chunks
            stream_text_chunks(&text, &iterator, &processor, chunk_size_bytes)?;
        }
        PyInput::Path(path) => {
            // For file paths, stream from file
            stream_from_file(&path, &iterator, &processor, chunk_size_bytes, encoding)?;
        }
        PyInput::Bytes(bytes) => {
            // Convert bytes to string and process
            let text = String::from_utf8(bytes)
                .map_err(|e| InternalError::EncodingError(e.to_string()))?;
            stream_text_chunks(&text, &iterator, &processor, chunk_size_bytes)?;
        }
        PyInput::FileObject(obj) => {
            // Stream from file-like object
            let obj_bound = obj.bind(py);
            stream_from_file_object(
                py,
                obj_bound,
                &iterator,
                &processor,
                chunk_size_bytes,
                encoding,
            )?;
        }
    }

    Ok(iterator)
}

/// Stream text in chunks
fn stream_text_chunks(
    text: &str,
    iterator: &SentenceIterator,
    processor: &SentenceProcessor,
    chunk_size: usize,
) -> PyResult<()> {
    use crate::iterator::{flush_buffer, process_text_incrementally};

    let state = iterator.get_state();

    // Process text in chunks
    let mut start = 0;
    while start < text.len() {
        let end = (start + chunk_size).min(text.len());

        // Find a safe UTF-8 boundary
        let mut safe_end = end;
        while safe_end < text.len() && !text.is_char_boundary(safe_end) {
            safe_end += 1;
        }

        let chunk = &text[start..safe_end];
        process_text_incrementally(chunk, &state, processor)?;

        start = safe_end;
    }

    // Flush any remaining text
    flush_buffer(&state, processor)?;

    Ok(())
}

/// Stream from a file
fn stream_from_file(
    path: &PathBuf,
    iterator: &SentenceIterator,
    processor: &SentenceProcessor,
    chunk_size: usize,
    encoding: &str,
) -> PyResult<()> {
    use crate::iterator::{flush_buffer, process_text_incrementally};
    use encoding_rs::Encoding;

    let state = iterator.get_state();

    // Open file
    let file = File::open(path).map_err(|e| InternalError::IoError(e.to_string()))?;

    // Handle different encodings
    if encoding == "utf-8" {
        // Fast path for UTF-8
        let reader = BufReader::new(file);
        let mut buffer = String::new();

        for line in reader.lines() {
            let line = line.map_err(|e| InternalError::IoError(e.to_string()))?;
            buffer.push_str(&line);
            buffer.push('\n');

            // Process when buffer is large enough
            if buffer.len() >= chunk_size {
                process_text_incrementally(&buffer, &state, processor)?;
                buffer.clear();
            }
        }

        // Process any remaining text
        if !buffer.is_empty() {
            process_text_incrementally(&buffer, &state, processor)?;
        }
    } else {
        // Handle other encodings
        let encoding_obj = Encoding::for_label(encoding.as_bytes())
            .ok_or_else(|| InternalError::EncodingError(format!("Unknown encoding: {encoding}")))?;

        let mut reader = BufReader::new(file);
        let mut raw_buffer = vec![0u8; chunk_size];
        let mut text_buffer = String::new();

        loop {
            let bytes_read = reader
                .read(&mut raw_buffer)
                .map_err(|e| InternalError::IoError(e.to_string()))?;

            if bytes_read == 0 {
                break;
            }

            let (decoded, _, _) = encoding_obj.decode(&raw_buffer[..bytes_read]);
            text_buffer.push_str(&decoded);

            // Process when buffer is large enough
            if text_buffer.len() >= chunk_size {
                process_text_incrementally(&text_buffer, &state, processor)?;
                text_buffer.clear();
            }
        }

        // Process any remaining text
        if !text_buffer.is_empty() {
            process_text_incrementally(&text_buffer, &state, processor)?;
        }
    }

    // Flush any remaining text
    flush_buffer(&state, processor)?;

    Ok(())
}

/// Stream from a Python file-like object
fn stream_from_file_object(
    _py: Python,
    obj: &Bound<'_, PyAny>,
    iterator: &SentenceIterator,
    processor: &SentenceProcessor,
    chunk_size: usize,
    encoding: &str,
) -> PyResult<()> {
    use crate::iterator::{flush_buffer, process_text_incrementally};
    use pyo3::types::PyBytes;

    let state = iterator.get_state();

    // Try to seek to beginning if possible
    let _ = obj.call_method1("seek", (0,));

    // Read a small amount first to determine if it's text or binary mode
    let first_chunk = obj.call_method1("read", (1,))?;

    // Check if we got an empty result (empty file)
    let is_empty = if let Ok(s) = first_chunk.extract::<String>() {
        s.is_empty()
    } else if let Ok(b) = first_chunk.extract::<Vec<u8>>() {
        b.is_empty()
    } else {
        // Neither string nor bytes - unexpected type
        return Err(InternalError::InvalidInput(
            "file.read() returned neither str nor bytes".to_string(),
        )
        .into());
    };

    if is_empty {
        // Empty file, nothing to process
        return Ok(());
    }

    // Determine if it's binary mode based on the type returned
    let is_binary_mode = first_chunk.downcast::<PyBytes>().is_ok();

    // Seek back to the beginning to read all data
    let _ = obj.call_method1("seek", (0,));

    if is_binary_mode {
        // Binary mode - decode bytes
        use encoding_rs::Encoding;

        let encoding_obj = Encoding::for_label(encoding.as_bytes())
            .ok_or_else(|| InternalError::EncodingError(format!("Unknown encoding: {encoding}")))?;

        let mut text_buffer = String::new();

        // Read and process chunks
        loop {
            let chunk_obj = obj.call_method1("read", (chunk_size,))?;
            let chunk_bytes: Vec<u8> = chunk_obj.extract()?;
            if chunk_bytes.is_empty() {
                break;
            }

            let (decoded, _, _) = encoding_obj.decode(&chunk_bytes);
            text_buffer.push_str(&decoded);
        }

        // Process all accumulated text using the same logic as text input
        if !text_buffer.is_empty() {
            stream_text_chunks(&text_buffer, iterator, processor, chunk_size)?;
        }
        // Return early to avoid duplicate flush
        return Ok(());
    } else {
        // Text mode - read strings directly
        let mut buffer = String::new();

        // Read and process chunks
        loop {
            let chunk_obj = obj.call_method1("read", (chunk_size,))?;

            if let Ok(chunk_str) = chunk_obj.downcast::<PyString>() {
                let chunk = chunk_str.extract::<String>()?;
                if chunk.is_empty() {
                    break;
                }

                buffer.push_str(&chunk);

                // Process when buffer is large enough
                if buffer.len() >= chunk_size {
                    process_text_incrementally(&buffer, &state, processor)?;
                    buffer.clear();
                }
            } else {
                break;
            }
        }

        // Process any remaining text
        if !buffer.is_empty() {
            process_text_incrementally(&buffer, &state, processor)?;
        }
    }

    // Flush any remaining text
    flush_buffer(&state, processor)?;

    Ok(())
}

/// Create an iterator adapter for existing PyIterator
#[allow(dead_code)]
pub fn adapt_python_iterator(
    _py: Python,
    iter: &Bound<'_, PyIterator>,
    processor: &SentenceProcessor,
    preserve_whitespace: bool,
) -> PyResult<SentenceIterator> {
    use crate::iterator::{flush_buffer, process_text_incrementally};

    let iterator = SentenceIterator::new_internal(preserve_whitespace);
    let state = iterator.get_state();

    // Process each item from the Python iterator
    for item in iter {
        let item = item?;
        if let Ok(text) = item.extract::<String>() {
            process_text_incrementally(&text, &state, processor)?;
        }
    }

    // Flush remaining text
    flush_buffer(&state, processor)?;

    Ok(iterator)
}
