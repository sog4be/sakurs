//! Progress reporting module

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Progress reporter for file processing
pub struct ProgressReporter {
    progress_bar: Option<ProgressBar>,
    quiet: bool,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(quiet: bool) -> Self {
        Self {
            progress_bar: None,
            quiet,
        }
    }

    /// Initialize progress bar for file processing
    pub fn init_files(&mut self, total_files: u64) {
        if self.quiet {
            return;
        }

        let pb = ProgressBar::new(total_files);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        pb.enable_steady_tick(Duration::from_millis(100));

        self.progress_bar = Some(pb);
    }

    /// Update progress for a completed file
    pub fn file_completed(&self, filename: &str) {
        if let Some(pb) = &self.progress_bar {
            pb.set_message(format!("Processed: {}", filename));
            pb.inc(1);
        }
    }

    /// Finish progress reporting
    pub fn finish(&self) {
        if let Some(pb) = &self.progress_bar {
            pb.finish_with_message("Complete");
        }
    }
}
