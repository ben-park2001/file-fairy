use anyhow::Result;
use async_trait::async_trait;

pub mod client;
pub mod config;
pub mod constants;
pub mod error;

pub use client::LocalLlmClient;
pub use config::LlmConfig;
pub use error::LlmError;

/// Defines the public contract for any LLM client in the application.
/// This allows for swapping out the backend (e.g., from llama.cpp to something else)
/// without changing the core application logic.
#[async_trait]
pub trait FileLlm: Send + Sync {
    /// Suggests a new filename based on the provided file content.
    ///
    /// # Arguments
    ///
    /// * `original_filename` - The original filename to use as context for the suggestion.
    /// * `content` - The text content extracted from a file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the suggested filename as a `String`.
    async fn suggest_filename(&self, original_filename: &str, content: &str) -> Result<String>;
}
