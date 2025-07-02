//! Plain text output formatter

use super::OutputFormatter;
use anyhow::Result;
use std::io::{self, Write};

/// Plain text formatter - outputs one sentence per line
pub struct TextFormatter<W: Write> {
    writer: W,
}

impl<W: Write> TextFormatter<W> {
    /// Create a new text formatter
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl TextFormatter<io::Stdout> {
    /// Create a formatter that writes to stdout
    pub fn stdout() -> Self {
        Self::new(io::stdout())
    }
}

impl<W: Write + Send + Sync> OutputFormatter for TextFormatter<W> {
    fn format_sentence(&mut self, sentence: &str, _offset: usize) -> Result<()> {
        writeln!(self.writer, "{}", sentence.trim())?;
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}
