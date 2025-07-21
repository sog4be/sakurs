//! Streaming functionality for processing large texts

use crate::exceptions::InternalError;
use crate::input::PyInput;
use crate::iterator::SentenceIterator;
use crate::language_config::LanguageConfig;
use pyo3::prelude::*;
use pyo3::types::PyIterator;
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

/// Create an iterator that loads all input but yields incrementally  
pub fn create_iter_split_iterator(
    py: Python,
    input: &Bound<'_, PyAny>,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    threads: Option<usize>,
    chunk_size: Option<usize>,
    encoding: &str,
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
    let processor = if let Some(rules) = custom_rules {
        SentenceProcessor::with_custom_rules(config, rules)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    } else {
        SentenceProcessor::with_config(config)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?
    };

    // Create iterator
    let iterator = SentenceIterator::new_internal(false); // No whitespace preservation for now

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
        let sentence = text[last_pos..boundary.offset].trim().to_string();
        if !sentence.is_empty() {
            sentences.push(sentence);
        }
        last_pos = boundary.offset;
    }

    // Add any remaining text
    if last_pos < text.len() {
        let sentence = text[last_pos..].trim().to_string();
        if !sentence.is_empty() {
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
    } else if let Ok(bytes_obj) = content.downcast::<PyBytes>() {
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

/// Create a memory-efficient iterator for large files
pub fn create_large_file_iterator(
    py: Python,
    file_path: &str,
    language: Option<&str>,
    language_config: Option<LanguageConfig>,
    max_memory_mb: usize,
    overlap_size: usize,
    encoding: &str,
) -> PyResult<LargeFileIterator> {
    use std::path::Path;

    // Validate file path
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(InternalError::FileNotFound(file_path.to_string()).into());
    }

    // Build processor configuration for memory-efficient processing
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

    // Configure for memory-efficient processing
    let chunk_size = (max_memory_mb * 1024 * 1024) / 4; // Reserve memory for processing
    config_builder = config_builder
        .chunk_size(chunk_size)
        .overlap_size(overlap_size)
        .threads(Some(1)); // Single thread for streaming

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

    Ok(LargeFileIterator::new(
        PathBuf::from(file_path),
        processor,
        chunk_size,
        overlap_size,
        encoding.to_string(),
    ))
}

/// Iterator for memory-efficient large file processing
#[pyclass]
pub struct LargeFileIterator {
    file_path: PathBuf,
    processor: SentenceProcessor,
    chunk_size: usize,
    overlap_size: usize,
    encoding: String,
    reader: Option<BufReader<File>>,
    carry_over: String,
    sentence_buffer: Vec<String>,
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
            carry_over: String::new(),
            sentence_buffer: Vec::new(),
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

        // Return buffered sentences first
        if !self.sentence_buffer.is_empty() {
            return Ok(Some(self.sentence_buffer.remove(0)));
        }

        if self.exhausted {
            return Err(PyStopIteration::new_err(()));
        }

        // Initialize reader on first call
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

        let reader = self.reader.as_mut().unwrap();

        // Read chunk from file
        let mut buffer = String::with_capacity(self.chunk_size);
        buffer.push_str(&self.carry_over);

        // Read until we have enough data or reach EOF
        if self.encoding == "utf-8" {
            // Fast path for UTF-8
            let mut line_buffer = String::new();
            while buffer.len() < self.chunk_size {
                match reader.read_line(&mut line_buffer) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        buffer.push_str(&line_buffer);
                        line_buffer.clear();
                    }
                    Err(e) => return Err(InternalError::IoError(e.to_string()).into()),
                }
            }
        } else {
            // Handle other encodings
            use encoding_rs::Encoding;

            let encoding_obj = Encoding::for_label(self.encoding.as_bytes()).ok_or_else(|| {
                InternalError::EncodingError(format!("Unknown encoding: {}", self.encoding))
            })?;

            let mut raw_buffer = vec![0u8; self.chunk_size];
            match reader.read(&mut raw_buffer) {
                Ok(0) => {} // EOF
                Ok(n) => {
                    let (decoded, _, _) = encoding_obj.decode(&raw_buffer[..n]);
                    buffer.push_str(&decoded);
                }
                Err(e) => return Err(InternalError::IoError(e.to_string()).into()),
            }
        }

        // Check if we're at EOF
        if buffer.len() <= self.carry_over.len() {
            self.exhausted = true;
            if !self.carry_over.is_empty() {
                // Process final carry-over as a sentence
                let sentence = self.carry_over.trim().to_string();
                self.carry_over.clear();
                if !sentence.is_empty() {
                    return Ok(Some(sentence));
                }
            }
            return Err(PyStopIteration::new_err(()));
        }

        // Process the chunk
        let input = sakurs_core::Input::from_text(&buffer);
        let output = self
            .processor
            .process(input)
            .map_err(|e| InternalError::ProcessingError(e.to_string()))?;

        if output.boundaries.is_empty() {
            // No boundaries found, carry over entire buffer
            self.carry_over = buffer;
            return self.__next__();
        }

        // Find the last safe boundary (not in overlap zone)
        let safe_boundary_pos = if buffer.len() < self.chunk_size {
            // Last chunk, process all boundaries
            output.boundaries.last().unwrap().offset
        } else {
            // Find last boundary before overlap zone
            let overlap_start = buffer.len().saturating_sub(self.overlap_size);
            output
                .boundaries
                .iter()
                .rposition(|b| b.offset < overlap_start)
                .map(|idx| output.boundaries[idx].offset)
                .unwrap_or(0)
        };

        if safe_boundary_pos == 0 {
            // No safe boundary, carry over entire buffer
            self.carry_over = buffer;
            return self.__next__();
        }

        // Extract sentences up to safe boundary
        let mut last_pos = 0;
        for boundary in &output.boundaries {
            if boundary.offset <= safe_boundary_pos {
                let sentence = buffer[last_pos..boundary.offset].trim().to_string();
                if !sentence.is_empty() {
                    self.sentence_buffer.push(sentence);
                }
                last_pos = boundary.offset;
            }
        }

        // Update carry-over with remaining text
        self.carry_over = buffer[safe_boundary_pos..].to_string();

        // Return first sentence from buffer
        if !self.sentence_buffer.is_empty() {
            Ok(Some(self.sentence_buffer.remove(0)))
        } else {
            self.__next__()
        }
    }
}
