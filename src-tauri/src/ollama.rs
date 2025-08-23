use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_BASE_URL: &str = "http://localhost:11434";
const TRUNCATED_CONTENT_LENGTH: usize = 3500;

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub response: String,
    pub done: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embedding: Vec<f64>,
}

/// Service for interacting with Ollama AI models
#[derive(Clone)]
pub struct OllamaService {
    client: reqwest::Client,
    base_url: String,
}

impl Default for OllamaService {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaService {
    /// Create a new Ollama service instance
    pub fn new() -> Self {
        Self::with_config(DEFAULT_BASE_URL, DEFAULT_TIMEOUT)
    }

    /// Create a new Ollama service with custom configuration
    pub fn with_config(base_url: &str, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    /// Generate a filename from summary using Ollama
    pub async fn generate_filename(&self, summary: &str, model: &str) -> Result<String, String> {
        let prompt = Self::create_filename_prompt(summary);
        self.generate_text(&prompt, model).await
    }

    /// Check if Ollama service is healthy
    pub async fn health_check(&self) -> Result<bool, String> {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err("Failed to fetch models from Ollama".to_string());
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let models = json["models"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|model| model["name"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(models)
    }

    /// Generic text generation method
    async fn generate_text(&self, prompt: &str, model: &str) -> Result<String, String> {
        let request = OllamaRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Ollama request failed with status: {}",
                response.status()
            ));
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(ollama_response.response.trim().to_string())
    }

    /// Create a prompt for filename generation
    fn create_filename_prompt(summary: &str) -> String {
        format!(
            "Based on the summary below, generate a clear and descriptive filename.\nRules:\n* Maximum 3 to 4 words.\n* Use only nouns and adjectives (no verbs).\n* Use lowercase letters only.\n* Connect words with underscores.\n* Do not include words like 'text', 'document', 'file', 'pdf', etc.\n* Focus on the main subject, key method/topic, and context or outcome.\n* Output only the filename, with no extra text.\n\nSummary: \"\"\"{}\"\"\"\n\nFilename:",
            summary
        )
    }
    
    /// Generate embeddings for text using Ollama
    /// <Example Usage>
    /// let embeddings = ollama_service
    ///     .generate_embedding("테스트 문장입니다", "dengcao/Qwen3-Embedding-0.6B:Q8_0")
    ///     .await?;
    /// println!("Embedding vector length: {}", embeddings.len());

    pub async fn generate_embedding(&self, text: &str, model: &str) -> Result<Vec<f64>, String> {
        let request = EmbeddingRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };

        let url = format!("{}/api/embeddings", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send embedding request to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Ollama embedding request failed with status: {}",
                response.status()
            ));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama embedding response: {}", e))?;

        Ok(embedding_response.embedding)
    }

}
