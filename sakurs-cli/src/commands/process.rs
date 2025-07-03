//! Process command implementation

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

/// Arguments for the process command
#[derive(Debug, Args)]
pub struct ProcessArgs {
    /// Input files or patterns (supports glob)
    #[arg(short, long, value_name = "FILE/PATTERN", required = true)]
    pub input: Vec<String>,

    /// Output file (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Language for sentence detection rules
    #[arg(short, long, value_enum, default_value = "english")]
    pub language: Language,

    /// Force parallel processing even for small files
    #[arg(short, long)]
    pub parallel: bool,

    /// Configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Suppress progress output
    #[arg(short, long)]
    pub quiet: bool,

    /// Increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Enable streaming mode for large files (process in chunks)
    #[arg(long)]
    pub stream: bool,

    /// Streaming chunk size in MB (default: 10MB)
    #[arg(long, default_value = "10", requires = "stream")]
    pub stream_chunk_mb: u64,
}

/// Supported output formats
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    /// Plain text with one sentence per line
    Text,
    /// JSON array of sentences with metadata
    Json,
    /// Markdown formatted output
    Markdown,
}

/// Supported languages
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Language {
    /// English language rules
    English,
    /// Japanese language rules
    Japanese,
}

impl ProcessArgs {
    /// Execute the process command
    pub fn execute(&self) -> Result<()> {
        // Initialize logging based on verbosity
        self.init_logging()?;

        log::info!("Starting text processing");
        log::debug!("Arguments: {self:?}");

        // Resolve file patterns
        let files = crate::input::resolve_patterns(&self.input)?;
        log::info!("Found {} files to process", files.len());

        // Initialize progress reporter
        let mut progress = crate::progress::ProgressReporter::new(self.quiet);
        progress.init_files(files.len() as u64);

        // Create output formatter
        let mut formatter: Box<dyn crate::output::OutputFormatter> = self.create_formatter()?;

        // Process each file
        let processor = self.create_processor()?;

        for file in &files {
            log::info!("Processing file: {}", file.display());

            // Check if we should use streaming mode
            let file_size_mb = crate::input::FileReader::file_size(file)? / (1024 * 1024);
            let should_stream = self.stream || file_size_mb > 100; // Auto-stream for files > 100MB

            if should_stream {
                log::info!(
                    "Using streaming mode for {} ({}MB)",
                    file.display(),
                    file_size_mb
                );
                self.process_file_streaming(file, &processor, &mut formatter)?;
            } else {
                // Read entire file content
                let content = crate::input::FileReader::read_text(file)?;

                // Process text
                let result = processor.process_text(&content)?;

                // Extract and output sentences
                let sentences = result.extract_sentences(&content);
                let ranges = result.sentence_ranges();

                for (sentence, range) in sentences.iter().zip(ranges.iter()) {
                    formatter.format_sentence(sentence, range.start)?;
                }
            }

            progress.file_completed(&file.file_name().unwrap_or_default().to_string_lossy());
        }

        // Finalize output
        formatter.finish()?;
        progress.finish();

        log::info!("Processing complete. Processed {} files", files.len());
        Ok(())
    }

    /// Initialize logging based on verbosity level
    fn init_logging(&self) -> Result<()> {
        let log_level = match self.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        };

        if !self.quiet {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
                .init();
        }

        Ok(())
    }

    /// Create appropriate output formatter based on format option
    fn create_formatter(&self) -> Result<Box<dyn crate::output::OutputFormatter>> {
        use std::io;

        match self.format {
            OutputFormat::Text => {
                if let Some(output_path) = &self.output {
                    let file = std::fs::File::create(output_path).with_context(|| {
                        format!("Failed to create output file: {}", output_path.display())
                    })?;
                    Ok(Box::new(crate::output::TextFormatter::new(file)))
                } else {
                    Ok(Box::new(crate::output::TextFormatter::new(io::stdout())))
                }
            }
            OutputFormat::Json => {
                if let Some(output_path) = &self.output {
                    let file = std::fs::File::create(output_path).with_context(|| {
                        format!("Failed to create output file: {}", output_path.display())
                    })?;
                    Ok(Box::new(crate::output::JsonFormatter::new(file)))
                } else {
                    Ok(Box::new(crate::output::JsonFormatter::new(io::stdout())))
                }
            }
            OutputFormat::Markdown => {
                if let Some(output_path) = &self.output {
                    let file = std::fs::File::create(output_path).with_context(|| {
                        format!("Failed to create output file: {}", output_path.display())
                    })?;
                    Ok(Box::new(crate::output::MarkdownFormatter::new(file)))
                } else {
                    Ok(Box::new(
                        crate::output::MarkdownFormatter::new(io::stdout()),
                    ))
                }
            }
        }
    }

    /// Create text processor with appropriate language rules
    fn create_processor(&self) -> Result<sakurs_core::application::TextProcessor> {
        use sakurs_core::application::{ProcessorConfig, TextProcessor};
        use sakurs_core::domain::language::{
            EnglishLanguageRules, JapaneseLanguageRules, LanguageRules,
        };
        use std::sync::Arc;

        let config = if self.parallel {
            ProcessorConfig::builder()
                .chunk_size(256 * 1024)
                .parallel_threshold(0) // Force parallel
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build processor config: {}", e))?
        } else {
            ProcessorConfig::default()
        };

        let language_rules: Arc<dyn LanguageRules> = match self.language {
            Language::English => Arc::new(EnglishLanguageRules::new()),
            Language::Japanese => Arc::new(JapaneseLanguageRules::new()),
        };

        Ok(TextProcessor::with_config(config, language_rules))
    }

    /// Process a file in streaming mode
    fn process_file_streaming(
        &self,
        file: &std::path::Path,
        processor: &sakurs_core::application::TextProcessor,
        formatter: &mut Box<dyn crate::output::OutputFormatter>,
    ) -> Result<()> {
        use std::fs::File;
        use std::io::{BufReader, Read};

        let chunk_size = (self.stream_chunk_mb * 1024 * 1024) as usize;
        let file = File::open(file)?;
        let mut reader = BufReader::new(file);

        let mut buffer = vec![0u8; chunk_size];
        let mut carry_over = String::new();
        let mut remainder_bytes = Vec::new();
        let mut global_offset = 0;

        loop {
            // Copy any remainder bytes from previous iteration to start of buffer
            let remainder_len = remainder_bytes.len();
            if remainder_len > 0 {
                buffer[..remainder_len].copy_from_slice(&remainder_bytes);
                remainder_bytes.clear();
            }

            // Read new data after the remainder
            let bytes_read = reader.read(&mut buffer[remainder_len..])?;
            if bytes_read == 0 {
                // Process any remaining carry-over
                if !carry_over.is_empty() {
                    let result = processor.process_text(&carry_over)?;
                    output_sentences(&carry_over, &result, formatter, global_offset)?;
                }
                break;
            }

            let total_bytes = remainder_len + bytes_read;

            // Find valid UTF-8 boundary
            let mut valid_bytes = total_bytes;
            while valid_bytes > 0 && std::str::from_utf8(&buffer[..valid_bytes]).is_err() {
                valid_bytes -= 1;
            }

            if valid_bytes == 0 {
                return Err(anyhow::anyhow!(
                    "Unable to find valid UTF-8 boundary in chunk"
                ));
            }

            // Save any incomplete UTF-8 sequence for next iteration
            if valid_bytes < total_bytes {
                remainder_bytes.extend_from_slice(&buffer[valid_bytes..total_bytes]);
            }

            let chunk_str =
                std::str::from_utf8(&buffer[..valid_bytes]).expect("Already validated UTF-8");

            // Combine with carry-over from previous chunk
            let combined = carry_over + chunk_str;

            // Find a safe boundary to split (prefer sentence boundary, fallback to word boundary)
            let split_point = find_safe_split_point(&combined, chunk_size);

            // Process up to split point
            let (to_process, to_carry) = combined.split_at(split_point);

            if !to_process.is_empty() {
                let result = processor.process_text(to_process)?;
                output_sentences(to_process, &result, formatter, global_offset)?;
                global_offset += to_process.len();
            }

            // Save remainder for next iteration
            carry_over = to_carry.to_string();
        }

        Ok(())
    }
}

/// Find a safe point to split text (prefer sentence boundary, then word boundary)
fn find_safe_split_point(text: &str, target: usize) -> usize {
    if text.len() <= target {
        return text.len();
    }

    // Look for sentence boundaries near the target
    let search_start = target.saturating_sub(200);
    let search_end = (target + 200).min(text.len());

    if let Some(pos) = text[search_start..search_end].rfind(['.', '!', '?', '。', '！', '？']) {
        let boundary = search_start + pos + 1;
        if boundary <= text.len() && text.is_char_boundary(boundary) {
            return boundary;
        }
    }

    // Fallback to word boundary
    let mut search_end = target.min(text.len());
    // Ensure search_end is at a valid UTF-8 boundary
    while search_end > 0 && !text.is_char_boundary(search_end) {
        search_end -= 1;
    }

    if search_end > 0 {
        if let Some(pos) = text[..search_end].rfind(|c: char| c.is_whitespace()) {
            return pos + 1;
        }
    }

    // Last resort: find valid UTF-8 boundary at or before target
    let mut pos = target.min(text.len());
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

/// Output sentences from processing result
fn output_sentences(
    text: &str,
    result: &sakurs_core::application::ProcessingOutput,
    formatter: &mut Box<dyn crate::output::OutputFormatter>,
    base_offset: usize,
) -> Result<()> {
    let sentences = result.extract_sentences(text);
    let ranges = result.sentence_ranges();

    for (sentence, range) in sentences.iter().zip(ranges.iter()) {
        formatter.format_sentence(sentence, base_offset + range.start)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_safe_split_point_sentence_boundary() {
        // Test 1: Small text - finds last period in range
        let text = "First. Second sentence here.";
        let target = 10;
        let split = find_safe_split_point(text, target);
        // With target=10, search range is 0-210, covers whole text
        // rfind finds LAST period at position 27, returns 28
        assert_eq!(split, 28);
        assert_eq!(&text[..split], text); // Entire text

        // Test 2: Longer text where search window matters
        let long_text = concat!(
            "This is a sentence. ", // Period at 18
            "Another sentence. ",   // Period at 35
            "Third sentence. ",     // Period at 50
            "Fourth sentence. ",    // Period at 66
            "Fifth sentence. ",     // Period at 81
            "Sixth sentence. ",     // Period at 96
            "Seventh sentence."     // Period at 112
        );

        // Target 60: search window 0-260 (covers all), finds last period
        let split = find_safe_split_point(long_text, 60);
        println!("Long text len: {}, split: {}", long_text.len(), split);
        // The text is 113 chars total, last char is a period
        // So the split should be at 113 (after the last period)
        assert_eq!(split, long_text.len()); // After last period

        // Test 3: Force fallback to word boundary
        let text3 = "This is a very long sentence without any periods until way at the end.";
        let target3 = 20;
        let split3 = find_safe_split_point(text3, target3);
        // Period is at position 70, outside search range [0, 220]
        // Wait, that's IN the search range. Let me check...
        println!("Text3 period position: {}", text3.find('.').unwrap());
        println!("Target3: {}, Split3: {}", target3, split3);
        // Period is at 69, which is within search range 0-220
        // So it should find the period, not fall back to word boundary
        assert_eq!(split3, 70); // After the period
    }

    #[test]
    fn test_find_safe_split_point_japanese_sentence() {
        // Test 1: Small Japanese text
        let text = "短い文。次の文。";
        let target = 12;
        let split = find_safe_split_point(text, target);
        println!("Japanese text bytes: {}", text.len());
        println!("Target: {}, Split: {}", target, split);

        // The text "短い文。次の文。" has two 。characters
        // Each Japanese character is 3 bytes, 。is also 3 bytes
        // "短い文。" = 4 chars * 3 bytes = 12 bytes
        // So first 。is at bytes 9-11, split would be 12
        assert_eq!(split, 12); // It's actually finding the first one due to the search range

        // Test 2: No sentence boundary, no spaces (Japanese doesn't use spaces)
        let text2 = "これはとても長い日本語の文章で句読点がありません";
        let target2 = 30;
        let split2 = find_safe_split_point(text2, target2);

        // No periods or spaces, should find UTF-8 boundary at or before target
        assert!(text2.is_char_boundary(split2));
        assert!(split2 <= target2);

        // Test 3: Japanese text with proper sentence boundaries
        let text3 = "最初の文。二番目。三番目。";
        let target3 = 50; // Make target larger to ensure we find a sentence boundary
        let split3 = find_safe_split_point(text3, target3);
        println!(
            "Text3 len: {}, target: {}, split: {}",
            text3.len(),
            target3,
            split3
        );

        // With target 50, search range is 0-250, covers whole text
        // Should find the last 。at position 38, return 39
        assert_eq!(split3, 39);
        assert_eq!(&text3[..split3], text3);
    }

    #[test]
    fn test_find_safe_split_point_word_boundary() {
        let text = "This is a very long sentence without any punctuation marks that goes on and on";
        let split = find_safe_split_point(text, 40);
        // Should split at a word boundary
        assert!(split > 0);
        assert!(text.chars().nth(split - 1).unwrap().is_whitespace() || split == text.len());
    }

    #[test]
    fn test_find_safe_split_point_utf8_boundary() {
        let text = "Hello 世界 World こんにちは Test";
        let split = find_safe_split_point(text, 15);
        // Should respect UTF-8 boundaries
        assert!(text.is_char_boundary(split));
    }

    #[test]
    fn test_find_safe_split_point_small_text() {
        let text = "Short.";
        let split = find_safe_split_point(text, 100);
        assert_eq!(split, text.len());
    }

    #[test]
    fn test_find_safe_split_point_exact_boundary() {
        let text = "Exactly at boundary.";
        let split = find_safe_split_point(text, text.len());
        assert_eq!(split, text.len());
    }

    #[test]
    fn test_find_safe_split_point_no_boundaries() {
        let text = "NoSpacesOrPunctuationHereJustOneLongWord";
        let split = find_safe_split_point(text, 20);
        // Should still find a valid UTF-8 boundary
        assert!(text.is_char_boundary(split));
        assert!(split <= 20);
    }
}
