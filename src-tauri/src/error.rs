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
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;
