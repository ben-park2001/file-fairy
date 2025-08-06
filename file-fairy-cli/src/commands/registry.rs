use crate::error::CliResult;
use clap::{ArgMatches, Command};

/// Trait for CLI command handlers with command definition
#[async_trait::async_trait]
pub trait CommandHandler {
    /// The name of the command (e.g., "scan", "organize")
    fn name(&self) -> &'static str;

    /// The description of the command
    fn about(&self) -> &'static str;

    /// Build the clap Command definition
    fn build_command(&self) -> Command;

    /// Execute the command with the given arguments
    async fn execute(&self, matches: &ArgMatches) -> CliResult<()>;
}

/// Registry of all available commands
pub struct CommandRegistry {
    handlers: Vec<Box<dyn CommandHandler>>,
}

impl CommandRegistry {
    /// Create a new command registry with all available commands
    pub fn new() -> Self {
        let handlers: Vec<Box<dyn CommandHandler>> = vec![
            Box::new(crate::commands::scan::ScanCommand::new()),
            Box::new(crate::commands::organize::OrganizeCommand::new()),
            Box::new(crate::commands::info::InfoCommand::new()),
        ];

        Self { handlers }
    }

    /// Get all command definitions for clap
    pub fn get_commands(&self) -> Vec<Command> {
        self.handlers.iter().map(|h| h.build_command()).collect()
    }

    /// Execute a command by name
    pub async fn execute_command(&self, name: &str, matches: &ArgMatches) -> CliResult<()> {
        let handler = self
            .handlers
            .iter()
            .find(|h| h.name() == name)
            .ok_or_else(|| crate::error::CliError::command_not_found(name))?;

        // Call the execute method on the found handler
        handler.execute(matches).await
    }
}
