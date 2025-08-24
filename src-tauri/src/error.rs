use thiserror::Error;

/// Main error type for the file-fairy application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Path error: {message}")]
    Path { message: String },

    #[error("Extractor error: {0}")]
    Extractor(#[from] crate::extractors::ExtractorError),

    #[error("Ollama service error: {0}")]
    Ollama(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] lancedb::Error),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        AppError::Other(message.to_string())
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
