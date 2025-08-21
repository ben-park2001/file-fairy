use super::error::ExtractorError;
use async_trait::async_trait;
use std::path::Path;

/// A common interface for all file content extractors.
///
/// By defining a shared trait, we can treat various file extractors
/// (PDF, DOCX, etc.) polymorphically. The `async_trait` macro allows
/// the use of `async fn` in the trait, which is essential for non-blocking I/O.
#[async_trait]
pub trait Extractor: Send + Sync {
    /// Extracts text content from a given file path.
    ///
    /// # Arguments
    /// * `path` - A reference to the file's path.
    ///
    /// # Returns
    /// A `Result` containing the extracted text as a `String` or an `ExtractorError`.
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError>;
}
