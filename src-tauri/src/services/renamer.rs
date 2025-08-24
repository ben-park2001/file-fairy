use ndarray::Array2;
use std::path::Path;
use tokio::fs;

use crate::error::{AppError, AppResult};

use super::{ExtractionService, OllamaService};

// Number of clusters for K-Means
const N_CLUSTERS: usize = 4;

pub struct RenamerService {
    ollama: OllamaService,
}

impl RenamerService {
    pub fn new() -> Self {
        RenamerService {
            ollama: OllamaService::new(),
        }
    }

    pub async fn generate_new_filename(&self, file_path: &str) -> AppResult<String> {
        // Extract the file content for analysis
        let path = Path::new(file_path);
        let content = ExtractionService::extract_file_content(path).await?;

        // Break down the content into manageable chunks
        let chunks = self.ollama.chunk_text(content.as_str()).await;

        // Create embeddings for the chunks
        let embeddings = self
            .ollama
            .generate_embeddings(&chunks)
            .await
            .map(|emb| {
                // Convert Vec<Vec<f32>> to Array2<f64>
                let flat: Vec<f64> = emb
                    .iter()
                    .flat_map(|v| v.iter().map(|&x| x as f64))
                    .collect();
                Array2::from_shape_vec((emb.len(), emb[0].len()), flat).unwrap()
            })
            .map_err(|e| AppError::Ollama(e))?;

        // Extract the key chunks from the embeddings
        let key_chunks = self
            .ollama
            .extract_key_chunks(&embeddings, &chunks, N_CLUSTERS)
            .await;

        // Generate a new filename based on the key chunks
        let original_filename =
            path.file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| AppError::Path {
                    message: "Invalid file name".into(),
                })?;
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        self.ollama
            .generate_ai_file_name(key_chunks, original_filename, extension)
            .await
            .map_err(|e| AppError::Ollama(e))
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
}
