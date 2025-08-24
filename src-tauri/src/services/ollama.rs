use linfa::prelude::*;
use linfa_clustering::KMeans;
use ndarray::{Array2, Axis};
use regex::Regex;
use std::path::Path;
use text_splitter::{Characters, ChunkConfig, TextSplitter};

use ollama_rs::{
    generation::{
        completion::request::GenerationRequest, embeddings::request::GenerateEmbeddingsRequest,
    },
    models::ModelOptions,
    Ollama,
};

const EMBEDDING_MODEL: &str = "dengcao/Qwen3-Embedding-0.6B:Q8_0";
const LANGUAGE_MODEL: &str = "qwen3:4b-instruct-2507-q4_K_M";
const CHUNK_SIZE: usize = 250;
const CHUNK_OVERLAP: usize = 50;

pub struct OllamaService {
    ollama: Ollama,
    text_splitter: TextSplitter<Characters>,
}

impl OllamaService {
    pub fn new() -> Self {
        let ollama = Ollama::default();

        let text_splitter_config = ChunkConfig::new(CHUNK_SIZE)
            .with_trim(true)
            .with_overlap(CHUNK_OVERLAP)
            .unwrap();

        let text_splitter = TextSplitter::new(text_splitter_config);

        OllamaService {
            ollama,
            text_splitter,
        }
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        let request = GenerateEmbeddingsRequest::new(EMBEDDING_MODEL.to_string(), text.into());
        let response = self
            .ollama
            .generate_embeddings(request)
            .await
            .map_err(|e| format!("Ollama embedding error: {}", e))?;

        if let Some(embedding) = response.embeddings.first() {
            Ok(embedding.clone())
        } else {
            Err("No embedding found".into())
        }
    }

    pub async fn generate_embeddings(&self, texts: &Vec<String>) -> Result<Vec<Vec<f32>>, String> {
        let request =
            GenerateEmbeddingsRequest::new(EMBEDDING_MODEL.to_string(), texts.clone().into());
        let response = self
            .ollama
            .generate_embeddings(request)
            .await
            .map_err(|e| format!("Ollama embedding error: {}", e))?;

        Ok(response.embeddings)
    }

    pub async fn generate_ai_file_name(
        &self,
        key_chunks: Vec<String>,
        original_filename: &str,
        file_extension: &str,
    ) -> Result<String, String> {
        let content_summary = key_chunks.join("\n");

        let prompt = format!(
            r#"You are a helpful assistant that generates concise, descriptive filenames based on document content. 
Generate a filename that:
- Is descriptive and meaningful
- Uses underscores instead of spaces
- Is professional and organized
- Does not include special characters except underscores and hyphens
- Is between 1 to 4 words (excluding extension)
- Captures the main topic/purpose of the document
- Respond with ONLY the filename base (without extension), nothing else.

Based on this document content, generate a descriptive filename:

Content summary: {}

Original filename: {}

Generate filename (without extension):"#,
            content_summary, original_filename,
        );

        let options = ModelOptions::default()
            .temperature(0.2)
            .repeat_penalty(1.1)
            .top_k(40)
            .top_p(0.95);

        let request = GenerationRequest::new(LANGUAGE_MODEL.to_string(), prompt).options(options);

        let response = self
            .ollama
            .generate(request)
            .await
            .map_err(|e| format!("Ollama generation error: {}", e))?;

        let suggested_name = self.sanitize_filename(response.response).await;

        Ok(format!("{}.{}", suggested_name, file_extension))
    }

    pub async fn chunk_text(&self, text: &str) -> Vec<String> {
        let chunks = self.text_splitter.chunks(text);
        chunks.into_iter().map(|c| c.to_string()).collect()
    }

    /// Extracts the most representative text chunks from a set of embeddings using K-Means clustering.
    ///
    /// This function groups the embedding vectors into a specified number of clusters and then
    /// finds the single chunk that is closest to the center of each cluster. This provides a
    /// diverse and representative sample of the document's content.
    ///
    /// # Arguments
    /// * `embeddings`: An `Array2<f64>` of shape (n_chunks, n_dimensions).
    /// * `original_chunks`: A slice of the original text chunks corresponding to the embeddings.
    /// * `k`: The desired number of key chunks (clusters) to extract.
    ///
    /// # Returns
    /// A `Vec<String>` containing the text of the most representative chunks.
    pub async fn extract_key_chunks(
        &self,
        embeddings: &Array2<f64>,
        original_chunks: &Vec<String>,
        k: usize,
    ) -> Vec<String> {
        let n_chunks = embeddings.len_of(Axis(0));
        if n_chunks <= k {
            return original_chunks.iter().map(|s| s.to_string()).collect();
        }

        // 1. Configure and run the K-Means algorithm using the builder pattern.
        let observations = DatasetBase::from(embeddings.clone());
        let model = KMeans::params(k)
            .max_n_iterations(100)
            .tolerance(1e-5)
            .fit(&observations)
            .expect("KMeans failed to fit the data");

        let centroids = model.centroids();
        let mut key_chunk_indices = Vec::new();

        // 2. For each centroid, find the original data point (embedding) that is closest to it.
        for i in 0..centroids.len_of(Axis(0)) {
            let centroid = centroids.index_axis(Axis(0), i);

            let mut min_dist = f64::MAX;
            let mut closest_sample_idx = 0;

            // Iterate through all original embeddings to find the one nearest to the current centroid
            for j in 0..n_chunks {
                let sample = embeddings.index_axis(Axis(0), j);
                // Calculate squared Euclidean distance (faster than full Euclidean)
                let dist = sample
                    .iter()
                    .zip(centroid.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>();

                if dist < min_dist {
                    min_dist = dist;
                    closest_sample_idx = j;
                }
            }
            key_chunk_indices.push(closest_sample_idx);
        }

        // 3. Sort indices to maintain the original document order and collect the text.
        key_chunk_indices.sort();
        key_chunk_indices.dedup();

        key_chunk_indices
            .into_iter()
            .map(|idx| original_chunks[idx].to_string())
            .collect()
    }

    async fn sanitize_filename(&self, filename: String) -> String {
        // 1. Remove or replace invalid characters
        let invalid_chars_re = Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
        let sanitized = invalid_chars_re.replace_all(&filename, "_");

        // 2. Replace multiple consecutive underscores with a single one
        let multi_underscore_re = Regex::new(r"_+").unwrap();
        let mut sanitized = multi_underscore_re.replace_all(&sanitized, "_").to_string();

        // 3. Remove leading/trailing dots and spaces
        sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ').to_string();

        // 4. Limit length to a reasonable maximum (e.g., 200 chars), preserving extension
        if sanitized.chars().count() > 200 {
            let path = Path::new(&sanitized);
            let stem = path.file_stem().unwrap_or_default().to_str().unwrap_or("");
            let ext = path.extension().unwrap_or_default().to_str().unwrap_or("");

            let ext_len = if ext.is_empty() { 0 } else { ext.len() + 1 };
            let max_stem_len = 200usize.saturating_sub(ext_len);

            let truncated_stem: String = stem.chars().take(max_stem_len).collect();

            sanitized = if ext.is_empty() {
                truncated_stem
            } else {
                format!("{}.{}", truncated_stem, ext)
            };
        }

        // 5. Ensure it's not empty
        if sanitized.is_empty() {
            sanitized = "unnamed_file".to_string();
        }

        sanitized
    }
}
