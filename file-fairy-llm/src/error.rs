use thiserror::Error;

/// Defines the specific errors that can occur within the LLM crate.
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Failed to load model from '{0}': {1}")]
    ModelLoad(String, String),

    #[error("Failed to create context: {0}")]
    ContextCreation(String),

    #[error("Failed to tokenize input: {0}")]
    Tokenization(String),

    #[error("Inference failed: {0}")]
    Inference(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Model not found at path: {0}")]
    ModelNotFound(String),

    #[error("An I/O error occurred")]
    Io(#[from] std::io::Error),

    #[error("An internal error occurred")]
    Internal(#[from] anyhow::Error),
}

impl LlmError {
    /// Creates a model load error with path and source information
    pub fn model_load<P: AsRef<str>>(path: P, source: impl ToString) -> Self {
        Self::ModelLoad(path.as_ref().to_string(), source.to_string())
    }
}
