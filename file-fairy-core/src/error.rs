use std::path::PathBuf;
use thiserror::Error;

/// Core error types for the file-fairy-core library.
///
/// This enum represents all possible errors that can occur in the core library.
/// It follows Rust best practices by using `thiserror` for automatic trait implementations
/// and providing contextual information through error variants.
#[derive(Error, Debug)]
pub enum CoreError {
    /// I/O operations failed (file system access, network, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration loading, parsing, or validation failed
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// Attempted operation on unsupported file type
    #[error("Unsupported file type: '{path}' - only extractable file types are supported")]
    UnsupportedFileType { path: PathBuf },

    /// Required path does not exist
    #[error("Path not found: '{path}' - file or directory does not exist")]
    PathNotFound { path: PathBuf },

    /// File content extraction failed
    #[error("Content extraction failed for '{path}': {source}")]
    Extractor {
        path: PathBuf,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// LLM processing failed
    #[error("LLM processing failed: {0}")]
    Llm(#[from] anyhow::Error),

    /// User cancelled the operation
    #[error("Operation cancelled by user")]
    UserCancelled,

    /// Generic error for cases that don't fit other categories
    #[error("Unexpected error: {message}")]
    Generic { message: String },
}

impl CoreError {
    /// Convenience constructor for generic errors
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic {
            message: msg.into(),
        }
    }

    /// Convenience constructor for unsupported file types
    pub fn unsupported_file_type(path: impl Into<PathBuf>) -> Self {
        Self::UnsupportedFileType { path: path.into() }
    }

    /// Convenience constructor for path not found errors
    pub fn path_not_found(path: impl Into<PathBuf>) -> Self {
        Self::PathNotFound { path: path.into() }
    }

    /// Convenience constructor for extractor errors
    pub fn extractor_error(
        path: impl Into<PathBuf>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::Extractor {
            path: path.into(),
            source,
        }
    }

    /// Returns true if this error indicates a recoverable condition
    /// that might succeed with different input or retry
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::PathNotFound { .. } | Self::UnsupportedFileType { .. } | Self::Config(_)
        )
    }

    /// Returns true if this error is due to user action/cancellation
    pub fn is_user_cancelled(&self) -> bool {
        matches!(self, Self::UserCancelled)
    }

    /// Returns the file path associated with this error, if any
    pub fn file_path(&self) -> Option<&PathBuf> {
        match self {
            Self::UnsupportedFileType { path } => Some(path),
            Self::PathNotFound { path } => Some(path),
            Self::Extractor { path, .. } => Some(path),
            _ => None,
        }
    }
}

// Implement conversion from the old-style constructors for backward compatibility
impl From<PathBuf> for CoreError {
    fn from(path: PathBuf) -> Self {
        Self::PathNotFound { path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_construction() {
        let path = PathBuf::from("/test/path");
        let error = CoreError::path_not_found(path.clone());
        
        assert!(error.is_recoverable());
        assert_eq!(error.file_path(), Some(&path));
    }

    #[test]
    fn test_user_cancellation() {
        let error = CoreError::UserCancelled;
        assert!(error.is_user_cancelled());
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_generic_error() {
        let error = CoreError::generic("Test message");
        assert!(!error.is_recoverable());
        assert_eq!(error.file_path(), None);
    }
}
