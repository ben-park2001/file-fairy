use super::CommandHandler;
use crate::{
    args::ArgBuilder,
    error::{CliError, CliResult},
};
use clap::{ArgMatches, Command};
use file_fairy_core::{DirectoryScanner, ScanOptions};
use std::path::PathBuf;

/// Handler for the 'scan' command
pub struct ScanCommand;

impl ScanCommand {
    /// Creates a new ScanCommand
    pub fn new() -> Self {
        Self
    }

    /// Builds scan options from command line arguments
    fn build_scan_options(&self, matches: &ArgMatches) -> CliResult<ScanOptions> {
        let mut options = ScanOptions::new();

        if matches.get_flag("recursive") {
            options = options.with_recursive(true);
        }

        if let Some(depth) = matches.get_one::<usize>("max-depth") {
            options = options.with_max_depth(*depth);
        }

        if matches.get_flag("follow-symlinks") {
            options = options.with_follow_symlinks(true);
        }

        if let Some(max_size_mb) = matches.get_one::<u64>("max-size") {
            options = options.with_max_file_size(max_size_mb * 1024 * 1024); // Convert MB to bytes
        }

        Ok(options)
    }
}

#[async_trait::async_trait]
impl CommandHandler for ScanCommand {
    fn name(&self) -> &'static str {
        "scan"
    }

    fn about(&self) -> &'static str {
        "Analyze a directory without modifying anything"
    }

    fn build_command(&self) -> Command {
        Command::new(self.name())
            .about(self.about())
            .arg(ArgBuilder::directory_path(
                "path",
                "Directory path to scan",
                ".",
            ))
            .arg(ArgBuilder::recursive())
            .arg(ArgBuilder::max_depth())
            .arg(ArgBuilder::follow_symlinks())
            .arg(ArgBuilder::max_size())
    }

    async fn execute(&self, matches: &ArgMatches) -> CliResult<()> {
        let path = PathBuf::from(matches.get_one::<String>("path").unwrap());

        let options = self.build_scan_options(matches)?;
        let scanner = DirectoryScanner::with_options(options);

        println!("🔍 Scanning directory: {}", path.display());
        println!("Please wait...\n");

        match scanner.scan(&path).await {
            Ok(results) => {
                println!("{}", results.format_summary());
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Scan failed: {}", e);
                Err(CliError::Core(e))
            }
        }
    }
}
