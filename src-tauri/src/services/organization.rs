//! File organization service module
//!
//! Provides high-level operations for organizing files using AI analysis,
//! including content extraction, summarization, and intelligent file placement.

use crate::{ollama::OllamaService, services::ExtractionService, types::OrganizationResult};
use std::path::Path;
use tokio::fs;

// Configuration constants
const CHUNK_SIZE: usize = 300; // Characters per chunk
const CHUNK_OVERLAP: usize = 30; // Overlap between chunks to maintain context
const EMBEDDING_MODEL: &str = "dengcao/Qwen3-Embedding-0.6B:Q8_0";
const MAX_REPRESENTATIVE_CHUNKS: usize = 5; // Maximum number of representative chunks

/// Service for file organization operations
#[derive(Clone)]
pub struct OrganizationService {
    ollama: OllamaService,
}

// Helper struct to hold representative data
#[derive(Debug, Clone)]
pub struct RepresentativeData {
    pub chunk: String,
    pub embedding: Vec<f32>,
    pub cluster_id: usize,
}

impl RepresentativeData {
    pub fn new(chunk: String, embedding: Vec<f32>, cluster_id: usize) -> Self {
        Self {
            chunk,
            embedding,
            cluster_id,
        }
    }
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
        filename_model: &str,
    ) -> Result<OrganizationResult, String> {
        // Validate input parameters
        if file_path.is_empty() {
            return Err("File path cannot be empty".to_string());
        }
        if filename_model.is_empty() {
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

        // Step 2: Chunk the extracted content into fixed-length pieces
        let chunks = self.chunk_content(&content)
            .await
            .map_err(|e| format!("Content chunking failed: {}", e))?;

        // Step 3: Convert each chunk to embeddings
        let embeddings = self.generate_embeddings(&chunks)
            .await
            .map_err(|e| format!("Embedding generation failed: {}", e))?;

        // Step 4: Cluster embeddings and get representative chunks
        let representative_data = self.cluster_and_select_representatives(&chunks, &embeddings)
            .await
            .map_err(|e| format!("Clustering failed: {}", e))?;

        // Step 5: Combine representative chunks into summary
        let summary = self.combine_representative_chunks(&representative_data)
            .await
            .map_err(|e| format!("Summary generation failed: {}", e))?;

        // Step 6: Generate filename using the summary
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

    /// Chunk the content into fixed-length pieces
    async fn chunk_content(&self, content: &str) -> Result<Vec<String>, String> {
        if content.is_empty() {
            return Ok(vec![]);
        }

        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < content.len() {
            let end = (start + CHUNK_SIZE).min(content.len());
            let chunk = content[start..end].to_string();
            chunks.push(chunk);
            
            // Move start position with overlap
            start += CHUNK_SIZE - CHUNK_OVERLAP;
            
            // Prevent infinite loop if overlap >= chunk_size
            if start <= (chunks.len() - 1) * (CHUNK_SIZE - CHUNK_OVERLAP) {
                break;
            }
        }
        
        Ok(chunks)
    }

    /// Generate embeddings for each chunk
    async fn generate_embeddings(&self, chunks: &[String]) -> Result<Vec<Vec<f32>>, String> {
        let mut embeddings = Vec::with_capacity(chunks.len());
        
        for chunk in chunks {
            let embedding = self.ollama
                .generate_embedding(chunk, EMBEDDING_MODEL)
                .await
                .map_err(|e| format!("Failed to generate embedding for chunk: {}", e))?;
            
            embeddings.push(embedding);
        }
        
        Ok(embeddings)
    }

    /// Cluster embeddings and select representative chunks
    async fn cluster_and_select_representatives(
        &self, 
        chunks: &[String], 
        embeddings: &[Vec<f32>]
    ) -> Result<Vec<RepresentativeData>, String> {
        if chunks.len() != embeddings.len() {
            return Err("Chunks and embeddings length mismatch".to_string());
        }
        
        let take_count = MAX_REPRESENTATIVE_CHUNKS.min(chunks.len());
        
        let representative_data = chunks
            .iter()
            .zip(embeddings.iter())
            .take(take_count)
            .enumerate()
            .map(|(index, (chunk, embedding))| {
                RepresentativeData::new(chunk.clone(), embedding.clone(), index)
            })
            .collect();
        
        Ok(representative_data)
    }

    /// Combine representative chunks into a single summary string
    async fn combine_representative_chunks(
        &self, 
        representative_data: &[RepresentativeData]
    ) -> Result<String, String> {
        if representative_data.is_empty() {
            return Ok(String::new());
        }
        
        let combined = representative_data
            .iter()
            .map(|data| data.chunk.as_str())
            .collect::<Vec<&str>>()
            .join("\n\n");
        
        Ok(combined)
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

    /// Generate filename for given summary
    pub async fn generate_filename(&self, summary: &str, model: &str) -> Result<String, String> {
        self.ollama.generate_filename(summary, model).await
    }
}

impl Default for OrganizationService {
    fn default() -> Self {
        Self::new()
    }
}
