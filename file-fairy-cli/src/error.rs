use file_fairy_core::CoreError;
use std::path::PathBuf;
use thiserror::Error;

/// CLI-specific error types that can occur during command execution
#[derive(Error, Debug)]
pub enum CliError {
    /// Error from the core library
    #[error("Core error: {0}")]
    Core(#[from] CoreError),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid command line argument
    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    /// User cancelled the operation
    #[error("Operation cancelled by user")]
    UserCancelled,

    /// Configuration validation error
    #[error("Configuration error: {0}")]
    Config(String),

    /// File not found error
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Command not found error
    #[error("Command not found: {command}")]
    CommandNotFound { command: String },
}

impl CliError {
    /// Creates a new InvalidArgument error
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument {
            message: message.into(),
        }
    }

    /// Creates a new Config error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Creates a new FileNotFound error
    pub fn file_not_found(path: impl Into<PathBuf>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Creates a new CommandNotFound error
    pub fn command_not_found(command: impl Into<String>) -> Self {
        Self::CommandNotFound {
            command: command.into(),
        }
    }
}

/// Type alias for CLI Results
pub type CliResult<T> = Result<T, CliError>;
