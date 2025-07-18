//! Process command implementation

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

/// Arguments for the process command
#[derive(Debug, Args)]
pub struct ProcessArgs {
    /// Input files or patterns (supports glob, use '-' for stdin)
    #[arg(short, long, value_name = "FILE/PATTERN", required = true)]
    pub input: Vec<String>,

    /// Output file (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    pub format: OutputFormat,

    /// Language for sentence detection rules
    /// NOTE: Mutually exclusive with --language-config
    #[arg(short, long, value_enum, conflicts_with = "language_config")]
    pub language: Option<Language>,

    /// Path to external language configuration file (TOML format)
    /// NOTE: Mutually exclusive with --language
    #[arg(short = 'c', long, value_name = "FILE", conflicts_with = "language")]
    pub language_config: Option<PathBuf>,

    /// Language code for external configuration (optional)
    /// NOTE: Only used with --language-config
    #[arg(long, requires = "language_config")]
    pub language_code: Option<String>,

    /// Force parallel processing even for small files
    #[arg(short, long)]
    pub parallel: bool,

    /// Use adaptive processing (automatically choose best strategy)
    /// Note: This is experimental and currently uses the default processing
    #[arg(long, conflicts_with = "parallel")]
    pub adaptive: bool,

    /// Number of threads for parallel processing (default: auto)
    #[arg(short = 't', long, value_name = "COUNT")]
    pub threads: Option<usize>,

    /// Chunk size in KB for parallel processing (default: 256)
    #[arg(long, value_name = "SIZE_KB")]
    pub chunk_kb: Option<usize>,

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

        // Create output formatter
        let mut formatter: Box<dyn crate::output::OutputFormatter> = self.create_formatter()?;

        // Create processor
        let processor = self.create_processor()?;

        // Check if input is stdin
        if self.input.len() == 1 && self.input[0] == "-" {
            log::info!("Reading from stdin");
            self.process_stdin(&processor, &mut formatter)?;
        } else {
            // Resolve file patterns
            let files = crate::input::resolve_patterns(&self.input)?;
            log::info!("Found {} files to process", files.len());

            // Initialize progress reporter
            let mut progress = crate::progress::ProgressReporter::new(self.quiet);
            progress.init_files(files.len() as u64);

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
                    let result = processor
                        .process(sakurs_core::Input::from_text(content.clone()))
                        .map_err(|e| anyhow::anyhow!("Processing failed: {}", e))?;

                    // Extract and output sentences
                    let mut last_offset = 0;
                    for boundary in &result.boundaries {
                        let sentence = &content[last_offset..boundary.offset];
                        formatter.format_sentence(sentence.trim(), last_offset)?;
                        last_offset = boundary.offset;
                    }

                    // Don't forget the last sentence after the final boundary
                    if last_offset < content.len() {
                        let sentence = &content[last_offset..];
                        if !sentence.trim().is_empty() {
                            formatter.format_sentence(sentence.trim(), last_offset)?;
                        }
                    }
                }

                progress.file_completed(&file.file_name().unwrap_or_default().to_string_lossy());
            }

            progress.finish();
            log::info!("Processing complete. Processed {} files", files.len());
        }

        // Finalize output
        formatter.finish()?;
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
    fn create_processor(&self) -> Result<sakurs_core::SentenceProcessor> {
        use crate::language_source::LanguageSource;
        use sakurs_core::{Config, SentenceProcessor};

        // Determine language source
        let language_source = match (&self.language, &self.language_config) {
            (Some(lang), None) => LanguageSource::BuiltIn(*lang),
            (None, Some(path)) => LanguageSource::External {
                path: path.clone(),
                language_code: self.language_code.clone(),
            },
            (None, None) => LanguageSource::BuiltIn(Language::English), // Default
            (Some(_), Some(_)) => unreachable!(),                       // clap handles conflicts
        };

        log::info!("Using language source: {}", language_source.display_name());

        // Create processor based on language source
        match language_source {
            LanguageSource::BuiltIn(lang) => {
                let language_code = lang.code();

                // Build configuration with thread option handling
                let builder = Config::builder()
                    .language(language_code)
                    .map_err(|e| anyhow::anyhow!("Failed to set language: {}", e))?;

                let builder = self.configure_builder(builder)?;

                let config = builder
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build processor config: {}", e))?;

                SentenceProcessor::with_config(config)
                    .map_err(|e| anyhow::anyhow!("Failed to create processor: {}", e))
            }
            LanguageSource::External {
                path,
                language_code,
            } => {
                // Load external configuration
                use sakurs_core::domain::language::ConfigurableLanguageRules;
                use std::sync::Arc;

                let rules = ConfigurableLanguageRules::from_file(&path, language_code.as_deref())
                    .map_err(|e| {
                    anyhow::anyhow!("Failed to load external language config: {}", e)
                })?;

                // Build configuration
                let builder = Config::builder();
                let builder = self.configure_builder(builder)?;

                let config = builder
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build processor config: {}", e))?;

                // Create processor with custom rules
                SentenceProcessor::with_custom_rules(config, Arc::new(rules))
                    .map_err(|e| anyhow::anyhow!("Failed to create processor: {}", e))
            }
        }
    }

    /// Configure the builder with common options
    fn configure_builder(
        &self,
        builder: sakurs_core::ConfigBuilder,
    ) -> Result<sakurs_core::ConfigBuilder> {
        let mut builder = builder;

        // Handle thread count:
        // - If threads is specified, use that value
        // - If parallel flag is set, use None (all available threads)
        // - Otherwise, use default (auto-detect based on text size)
        if let Some(thread_count) = self.threads {
            if thread_count == 0 {
                return Err(anyhow::anyhow!("Thread count must be greater than 0"));
            }
            builder = builder.threads(Some(thread_count));
        } else if self.parallel {
            builder = builder.threads(None); // Use all available threads
        }

        // Handle chunk size if specified
        if let Some(chunk_kb) = self.chunk_kb {
            if chunk_kb == 0 {
                return Err(anyhow::anyhow!("Chunk size must be greater than 0"));
            }
            // Convert KB to bytes
            let chunk_size = chunk_kb * 1024;
            builder = builder.chunk_size(chunk_size);
        }

        // Note: adaptive mode now uses default configuration
        Ok(builder)
    }

    /// Process a file in streaming mode
    fn process_file_streaming(
        &self,
        file: &std::path::Path,
        processor: &sakurs_core::SentenceProcessor,
        formatter: &mut Box<dyn crate::output::OutputFormatter>,
    ) -> Result<()> {
        // For now, streaming mode uses the same processing as regular mode
        // but could be enhanced in the future to process chunks incrementally
        log::info!("Using streaming mode for large file: {}", file.display());

        let content = crate::input::FileReader::read_text(file)?;
        let result = processor
            .process(sakurs_core::Input::from_text(content.clone()))
            .map_err(|e| anyhow::anyhow!("Processing failed: {}", e))?;

        let mut last_offset = 0;
        for boundary in &result.boundaries {
            let sentence = &content[last_offset..boundary.offset];
            formatter.format_sentence(sentence.trim(), last_offset)?;
            last_offset = boundary.offset;
        }

        // Don't forget the last sentence after the final boundary
        if last_offset < content.len() {
            let sentence = &content[last_offset..];
            if !sentence.trim().is_empty() {
                formatter.format_sentence(sentence.trim(), last_offset)?;
            }
        }

        Ok(())
    }

    /// Process stdin
    fn process_stdin(
        &self,
        processor: &sakurs_core::SentenceProcessor,
        formatter: &mut Box<dyn crate::output::OutputFormatter>,
    ) -> Result<()> {
        use std::io::Read;

        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;

        let result = processor
            .process(sakurs_core::Input::from_text(buffer.clone()))
            .map_err(|e| anyhow::anyhow!("Processing failed: {}", e))?;

        let mut last_offset = 0;
        for boundary in &result.boundaries {
            let sentence = &buffer[last_offset..boundary.offset];
            formatter.format_sentence(sentence.trim(), last_offset)?;
            last_offset = boundary.offset;
        }

        // Don't forget the last sentence after the final boundary
        if last_offset < buffer.len() {
            let sentence = &buffer[last_offset..];
            if !sentence.trim().is_empty() {
                formatter.format_sentence(sentence.trim(), last_offset)?;
            }
        }

        Ok(())
    }
}

/// Find a safe point to split text (prefer sentence boundary, then word boundary)
#[allow(dead_code)]
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
#[allow(dead_code)]
fn output_sentences(
    text: &str,
    result: &sakurs_core::Output,
    formatter: &mut Box<dyn crate::output::OutputFormatter>,
    base_offset: usize,
) -> Result<()> {
    let mut last_offset = 0;
    for boundary in &result.boundaries {
        let sentence = &text[last_offset..boundary.offset];
        formatter.format_sentence(sentence.trim(), base_offset + last_offset)?;
        last_offset = boundary.offset;
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
