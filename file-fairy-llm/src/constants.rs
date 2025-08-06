/// Default configuration values for the LLM client
pub const DEFAULT_MAX_TOKENS: u32 = 32;
pub const DEFAULT_CONTEXT_SIZE: u32 = 8192;
pub const DEFAULT_SEED: u32 = 103;
pub const DEFAULT_BATCH_SIZE: usize = 8192;

#[cfg(any(feature = "cuda", feature = "vulkan"))]
pub const DEFAULT_GPU_LAYERS: u32 = 1000;

/// Content processing constants
pub const DEFAULT_CONTENT_TRUNCATION: usize = 8000;
pub const FALLBACK_FILENAME: &str = "suggested_filename";

/// Filename validation constants
pub const MAX_FILENAME_LENGTH: usize = 255;
