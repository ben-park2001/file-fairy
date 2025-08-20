use clap::Command;

mod args;
mod commands;
mod config;
mod error;

use args::ArgBuilder;
use commands::CommandRegistry;
use error::CliResult;

#[tokio::main]
async fn main() -> CliResult<()> {
    let registry = CommandRegistry::new();
    let app = create_app(&registry);
    let matches = app.get_matches();

    // Global verbose flag
    let _verbose = matches.get_flag("verbose");

    // Execute the appropriate command
    if let Some((command_name, sub_matches)) = matches.subcommand() {
        registry.execute_command(command_name, sub_matches).await
    } else {
        unreachable!() // Due to subcommand_required(true)
    }
}

fn create_app(registry: &CommandRegistry) -> Command {
    let mut app = Command::new("file-fairy")
        .version("0.1.0")
        .author("File Fairy Team")
        .about("AI-powered file organization tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(ArgBuilder::verbose());

    // Add all commands from registry
    for command in registry.get_commands() {
        app = app.subcommand(command);
    }

    // Add the suggest command (not yet refactored)
    app = app.subcommand(create_suggest_command());

    app
}

fn create_suggest_command() -> Command {
    Command::new("suggest")
        .about("Generate filename suggestions for a file")
        .arg(ArgBuilder::file_path(
            "file",
            "The file to analyze and generate a filename for",
        ))
        .arg(ArgBuilder::model_path(
            "models/Qwen3-4B-Instruct-2507-Q4_K_M.gguf",
        ))
        .arg(ArgBuilder::max_tokens("32"))
        .arg(ArgBuilder::threads("8"))
}
