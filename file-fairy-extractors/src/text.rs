use crate::error::ExtractorError;
use crate::traits::Extractor;
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;

/// An extractor for plain text files (e.g., .txt, .md).
/// 
/// Reads the entire file content as UTF-8 text.
#[derive(Debug, Default)]
pub struct TextExtractor;

impl TextExtractor {
    /// Creates a new text extractor
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Extractor for TextExtractor {
    /// Reads the entire file into a string.
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        fs::read_to_string(path)
            .await
            .map_err(|e| ExtractorError::Io {
                path: path.to_path_buf(),
                source: e,
            })
    }
}
