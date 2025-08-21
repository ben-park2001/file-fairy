//! File organization service module
//!
//! Provides high-level operations for organizing files using AI analysis,
//! including content extraction, summarization, and intelligent file placement.

use crate::{ollama::OllamaService, services::ExtractionService, types::OrganizationResult};
use std::path::Path;
use tokio::fs;

/// Service for file organization operations
pub struct OrganizationService {
    ollama: OllamaService,
}

impl OrganizationService {
    /// Create a new organization service
    pub fn new() -> Self {
        Self {
            ollama: OllamaService::new(),
        }
    }

    /// Analyze and organize a file, returning summary, filename, and organization path
    pub async fn analyze_and_organize_file(
        &self,
        file_path: &str,
        summary_model: &str,
        filename_model: &str,
    ) -> Result<OrganizationResult, String> {
        // Validate input parameters
        if file_path.is_empty() {
            return Err("File path cannot be empty".to_string());
        }
        if summary_model.is_empty() || filename_model.is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        // Step 1: Extract file content
        let path = Path::new(file_path);
        let content = ExtractionService::extract_file_content(path)
            .await
            .map_err(|e| format!("Content extraction failed: {}", e))?;

        if content.trim().is_empty() {
            return Err("File content is empty or could not be extracted".to_string());
        }

        // Step 2: Generate summary
        let summary = self
            .ollama
            .generate_summary(&content, summary_model)
            .await
            .map_err(|e| format!("Summary generation failed: {}", e))?;

        // Step 3: Generate filename
        let new_name = self
            .ollama
            .generate_filename(&summary, filename_model)
            .await
            .map_err(|e| format!("Filename generation failed: {}", e))?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("txt");
        let new_filename = format!("{}.{}", new_name, extension);

        Ok(OrganizationResult::new(summary, new_filename))
    }

    /// Rename/move a file to a new location with enhanced error handling
    pub async fn rename_file(&self, old_path: &str, new_path: &str) -> Result<(), String> {
        // Validate input paths
        if old_path.is_empty() || new_path.is_empty() {
            return Err("File paths cannot be empty".to_string());
        }

        let old_path_obj = std::path::Path::new(old_path);
        let new_path_obj = std::path::Path::new(new_path);

        // Check if source file exists
        if !old_path_obj.exists() {
            return Err(format!("Source file does not exist: {}", old_path));
        }

        // Check if destination already exists
        if new_path_obj.exists() {
            return Err(format!("Destination file already exists: {}", new_path));
        }

        // Ensure the parent directory exists
        if let Some(parent) = new_path_obj.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await.map_err(|e| {
                    format!("Failed to create directory '{}': {}", parent.display(), e)
                })?;
            }
        }

        // Perform the rename operation
        fs::rename(old_path, new_path)
            .await
            .map_err(|e| format!("Failed to rename '{}' to '{}': {}", old_path, new_path, e))
    }

    /// Health check for the organization service
    pub async fn health_check(&self) -> Result<bool, String> {
        self.ollama.health_check().await
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        self.ollama.list_models().await
    }

    /// Generate summary for given content
    pub async fn generate_summary(&self, content: &str, model: &str) -> Result<String, String> {
        self.ollama.generate_summary(content, model).await
    }

    /// Generate filename for given summary
    pub async fn generate_filename(&self, summary: &str, model: &str) -> Result<String, String> {
        self.ollama.generate_filename(summary, model).await
    }

    /// Generate organization path for given parameters
    pub async fn generate_organization_path(
        &self,
        summary: &str,
        file_path: &str,
        folder_structure: &[String],
        model: &str,
    ) -> Result<String, String> {
        self.ollama
            .generate_organization_path(summary, file_path, folder_structure, model)
            .await
    }
}

impl Default for OrganizationService {
    fn default() -> Self {
        Self::new()
    }
}
