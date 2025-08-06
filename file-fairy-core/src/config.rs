use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration for the File Fairy core functionality.
///
/// Contains settings for the LLM client and other core behaviors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Path to the language model file
    pub model_path: PathBuf,
    /// Maximum number of tokens to generate
    pub max_tokens: u32,
    /// Context size for the model
    pub context_size: u32,
    /// Number of threads to use
    pub threads: u32,
    /// Random seed for reproducible results
    pub seed: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("models/default-model.gguf"),
            max_tokens: 256,
            context_size: 8192,
            threads: 8,
            seed: 103,
        }
    }
}

impl AppConfig {
    /// Creates a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method to set the model path
    pub fn with_model_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.model_path = path.into();
        self
    }

    /// Builder method to set the maximum tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = tokens;
        self
    }

    /// Builder method to set the context size
    pub fn with_context_size(mut self, size: u32) -> Self {
        self.context_size = size;
        self
    }

    /// Builder method to set the number of threads
    pub fn with_threads(mut self, threads: u32) -> Self {
        self.threads = threads;
        self
    }

    /// Builder method to set the seed
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_tokens == 0 {
            return Err("max_tokens must be greater than 0".to_string());
        }
        if self.context_size == 0 {
            return Err("context_size must be greater than 0".to_string());
        }
        if self.threads == 0 {
            return Err("threads must be greater than 0".to_string());
        }
        Ok(())
    }
}
