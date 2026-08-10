//! Streaming functionality for processing large texts

use crate::exceptions::InternalError;
use crate::input::PyInput;
use crate::iterator::SentenceIterator;
use crate::language_config::LanguageConfig;
use crate::output::normalize_sentence_text;
use encoding_rs::{CoderResult, Decoder, Encoding};
use pyo3::prelude::*;
use pyo3::types::PyIterator;
use sakurs_core::{Config, SentenceProcessor};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

/// Default chunk size for streaming (10MB)
#[allow(dead_code)]
const DEFAULT_CHUNK_SIZE_MB: usize = 10;

/// Default overlap size for streaming (1KB)
#[allow(dead_code)]
const DEFAULT_OVERLAP_SIZE: usize = 1024;

/// Create an iterator over sentences computed from the complete input.
#[allow(clippy::too_many_arguments)]
pub fn create_iter_split_iterator(
    py: Python,
    input: &Bound<'_, PyAny>,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    threads: Option<usize>,
    chunk_size: Option<usize>,
    preserve_whitespace: bool,
    encoding: &str,
) -> PyResult<SentenceIterator> {
    // Build processor configuration
    let (mut config_builder, custom_language) = if let Some(lang_config) = language_config {
        // Use custom language configuration
        let core_config = lang_config.to_core_config(py)?;

        (
            Config::builder()
                .language("en")
                .map_err(|e| InternalError::ConfigurationError(e.to_string()))?,
            Some(core_config),
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

    // Configure for iter_split (uses normal processing settings)
    if let Some(threads) = threads {
        config_builder = config_builder.threads(Some(threads));
    }
    if let Some(chunk_size) = chunk_size {
        config_builder = config_builder.chunk_size(chunk_size);
    }

    let config = config_builder
        .build()
        .map_err(|e| InternalError::ConfigurationError(e.to_string()))?;

    // Create processor
    let processor = if let Some(language) = custom_language {
        SentenceProcessor::with_language_config(config, &language)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    } else {
        SentenceProcessor::with_config(config)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    };

    create_iter_split_iterator_from_processor(py, input, &processor, preserve_whitespace, encoding)
}

/// Create an iterator using an already configured sentence processor.
pub(crate) fn create_iter_split_iterator_from_processor(
    py: Python,
    input: &Bound<'_, PyAny>,
    processor: &SentenceProcessor,
    preserve_whitespace: bool,
    encoding: &str,
) -> PyResult<SentenceIterator> {
    let iterator = SentenceIterator::new_internal(preserve_whitespace);

    // Process all input at once and populate iterator
    let py_input = PyInput::from_py_object(py, input)?;

    // Get the full text from input
    let text = match py_input {
        PyInput::Text(text) => text,
        PyInput::Path(path) => {
            std::fs::read_to_string(&path).map_err(|e| InternalError::IoError(e.to_string()))?
        }
        PyInput::Bytes(bytes) => {
            String::from_utf8(bytes).map_err(|e| InternalError::EncodingError(e.to_string()))?
        }
        PyInput::FileObject(obj) => {
            // Read entire content from file-like object
            let obj_bound = obj.bind(py);
            read_all_from_file_object(py, obj_bound, encoding)?
        }
    };

    // Process the entire text at once
    let input = sakurs_core::Input::from_text(&text);
    let output = processor
        .process(input)
        .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

    // Convert boundaries to sentences and add to iterator
    let mut sentences = Vec::new();
    let mut last_pos = 0;

    for boundary in output.boundaries {
        let raw_sentence = &text[last_pos..boundary.offset];
        if let Some(sentence) = normalize_sentence_text(raw_sentence, preserve_whitespace) {
            sentences.push(sentence);
        }
        last_pos = boundary.offset;
    }

    // Add any remaining text
    if last_pos < text.len() {
        let raw_sentence = &text[last_pos..];
        if let Some(sentence) = normalize_sentence_text(raw_sentence, preserve_whitespace) {
            sentences.push(sentence);
        }
    }

    // Add all sentences to the iterator
    iterator.add_sentences(sentences)?;
    iterator.mark_exhausted()?;

    Ok(iterator)
}

/// Read all content from a file-like object
fn read_all_from_file_object(
    _py: Python,
    obj: &Bound<'_, PyAny>,
    encoding: &str,
) -> PyResult<String> {
    use pyo3::types::PyBytes;

    // Try to seek to beginning if possible
    let _ = obj.call_method1("seek", (0,));

    // Read all content at once
    let content = obj.call_method0("read")?;

    // Check if it's bytes or string
    if let Ok(text) = content.extract::<String>() {
        Ok(text)
    } else if let Ok(bytes_obj) = content.cast::<PyBytes>() {
        // It's bytes, decode it
        let bytes = bytes_obj.extract::<Vec<u8>>()?;
        use encoding_rs::Encoding;

        let encoding_obj = Encoding::for_label(encoding.as_bytes())
            .ok_or_else(|| InternalError::EncodingError(format!("Unknown encoding: {encoding}")))?;

        let (decoded, _, _) = encoding_obj.decode(&bytes);
        Ok(decoded.to_string())
    } else {
        Err(
            InternalError::InvalidInput("file.read() returned neither str nor bytes".to_string())
                .into(),
        )
    }
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

/// Create an incremental iterator for large files.
pub fn create_large_file_iterator(
    py: Python,
    file_path: &Path,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    max_memory_mb: usize,
    overlap_size: usize,
    encoding: &str,
) -> PyResult<LargeFileIterator> {
    // Validate file path
    if !file_path.exists() {
        return Err(InternalError::FileNotFound(file_path.display().to_string()).into());
    }

    // Build processor configuration for incremental processing
    let (mut config_builder, custom_language) = if let Some(lang_config) = language_config {
        // Use custom language configuration
        let core_config = lang_config.to_core_config(py)?;

        (
            Config::builder()
                .language("en")
                .map_err(|e| InternalError::ConfigurationError(e.to_string()))?,
            Some(core_config),
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

    // Derive a target chunk size from the requested memory budget.
    let chunk_size = (max_memory_mb * 1024 * 1024) / 4; // Reserve memory for processing
    config_builder = config_builder.chunk_size(chunk_size).threads(Some(1)); // Single thread for streaming

    let config = config_builder
        .build()
        .map_err(|e| InternalError::ConfigurationError(e.to_string()))?;

    // Create processor
    let processor = if let Some(language) = custom_language {
        SentenceProcessor::with_language_config(config, &language)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    } else {
        SentenceProcessor::with_config(config)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    };

    Ok(LargeFileIterator::new(
        file_path.to_path_buf(),
        processor,
        chunk_size,
        overlap_size,
        encoding.to_string(),
    ))
}

/// Iterator for incremental large-file processing.
#[pyclass]
pub struct LargeFileIterator {
    file_path: PathBuf,
    processor: SentenceProcessor,
    chunk_size: usize,
    overlap_size: usize,
    encoding: String,
    reader: Option<BufReader<File>>,
    decoder: Option<Decoder>,
    carry_over: String,
    sentence_buffer: VecDeque<String>,
    exhausted: bool,
}

impl LargeFileIterator {
    fn new(
        file_path: PathBuf,
        processor: SentenceProcessor,
        chunk_size: usize,
        overlap_size: usize,
        encoding: String,
    ) -> Self {
        Self {
            file_path,
            processor,
            chunk_size,
            overlap_size,
            encoding,
            reader: None,
            decoder: None,
            carry_over: String::new(),
            sentence_buffer: VecDeque::new(),
            exhausted: false,
        }
    }
}

#[pymethods]
impl LargeFileIterator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<String>> {
        use pyo3::exceptions::PyStopIteration;

        loop {
            // Return buffered sentences first.
            if let Some(sentence) = self.sentence_buffer.pop_front() {
                return Ok(Some(sentence));
            }

            if self.exhausted {
                return Err(PyStopIteration::new_err(()));
            }

            // Initialize reader on first call.
            if self.reader.is_none() {
                let file = File::open(&self.file_path).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        InternalError::FileNotFound(self.file_path.display().to_string())
                    } else {
                        InternalError::IoError(e.to_string())
                    }
                })?;
                self.reader = Some(BufReader::new(file));
            }

            let read_target = next_buffer_target(self.chunk_size, self.carry_over.len());
            let mut buffer = String::with_capacity(read_target);
            buffer.push_str(&self.carry_over);
            let mut reached_eof = false;

            // Grow an oversized carry geometrically. This keeps repeated
            // whole-buffer scans linear when a long region has no boundary.
            if self.encoding == "utf-8" {
                let reader = self.reader.as_mut().unwrap();
                let mut line_buffer = String::new();
                let mut read_any = false;
                while buffer.len() < read_target || !read_any {
                    match reader.read_line(&mut line_buffer) {
                        Ok(0) => {
                            reached_eof = true;
                            break;
                        }
                        Ok(_) => {
                            read_any = true;
                            buffer.push_str(&line_buffer);
                            line_buffer.clear();
                        }
                        Err(e) => return Err(InternalError::IoError(e.to_string()).into()),
                    }
                }
            } else {
                if self.decoder.is_none() {
                    let encoding_obj =
                        Encoding::for_label(self.encoding.as_bytes()).ok_or_else(|| {
                            InternalError::EncodingError(format!(
                                "Unknown encoding: {}",
                                self.encoding
                            ))
                        })?;
                    self.decoder = Some(encoding_obj.new_decoder());
                }

                let bytes_to_read = read_target.saturating_sub(buffer.len()).max(1);
                let mut raw_buffer = vec![0u8; bytes_to_read];
                let bytes_read = self
                    .reader
                    .as_mut()
                    .unwrap()
                    .read(&mut raw_buffer)
                    .map_err(|e| InternalError::IoError(e.to_string()))?;
                reached_eof = bytes_read == 0;

                let decoded = decode_chunk(
                    self.decoder.as_mut().unwrap(),
                    &raw_buffer[..bytes_read],
                    reached_eof,
                )?;
                buffer.push_str(&decoded);
            }

            if buffer.is_empty() && reached_eof {
                self.exhausted = true;
                self.carry_over.clear();
                return Err(PyStopIteration::new_err(()));
            }

            let output = self
                .processor
                .process(sakurs_core::Input::from_text(&buffer))
                .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

            if output.boundaries.is_empty() {
                if reached_eof {
                    self.exhausted = true;
                    self.carry_over.clear();
                    if let Some(sentence) = normalize_sentence_text(&buffer, false) {
                        return Ok(Some(sentence));
                    }
                    return Err(PyStopIteration::new_err(()));
                }
                self.carry_over = buffer;
                continue;
            }

            // At EOF every discovered boundary is safe. Otherwise keep the
            // configured overlap to resolve contexts crossing the next read.
            let safe_boundary_pos = if reached_eof {
                output.boundaries.last().unwrap().offset
            } else {
                let overlap_start = buffer.len().saturating_sub(self.overlap_size);
                output
                    .boundaries
                    .iter()
                    .rposition(|boundary| boundary.offset < overlap_start)
                    .map(|index| output.boundaries[index].offset)
                    .unwrap_or(0)
            };

            if safe_boundary_pos == 0 {
                self.carry_over = buffer;
                continue;
            }

            let mut last_pos = 0;
            for boundary in &output.boundaries {
                if boundary.offset <= safe_boundary_pos {
                    if let Some(sentence) =
                        normalize_sentence_text(&buffer[last_pos..boundary.offset], false)
                    {
                        self.sentence_buffer.push_back(sentence);
                    }
                    last_pos = boundary.offset;
                }
            }

            self.carry_over = buffer[safe_boundary_pos..].to_string();

            if reached_eof {
                if let Some(sentence) = normalize_sentence_text(&self.carry_over, false) {
                    self.sentence_buffer.push_back(sentence);
                }
                self.carry_over.clear();
                self.exhausted = true;
            }
        }
    }
}

fn next_buffer_target(chunk_size: usize, carry_len: usize) -> usize {
    if carry_len >= chunk_size {
        carry_len.saturating_mul(2)
    } else {
        chunk_size
    }
}

fn decode_chunk(decoder: &mut Decoder, bytes: &[u8], reached_eof: bool) -> PyResult<String> {
    let capacity = decoder.max_utf8_buffer_length(bytes.len()).ok_or_else(|| {
        InternalError::EncodingError("Decoded chunk size exceeds platform limits".to_string())
    })?;
    let mut decoded = String::with_capacity(capacity);
    let mut total_read = 0;

    loop {
        let (result, bytes_read, _) =
            decoder.decode_to_string(&bytes[total_read..], &mut decoded, reached_eof);
        total_read += bytes_read;

        match result {
            CoderResult::InputEmpty => return Ok(decoded),
            CoderResult::OutputFull => {
                let additional = decoder
                    .max_utf8_buffer_length(bytes.len() - total_read)
                    .ok_or_else(|| {
                        InternalError::EncodingError(
                            "Decoded chunk size exceeds platform limits".to_string(),
                        )
                    })?;
                decoded.reserve(additional.max(4));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::next_buffer_target;

    #[test]
    fn oversized_carry_grows_geometrically() {
        assert_eq!(next_buffer_target(256, 0), 256);
        assert_eq!(next_buffer_target(256, 255), 256);
        assert_eq!(next_buffer_target(256, 256), 512);
        assert_eq!(next_buffer_target(256, 1024), 2048);
    }
}
