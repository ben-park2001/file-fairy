use crate::constants::*;
use crate::error::LlmError;
use std::path::PathBuf;

/// Configuration for the LocalLlmClient
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub model_path: PathBuf,
    pub max_tokens: u32,
    pub context_size: u32,
    pub threads: Option<u32>,
    pub seed: u32,
    #[cfg(any(feature = "cuda", feature = "vulkan"))]
    pub gpu_layers: u32,
}

impl LlmConfig {
    /// Creates a new configuration with the specified model path
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            max_tokens: DEFAULT_MAX_TOKENS,
            context_size: DEFAULT_CONTEXT_SIZE,
            threads: None,
            seed: DEFAULT_SEED,
            #[cfg(any(feature = "cuda", feature = "vulkan"))]
            gpu_layers: DEFAULT_GPU_LAYERS,
        }
    }

    /// Sets the maximum number of tokens to generate
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Sets the context size
    pub fn with_context_size(mut self, context_size: u32) -> Self {
        self.context_size = context_size;
        self
    }

    /// Sets the number of threads to use
    pub fn with_threads(mut self, threads: u32) -> Self {
        self.threads = Some(threads);
        self
    }

    /// Sets the random seed
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    /// Sets the number of GPU layers to offload
    #[cfg(any(feature = "cuda", feature = "vulkan"))]
    pub fn with_gpu_layers(mut self, gpu_layers: u32) -> Self {
        self.gpu_layers = gpu_layers;
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), LlmError> {
        if !self.model_path.exists() {
            return Err(LlmError::ModelNotFound(
                self.model_path.display().to_string(),
            ));
        }

        if self.max_tokens == 0 {
            return Err(LlmError::Configuration(
                "max_tokens must be greater than 0".to_string(),
            ));
        }

        if self.context_size < 512 {
            return Err(LlmError::Configuration(
                "context_size must be at least 512".to_string(),
            ));
        }

        Ok(())
    }
}
