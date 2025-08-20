use anyhow::{Result, anyhow};
use async_trait::async_trait;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, Special};
use llama_cpp_2::sampling::LlamaSampler;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::pin::pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

use crate::constants::*;
use crate::error::LlmError;
use crate::{FileLlm, LlmConfig};

/// Loaded model components that can be reused across inference calls
struct LoadedModel {
    backend: LlamaBackend,
    model: LlamaModel,
}

/// A client for running inference with local GGUF models using llama.cpp
pub struct LocalLlmClient {
    config: LlmConfig,
    // Cached model components - wrapped in Arc<Mutex<Option<>>> for thread-safe lazy loading
    loaded_model: Arc<Mutex<Option<LoadedModel>>>,
}

impl LocalLlmClient {
    /// Creates a new LocalLlmClient instance with validation
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        config.validate()?;

        Ok(Self {
            config,
            loaded_model: Arc::new(Mutex::new(None)),
        })
    }

    /// Creates a client with default configuration for the given model path
    pub fn with_model_path(model_path: PathBuf) -> Result<Self, LlmError> {
        let config = LlmConfig::new(model_path);
        Self::new(config)
    }

    /// Gets a reference to the current configuration
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Loads the model if not already loaded (lazy loading)
    async fn ensure_model_loaded(&self) -> Result<(), LlmError> {
        let mut loaded_model = self.loaded_model.lock().await;

        if loaded_model.is_none() {
            let model_path = self.config.model_path.clone();
            #[cfg(any(feature = "cuda", feature = "vulkan"))]
            let gpu_layers = self.config.gpu_layers;

            // Load model in a blocking task
            let loaded = task::spawn_blocking(move || -> Result<LoadedModel, LlmError> {
                // Initialize the backend
                let mut backend = LlamaBackend::init().map_err(|e| {
                    LlmError::model_load(
                        &model_path.display().to_string(),
                        format!("Failed to initialize backend: {}", e),
                    )
                })?;

                backend.void_logs();

                // Set up model parameters
                let model_params = {
                    #[cfg(any(feature = "cuda", feature = "vulkan"))]
                    {
                        LlamaModelParams::default().with_n_gpu_layers(gpu_layers as i32)
                    }
                    #[cfg(not(any(feature = "cuda", feature = "vulkan")))]
                    {
                        LlamaModelParams::default()
                    }
                };

                let model_params = pin!(model_params);

                // Load the model
                let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
                    .map_err(|e| {
                        LlmError::model_load(
                            &model_path.display().to_string(),
                            format!("Failed to load model: {}", e),
                        )
                    })?;

                Ok(LoadedModel { backend, model })
            })
            .await
            .map_err(|e| LlmError::Internal(anyhow!("Task join error: {}", e)))??;

            *loaded_model = Some(loaded);
        }

        Ok(())
    }

    /// Preloads the model. This is optional - the model will be loaded automatically
    /// on the first inference call if not already loaded.
    pub async fn preload_model(&self) -> Result<(), LlmError> {
        self.ensure_model_loaded().await
    }

    /// Returns true if the model is currently loaded in memory
    pub async fn is_model_loaded(&self) -> bool {
        let loaded_model = self.loaded_model.lock().await;
        loaded_model.is_some()
    }

    /// Runs inference with the given prompt and returns the generated text
    async fn run_inference(&self, prompt: &str) -> Result<String, LlmError> {
        // Ensure model is loaded
        self.ensure_model_loaded().await?;

        let max_tokens = self.config.max_tokens;
        let context_size = self.config.context_size;
        let threads = self.config.threads;
        let prompt = prompt.to_string();

        // Clone the loaded model for use in the blocking task
        let loaded_model = self.loaded_model.clone();

        // Run the inference in a blocking task to avoid blocking the async runtime
        task::spawn_blocking(move || -> Result<String, LlmError> {
            // Get the loaded model (we know it's loaded because ensure_model_loaded succeeded)
            let loaded_model_guard = loaded_model.blocking_lock();
            let loaded_model_ref = loaded_model_guard.as_ref().unwrap();

            // Initialize the context
            let mut ctx_params = LlamaContextParams::default()
                .with_n_ctx(Some(NonZeroU32::new(context_size).unwrap()))
                .with_n_batch(4096);

            if let Some(threads) = threads {
                ctx_params = ctx_params.with_n_threads(threads as i32);
            }

            let mut ctx = loaded_model_ref.model
                .new_context(&loaded_model_ref.backend, ctx_params)
                .map_err(|e| LlmError::ContextCreation(format!("Failed to create context: {}", e)))?;

            // Tokenize the prompt
            let tokens_list = loaded_model_ref.model
                .str_to_token(&prompt, AddBos::Always)
                .map_err(|e| LlmError::Tokenization(format!("Failed to tokenize prompt: {}", e)))?;

            let n_ctx = ctx.n_ctx() as i32;
            let n_kv_req = tokens_list.len() as i32 + (max_tokens as i32 - tokens_list.len() as i32);

            // Check if we have enough context space
            if n_kv_req > n_ctx {
                return Err(LlmError::Inference(format!(
                    "Required KV cache size ({}) exceeds context size ({}). Either reduce max_tokens or increase context_size",
                    n_kv_req, n_ctx
                )));
            }

            if tokens_list.len() >= context_size as usize {
                return Err(LlmError::Inference(
                    "The prompt is too long, it has more tokens than context_size".to_string(),
                ));
            }


            // Create a batch for token processing - ensure batch size is at least as large as prompt tokens
            let mut batch = LlamaBatch::new(8192, 1);

            let last_index: i32 = (tokens_list.len() - 1) as i32;
            for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
                let is_last = i == last_index;
                batch.add(token, i, &[0], is_last).map_err(|e| {
                    LlmError::Inference(format!("Failed to add token to batch: {}", e))
                })?;
            }

            // Process the initial batch
            ctx.decode(&mut batch)
                .map_err(|e| LlmError::Inference(format!("Failed to decode initial batch: {}", e)))?;

            // Initialize sampler
            let mut sampler = LlamaSampler::chain_simple([
                LlamaSampler::temp(0.3),
                LlamaSampler::top_k(40),
                LlamaSampler::greedy(),
            ]);

            // Generate tokens
            let mut generated_text = String::new();
            let mut n_cur = batch.n_tokens();
            let mut decoder = encoding_rs::UTF_8.new_decoder();

            while n_cur <= context_size as i32 {
                // Sample the next token
                let token = sampler.sample(&ctx, batch.n_tokens() - 1);
                sampler.accept(token);

                // Check if it's end of generation
                if loaded_model_ref.model.is_eog_token(token) {
                    break;
                }

                // Convert token to string and append to result
                let output_bytes = loaded_model_ref.model
                    .token_to_bytes(token, Special::Tokenize)
                    .map_err(|e| LlmError::Inference(format!("Failed to convert token to bytes: {}", e)))?;

                let mut output_string = String::with_capacity(32);
                let _decode_result = decoder.decode_to_string(&output_bytes, &mut output_string, false);
                generated_text.push_str(&output_string);

                // Prepare for next iteration
                batch.clear();
                batch.add(token, n_cur, &[0], true).map_err(|e| {
                    LlmError::Inference(format!("Failed to add token to batch: {}", e))
                })?;

                n_cur += 1;

                // Decode the next token
                ctx.decode(&mut batch)
                    .map_err(|e| LlmError::Inference(format!("Failed to decode batch: {}", e)))?;
            }

            Ok(generated_text.trim().to_string())
        })
        .await
        .map_err(|e| LlmError::Internal(anyhow!("Task join error: {}", e)))?
    }

    /// Creates a prompt for summarizing content
    fn create_summary_prompt(&self, content: &str) -> String {
        let truncated_content = Self::truncate_content(content, DEFAULT_CONTENT_TRUNCATION);

        format!(
            r#"<|im_start|>system
You are a helpful assistant.<|im_end|>
<|im_start|>user
Provide a concise and accurate summary of the following text, focusing on the main ideas and key details.
Limit your summary to a maximum of 150 words.

Text: {}

Summary:<|im_end|>
<|im_start|>assistant"#,
            truncated_content
        )
    }

    /// Creates a prompt for filename suggestion based on a summary
    fn create_filename_prompt(&self, summary: &str) -> String {
        format!(
            r#"<|im_start|>system
You are a helpful assistant.<|im_end|>
<|im_start|>user
Based on the summary below, generate a specific and descriptive filename that captures the essence of the document.
Limit the filename to a maximum of 3 words. Use nouns and avoid starting with verbs like 'depicts', 'shows', 'presents', etc.
Do not include any data type words like 'text', 'document', 'pdf', etc. Use only letters and connect words with underscores.

Summary: {}

Examples:
1. Summary: A research paper on the fundamentals of string theory.
   Filename: fundamentals_string_theory

2. Summary: An article discussing the effects of climate change on polar bears.
   Filename: climate_change_polar_bears

Now generate the filename.

Output only the filename, without any additional text.

Filename:<|im_end|>
<|im_start|>assistant"#,
            summary
        )
    }

    /// Truncates content to specified length with word boundary awareness
    fn truncate_content(content: &str, max_length: usize) -> String {
        // If content is shorter than limit in chars, return as-is
        if content.chars().count() <= max_length {
            return content.to_string();
        }

        // Truncate by character count to avoid splitting multi-byte chars
        let truncated = content.chars().take(max_length).collect::<String>();

        // Try to truncate at last word boundary
        if let Some(pos) = truncated.rfind(' ') {
            truncated[..pos].to_string()
        } else {
            truncated
        }
    }

    /// Cleans and validates a suggested filename
    fn clean_filename(raw_filename: &str) -> String {
        let cleaned = raw_filename
            .lines()
            .next()
            .unwrap_or(raw_filename)
            .trim()
            .chars()
            .filter(|c| c.is_alphanumeric() || matches!(*c, '_' | '-' | ' '))
            .collect::<String>()
            .trim()
            .replace("  ", " ")
            .replace(' ', "_");

        if cleaned.is_empty() {
            FALLBACK_FILENAME.to_string()
        } else {
            cleaned
        }
    }
}

#[async_trait]
impl FileLlm for LocalLlmClient {
    async fn suggest_filename(&self, _original_filename: &str, content: &str) -> Result<String> {
        if content.trim().is_empty() {
            return Ok(FALLBACK_FILENAME.to_string());
        }

        // Step 1: Generate summary
        let summary_prompt = self.create_summary_prompt(content);
        let summary = self
            .run_inference(&summary_prompt)
            .await
            .map_err(|e| anyhow!("Failed to generate content summary: {}", e))?;

        // Step 2: Generate filename based on summary
        let filename_prompt = self.create_filename_prompt(&summary);
        let suggested_name = self
            .run_inference(&filename_prompt)
            .await
            .map_err(|e| anyhow!("Failed to generate filename suggestion: {}", e))?;

        Ok(Self::clean_filename(&suggested_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LlmConfig;
    use std::fs;

    // Helper function to create a temporary config for testing
    fn create_test_config() -> LlmConfig {
        // Create a temporary file for testing
        let temp_file = std::env::temp_dir().join("test_model.gguf");
        fs::write(&temp_file, "dummy content").expect("Failed to create test file");
        LlmConfig::new(temp_file)
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config();
        let client = LocalLlmClient::new(config.clone()).unwrap();
        assert_eq!(client.config.model_path, config.model_path);
        assert_eq!(client.config.max_tokens, DEFAULT_MAX_TOKENS);
        assert_eq!(client.config.context_size, DEFAULT_CONTEXT_SIZE);
    }

    #[test]
    fn test_client_configuration() {
        let config = create_test_config()
            .with_max_tokens(1024)
            .with_context_size(4096)
            .with_threads(8)
            .with_seed(42);

        let client = LocalLlmClient::new(config).unwrap();

        assert_eq!(client.config.max_tokens, 1024);
        assert_eq!(client.config.context_size, 4096);
        assert_eq!(client.config.threads, Some(8));
        assert_eq!(client.config.seed, 42);
    }

    #[test]
    fn test_summary_prompt_creation() {
        let config = create_test_config();
        let client = LocalLlmClient::new(config).unwrap();
        let content = "This is a document about machine learning and artificial intelligence.";
        let prompt = client.create_summary_prompt(content);

        assert!(prompt.contains("machine learning and artificial intelligence"));
        assert!(prompt.contains(content));
    }

    #[test]
    fn test_filename_prompt_creation() {
        let config = create_test_config();
        let client = LocalLlmClient::new(config).unwrap();
        let summary =
            "A research paper discussing machine learning algorithms and their applications.";
        let prompt = client.create_filename_prompt(summary);

        assert!(prompt.contains("machine learning algorithms"));
        assert!(prompt.contains(summary));
    }

    #[test]
    fn test_summary_prompt_truncation() {
        let config = create_test_config();
        let client = LocalLlmClient::new(config).unwrap();
        let long_content = "a".repeat(3000);
        let prompt = client.create_summary_prompt(&long_content);

        // The prompt should not contain the full 3000 characters
        assert!(prompt.len() < 10_000); // Some buffer for the prompt text
    }

    #[tokio::test]
    async fn test_model_not_loaded_initially() {
        let config = create_test_config();
        let client = LocalLlmClient::new(config).unwrap();
        assert!(!client.is_model_loaded().await);
    }

    #[test]
    fn test_clean_filename() {
        assert_eq!(LocalLlmClient::clean_filename("hello world"), "hello_world");
        assert_eq!(LocalLlmClient::clean_filename("test@#$%file"), "testfile");
        assert_eq!(LocalLlmClient::clean_filename(""), FALLBACK_FILENAME);
        assert_eq!(LocalLlmClient::clean_filename("   "), FALLBACK_FILENAME);
        assert_eq!(
            LocalLlmClient::clean_filename("multiple  spaces"),
            "multiple_spaces"
        );
    }

    #[test]
    fn test_truncate_content() {
        let short_content = "short";
        assert_eq!(
            LocalLlmClient::truncate_content(short_content, 100),
            short_content
        );

        let long_content = "this is a very long content that should be truncated";
        let truncated = LocalLlmClient::truncate_content(long_content, 20);
        assert!(truncated.len() <= 20);
        assert!(long_content.starts_with(&truncated));
    }

    #[test]
    fn test_config_validation() {
        // Test with non-existent path
        let invalid_config = LlmConfig::new(PathBuf::from("/path/that/does/not/exist"));
        assert!(LocalLlmClient::new(invalid_config).is_err());

        // Test with valid path
        let valid_config = create_test_config();
        assert!(LocalLlmClient::new(valid_config).is_ok());
    }
}
