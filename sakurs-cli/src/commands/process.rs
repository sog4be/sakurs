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
        log::debug!("Arguments: {:?}", self);

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

            // Read file content
            let content = crate::input::FileReader::read_text(file)?;

            // Process text
            let result = processor.process_text(&content)?;

            // Extract and output sentences
            let sentences = result.extract_sentences(&content);
            let ranges = result.sentence_ranges();

            for (sentence, range) in sentences.iter().zip(ranges.iter()) {
                formatter.format_sentence(sentence, range.start)?;
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
}
