//! File content extraction service module
//!
//! Provides unified interface for extracting text content from various file types
//! including PDF, Office documents, text files, and other supported formats.

use crate::{error::AppResult, extractors::extractor_from_file_path};
use std::path::Path;

/// Service for file content extraction
pub struct ExtractionService;

impl ExtractionService {
    /// Extract content from a file
    pub async fn extract_file_content(file_path: &Path) -> AppResult<String> {
        if !file_path.exists() {
            return Err(crate::error::AppError::Path {
                message: format!(
                    "File does not exist: {}",
                    file_path.to_str().unwrap_or("unknown path")
                ),
            });
        }

        let extractor =
            extractor_from_file_path(file_path).ok_or_else(|| crate::error::AppError::Path {
                message: "Unsupported file type".to_string(),
            })?;

        let content = extractor.extract(file_path).await?;
        Ok(content)
    }
}
