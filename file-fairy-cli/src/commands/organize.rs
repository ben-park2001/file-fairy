use super::CommandHandler;
use crate::{args::ArgBuilder, config::CliConfig, error::{CliError, CliResult}};
use clap::{ArgMatches, Command};
use file_fairy_core::{FileOrganizeService, OrganizeOptions};
use std::{io::{self, Write}, path::PathBuf};

/// Handler for the 'organize' command
pub struct OrganizeCommand;

impl OrganizeCommand {
    /// Creates a new OrganizeCommand
    pub fn new() -> Self {
        Self
    }

    /// Shows safety warning for apply mode and gets user confirmation
    fn show_safety_warning(&self) -> CliResult<()> {
        println!("⚠️  WARNING: --apply flag detected. This will actually move and rename files!");
        println!("   Make sure you have backups and are in the correct directory.");
        print!("   Continue? [y/N]: ");
        io::stdout().flush().map_err(CliError::Io)?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(CliError::Io)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            println!("❌ Aborted by user");
            return Err(CliError::UserCancelled);
        }

        Ok(())
    }

    /// Builds organize options from command line arguments
    fn build_organize_options(&self, matches: &ArgMatches) -> CliResult<OrganizeOptions> {
        let mut options = OrganizeOptions::new();

        let apply = matches.get_flag("apply");
        options = options.with_apply(apply);

        if matches.get_flag("interactive") {
            options = options.with_interactive(true);
        }

        if matches.get_flag("recursive") {
            options = options.with_recursive(true);
        }

        if let Some(depth) = matches.get_one::<usize>("max-depth") {
            options = options.with_max_depth(*depth);
        }

        if let Some(target_dir) = matches.get_one::<String>("target-dir") {
            options = options.with_target_dir(PathBuf::from(target_dir));
        }

        if matches.get_flag("follow-symlinks") {
            options = options.with_follow_symlinks(true);
        }

        if let Some(max_size_mb) = matches.get_one::<u64>("max-size") {
            options = options.with_max_file_size(max_size_mb * 1024 * 1024); // Convert MB to bytes
        }

        Ok(options)
    }

    /// Builds application configuration from command line arguments
    fn build_config(&self, matches: &ArgMatches) -> CliResult<file_fairy_core::config::AppConfig> {
        let model_path = PathBuf::from(matches.get_one::<String>("model").unwrap());
        let max_tokens = *matches.get_one::<u32>("max-tokens").unwrap();
        let threads = *matches.get_one::<u32>("threads").unwrap();

        CliConfig::create_app_config(model_path, max_tokens, threads)
    }

    /// Displays information about the operation being performed
    fn display_operation_info(&self, path: &PathBuf, apply: bool, options: &OrganizeOptions) {
        if apply {
            println!("🚀 ORGANIZING FILES (LIVE MODE)");
        } else {
            println!("🔍 ORGANIZING FILES (PREVIEW MODE)");
            println!("   No files will be moved. Use --apply to execute changes.");
        }

        println!("📁 Path: {}", path.display());

        if options.interactive {
            println!("🤝 Interactive mode enabled - you'll be prompted for each file");
        }

        println!("Please wait...\n");
    }
}

#[async_trait::async_trait]
impl CommandHandler for OrganizeCommand {
    fn name(&self) -> &'static str {
        "organize"
    }
    
    fn about(&self) -> &'static str {
        "Categorize, rename, and move files based on content and metadata"
    }
    
    fn build_command(&self) -> Command {
        Command::new(self.name())
            .about(self.about())
            .arg(ArgBuilder::directory_path(
                "path",
                "Directory path to organize",
                ".",
            ))
            .arg(ArgBuilder::apply())
            .arg(ArgBuilder::interactive())
            .arg(ArgBuilder::recursive())
            .arg(ArgBuilder::max_depth())
            .arg(ArgBuilder::target_dir())
            .arg(ArgBuilder::follow_symlinks())
            .arg(ArgBuilder::max_size())
            .arg(ArgBuilder::model_path("models/gemma-3n-E2B-it-Q4_K_M.gguf"))
            .arg(ArgBuilder::max_tokens("32"))
            .arg(ArgBuilder::threads("8"))
    }

    async fn execute(&self, matches: &ArgMatches) -> CliResult<()> {
        let path = PathBuf::from(matches.get_one::<String>("path").unwrap());
        
        // Safety warning for apply mode
        let apply = matches.get_flag("apply");
        if apply {
            self.show_safety_warning()?;
        }

        let config = self.build_config(matches)?;
        let options = self.build_organize_options(matches)?;

        let service = FileOrganizeService::new(config)
            .map_err(CliError::Core)?;

        self.display_operation_info(&path, apply, &options);

        match service.organize(&path, options).await {
            Ok(results) => {
                println!("{}", results.format_preview());

                if !apply && results.total_actions() > 0 {
                    println!("\n💡 To execute these changes, run the same command with --apply");
                    println!("   For more control, add --interactive to review each file");
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Organization failed: {}", e);
                Err(CliError::Core(e))
            }
        }
    }
}
