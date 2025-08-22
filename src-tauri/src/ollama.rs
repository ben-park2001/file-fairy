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

    /// Generate a summary from file content using Ollama
    pub async fn generate_summary(&self, content: &str, model: &str) -> Result<String, String> {
        let truncated_content = content
            .chars()
            .take(TRUNCATED_CONTENT_LENGTH)
            .collect::<String>();
        let prompt = Self::create_summary_prompt(&truncated_content);
        self.generate_text(&prompt, model).await
    }

    /// Generate a filename from summary using Ollama
    pub async fn generate_filename(&self, summary: &str, model: &str) -> Result<String, String> {
        let prompt = Self::create_filename_prompt(summary);
        self.generate_text(&prompt, model).await
    }

    /// Generate an organization path from summary and context
    pub async fn generate_organization_path(
        &self,
        summary: &str,
        file_path: &str,
        folder_structure: &[String],
        model: &str,
    ) -> Result<String, String> {
        let prompt = Self::create_organization_prompt(summary, file_path, folder_structure);
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

    /// Create a prompt for summary generation
    fn create_summary_prompt(content: &str) -> String {
        format!(
            "Read the text below and write a short summary.\nRules:\n* Maximum 150 words.\n* Use specific keywords from the text.\n* Focus on the main topic, key ideas, and important keywords.\n* Do not add opinions or extra details.\n* Write in clear, plain language.\n\nText: \"\"\"{}\"\"\"\n\nSummary:",
            content
        )
    }

    /// Create a prompt for filename generation
    fn create_filename_prompt(summary: &str) -> String {
        format!(
            "Based on the summary below, generate a clear and descriptive filename.\nRules:\n* Maximum 3 to 4 words.\n* Use only nouns and adjectives (no verbs).\n* Use lowercase letters only.\n* Connect words with underscores.\n* Do not include words like 'text', 'document', 'file', 'pdf', etc.\n* Focus on the main subject, key method/topic, and context or outcome.\n* Output only the filename, with no extra text.\n\nSummary: \"\"\"{}\"\"\"\n\nFilename:",
            summary
        )
    }

    /// Create a prompt for organization path generation
    fn create_organization_prompt(
        summary: &str,
        file_path: &str,
        folder_structure: &[String],
    ) -> String {
        let folder_list = folder_structure.join(", ");

        format!(
            "Based on the file summary and existing folder structure, suggest the best organization path.\n\nFile Summary: \"\"\"{}\"\"\"\nFile Path: {}\nExisting Folders: [{}]\n\nRules:\n* Choose from existing folders when possible\n* If no existing folder fits, suggest ONE new folder name\n* Use clear, descriptive folder names\n* Focus on content type, subject matter, or purpose\n* Output format: folder_name/subfolder_name (if needed)\n* Keep it simple - maximum 2 levels deep\n\nOrganization Path:",
            summary, file_path, folder_list
        )
    }
}
