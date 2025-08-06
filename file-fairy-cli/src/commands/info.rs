use super::CommandHandler;
use crate::error::CliResult;
use clap::{ArgMatches, Command};
use file_fairy_core::FileFairyInfo;

/// Handler for the 'info' command
pub struct InfoCommand;

impl InfoCommand {
    /// Creates a new InfoCommand
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl CommandHandler for InfoCommand {
    fn name(&self) -> &'static str {
        "info"
    }
    
    fn about(&self) -> &'static str {
        "Display information about File Fairy's configuration and capabilities"
    }
    
    fn build_command(&self) -> Command {
        Command::new(self.name())
            .about(self.about())
            .subcommand(
                Command::new("summary").about("Show comprehensive File Fairy information (default)"),
            )
            .subcommand(Command::new("formats").about("Show supported file formats"))
            .subcommand(Command::new("system").about("Show system information"))
    }

    async fn execute(&self, matches: &ArgMatches) -> CliResult<()> {
        let info = FileFairyInfo::new();

        match matches.subcommand() {
            Some(("formats", _)) => {
                println!("{}", info.format_formats());
            }
            Some(("system", _)) => {
                println!("{}", info.format_system());
            }
            Some(("summary", _)) | None => {
                // Default to summary if no subcommand specified
                println!("{}", info.format_summary());
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}
