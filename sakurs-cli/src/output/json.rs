//! JSON output formatter

use super::OutputFormatter;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// JSON formatter - outputs sentences as JSON array
pub struct JsonFormatter<W: Write> {
    writer: W,
    sentences: Vec<SentenceData>,
}

/// Data structure for JSON output
#[derive(Debug, Serialize, Deserialize)]
pub struct SentenceData {
    /// The sentence text
    pub text: String,
    /// Starting offset in the original text
    pub offset: usize,
    /// Length of the sentence
    pub length: usize,
}

impl<W: Write> JsonFormatter<W> {
    /// Create a new JSON formatter
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            sentences: Vec::new(),
        }
    }
}

impl<W: Write + Send + Sync> OutputFormatter for JsonFormatter<W> {
    fn format_sentence(&mut self, sentence: &str, offset: usize) -> Result<()> {
        self.sentences.push(SentenceData {
            text: sentence.trim().to_string(),
            offset,
            length: sentence.len(),
        });
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        serde_json::to_writer_pretty(&mut self.writer, &self.sentences)?;
        writeln!(self.writer)?;
        self.writer.flush()?;
        Ok(())
    }
}
