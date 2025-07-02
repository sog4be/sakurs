//! Markdown output formatter

use super::OutputFormatter;
use anyhow::Result;
use std::io::Write;

/// Markdown formatter - outputs sentences as a markdown list
pub struct MarkdownFormatter<W: Write> {
    writer: W,
    sentence_count: usize,
}

impl<W: Write> MarkdownFormatter<W> {
    /// Create a new markdown formatter
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            sentence_count: 0,
        }
    }
}

impl<W: Write + Send + Sync> OutputFormatter for MarkdownFormatter<W> {
    fn format_sentence(&mut self, sentence: &str, _offset: usize) -> Result<()> {
        self.sentence_count += 1;
        writeln!(self.writer, "{}. {}", self.sentence_count, sentence.trim())?;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        writeln!(self.writer)?;
        writeln!(self.writer, "---")?;
        writeln!(self.writer, "*Total sentences: {}*", self.sentence_count)?;
        self.writer.flush()?;
        Ok(())
    }
}
