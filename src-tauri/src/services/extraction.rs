//! File content extraction service module
//!
//! Provides unified interface for extracting text content from various file types
//! including PDF, Office documents, text files, and other supported formats.

use crate::{error::AppResult, extractors::extractor_from_file_path};
use std::path::Path;

/// Service for file content extraction
#[derive(Clone)]
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

        Self::clean_content(&content)
    }

    /// Cleans extracted text content by removing unwanted characters and normalizing whitespace.
    ///
    /// This default implementation provides a baseline for cleaning text.
    /// It removes control characters (except for newlines and tabs),
    /// collapses multiple whitespace characters, and trims excess newlines.
    ///
    /// # Arguments
    /// * `content` - The raw extracted text as a string slice.
    ///
    /// # Returns
    /// A `Result` containing the cleaned text as a `String` or an `ExtractorError`.
    fn clean_content(content: &str) -> AppResult<String> {
        if content.is_empty() {
            return Ok(String::new());
        }

        // Remove control characters, but keep newlines and tabs.
        let content: String = content
            .chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
            .collect();

        // Replace multiple spaces/tabs with a single space.
        let re_spaces = regex::Regex::new(r"[ \t]+").unwrap();
        let content = re_spaces.replace_all(&content, " ");

        // Replace 3 or more newlines (with optional whitespace between) with just two.
        let re_newlines = regex::Regex::new(r"\n\s*\n\s*\n+").unwrap();
        let content = re_newlines.replace_all(&content, "\n\n");

        // Trim leading/trailing whitespace from each line, then join them back.
        let lines: Vec<&str> = content.split('\n').map(|line| line.trim()).collect();
        let content = lines.join("\n");

        // Trim whitespace from the start and end of the entire text.
        Ok(content.trim().to_string())
    }
}
