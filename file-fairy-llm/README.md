# File Fairy LLM

A high-performance Rust library for running local Large Language Models (LLMs) with a focus on file organization and intelligent filename generation. This crate provides async-first, memory-efficient LLM inference using GGUF models via llama.cpp bindings.

## 🚀 Features

- **Local LLM Inference**: Run GGUF models locally without external API dependencies
- **Intelligent Filename Generation**: AI-powered filename suggestions based on document content
- **Async/Await Support**: Fully async API for non-blocking operations
- **Memory Efficient**: Lazy model loading with intelligent caching
- **GPU Acceleration**: Optional CUDA and Vulkan support for improved performance
- **Thread-Safe**: Safe concurrent access with proper synchronization
- **Configurable**: Extensive configuration options for performance tuning
- **Content-Aware**: Sophisticated prompting system for context-aware filename generation

## 🎯 Primary Use Cases

- **File Organization**: Generate meaningful filenames based on document content
- **Document Classification**: Analyze and categorize documents automatically
- **Batch Processing**: Efficiently process multiple files with cached model loading
- **Content Analysis**: Extract key information from documents for organization

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
file-fairy-llm = { path = "../file-fairy-llm" }

# Optional: Enable GPU acceleration
[features]
cuda = ["file-fairy-llm/cuda"]
vulkan = ["file-fairy-llm/vulkan"]
```

## 🎯 Quick Start

### Basic Usage

```rust
use file_fairy_llm::{LocalLlmClient, LlmConfig, FileLlm};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client with a local GGUF model
    let config = LlmConfig::new(PathBuf::from("models/gemma-3n-E2B-it-Q4_K_M.gguf"))
        .with_max_tokens(64)
        .with_context_size(4096)
        .with_threads(8);

    let client = LocalLlmClient::new(config)?;

    // Generate a filename suggestion
    let original_filename = "document.pdf";
    let content = "Annual financial report for Q3 2024 showing revenue growth...";

    let suggested_filename = client
        .suggest_filename(original_filename, content)
        .await?;

    println!("Suggested filename: {}", suggested_filename);
    // Output: "Annual_Financial_Report_Q3_2024.pdf"

    Ok(())
}
```

### Advanced Configuration

```rust
use file_fairy_llm::{LocalLlmClient, LlmConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = LlmConfig::new(PathBuf::from("models/model.gguf"))
        .with_max_tokens(128)           // Generate up to 128 tokens
        .with_context_size(8192)        // Use 8K context window
        .with_threads(16)               // Use 16 CPU threads
        .with_seed(42);                 // Set deterministic seed

    #[cfg(feature = "cuda")]
    let config = config.with_gpu_layers(32); // Offload 32 layers to GPU

    let client = LocalLlmClient::new(config)?;

    // Optionally preload the model for faster first inference
    client.preload_model().await?;

    Ok(())
}
```

### Batch Processing

```rust
use file_fairy_llm::{LocalLlmClient, LlmConfig, FileLlm};
use std::path::PathBuf;

async fn process_documents() -> Result<(), Box<dyn std::error::Error>> {
    let client = LocalLlmClient::new(
        LlmConfig::new(PathBuf::from("models/model.gguf"))
    )?;

    let documents = vec![
        ("report.pdf", "Quarterly financial analysis..."),
        ("memo.docx", "Meeting notes from board discussion..."),
        ("presentation.pptx", "Product launch strategy slides..."),
    ];

    // Process multiple documents efficiently (model is cached after first use)
    for (original_name, content) in documents {
        let suggested_name = client
            .suggest_filename(original_name, content)
            .await?;
        println!("{} -> {}", original_name, suggested_name);
    }

    Ok(())
}
```

## 🏗️ Architecture

### Core Components

#### `FileLlm` Trait

The main interface for LLM operations:

```rust
#[async_trait]
pub trait FileLlm: Send + Sync {
    async fn suggest_filename(&self, original_filename: &str, content: &str) -> Result<String>;
}
```

#### `LocalLlmClient`

Primary implementation using llama.cpp:

- **Lazy Loading**: Models are loaded on first use
- **Memory Caching**: Loaded models stay in memory for subsequent calls
- **Thread Safety**: Safe concurrent access with proper locking
- **Resource Management**: Efficient handling of GPU/CPU resources

#### `LlmConfig`

Comprehensive configuration system:

```rust
pub struct LlmConfig {
    pub model_path: PathBuf,
    pub max_tokens: u32,
    pub context_size: u32,
    pub threads: Option<u32>,
    pub seed: u32,
    #[cfg(any(feature = "cuda", feature = "vulkan"))]
    pub gpu_layers: u32,
}
```

## 📋 Configuration Options

### Core Settings

| Parameter      | Default  | Description                     |
| -------------- | -------- | ------------------------------- |
| `model_path`   | Required | Path to GGUF model file         |
| `max_tokens`   | 32       | Maximum tokens to generate      |
| `context_size` | 8192     | Context window size             |
| `threads`      | Auto     | Number of CPU threads           |
| `seed`         | 103      | Random seed for reproducibility |

### GPU Acceleration (Optional)

| Parameter    | Default | Description                        |
| ------------ | ------- | ---------------------------------- |
| `gpu_layers` | 1000    | Number of layers to offload to GPU |

### Content Processing

| Parameter            | Default              | Description                       |
| -------------------- | -------------------- | --------------------------------- |
| `content_truncation` | 8000                 | Max content length for processing |
| `fallback_filename`  | "suggested_filename" | Fallback when generation fails    |

## 🤖 Filename Generation System

### Intelligent Prompting

The system uses a sophisticated prompting strategy that includes:

- **Content Analysis**: Deep understanding of document purpose and context
- **Language Detection**: Automatic detection and handling of multiple languages
- **Format Preservation**: Maintains appropriate file extensions
- **Business Context**: Considers organizational and domain-specific naming conventions

### Prompt Structure

```
You are File Fairy, an AI-powered file organization assistant...

## Task
Analyze the provided file content and generate a new filename...

## Instructions
### Primary Objectives
1. Content Analysis: Thoroughly analyze document content
2. Language Detection: Identify primary language
3. Clarity: Create immediately understandable filenames
4. Conciseness: Keep filenames reasonably short (3-8 words)

### Naming Guidelines
- Use descriptive keywords that capture document essence
- Include document type when relevant (report, invoice, manual, etc.)
- Incorporate dates in YYYY-MM-DD format when chronologically important
- Use proper capitalization and file-system safe characters
```

### Example Transformations

| Original       | Content Summary                         | Generated                                       |
| -------------- | --------------------------------------- | ----------------------------------------------- |
| `document.pdf` | Annual financial report for FY 2023     | `Annual_Financial_Report_FY2023.pdf`            |
| `temp123.docx` | Emergency evacuation procedure (French) | `Procedure_Evacuation_Urgence_Incendie.docx`    |
| `scan001.jpg`  | Receipt from ABC Electronics for laptop | `Receipt_ABC_Electronics_Laptop_2024-03-15.jpg` |

## 🛠️ Error Handling

Comprehensive error handling with detailed context:

```rust
pub enum LlmError {
    ModelLoad(String, String),           // Model loading failures
    ContextCreation(String),             // Context initialization errors
    Tokenization(String),                // Token processing issues
    Inference(String),                   // Runtime inference errors
    Configuration(String),               // Invalid configuration
    ModelNotFound(String),               // Missing model file
    Io(std::io::Error),                 // File system errors
    Internal(anyhow::Error),            // Internal library errors
}
```

### Error Handling Example

```rust
match client.suggest_filename("doc.pdf", content).await {
    Ok(filename) => println!("Success: {}", filename),
    Err(LlmError::ModelNotFound(path)) => {
        eprintln!("Model not found at: {}", path);
    }
    Err(LlmError::Inference(msg)) => {
        eprintln!("Inference failed: {}", msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## 🚀 Performance Optimization

### Memory Management

- **Lazy Loading**: Models loaded only when needed
- **Smart Caching**: Loaded models persist between calls
- **Resource Cleanup**: Automatic resource management

### Concurrency

```rust
// Safe concurrent access to the same client
let client = Arc::new(LocalLlmClient::new(config)?);

let tasks: Vec<_> = files.into_iter().map(|(name, content)| {
    let client = Arc::clone(&client);
    tokio::spawn(async move {
        client.suggest_filename(&name, &content).await
    })
}).collect();

// Process results
for task in tasks {
    match task.await? {
        Ok(filename) => println!("Generated: {}", filename),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Performance Tips

1. **Preload Models**: Use `preload_model()` for better first-call performance
2. **Reuse Clients**: Create one client and reuse it for multiple operations
3. **Optimize Context Size**: Balance memory usage vs. content processing capability
4. **GPU Acceleration**: Enable CUDA/Vulkan features for compatible hardware
5. **Thread Tuning**: Adjust thread count based on hardware capabilities

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run with specific features
cargo test --features cuda
cargo test --features vulkan

# Run with output
cargo test -- --nocapture
```

### Example Test

```rust
#[tokio::test]
async fn test_filename_generation() {
    let config = LlmConfig::new(PathBuf::from("test_model.gguf"));
    let client = LocalLlmClient::new(config).unwrap();

    let result = client.suggest_filename(
        "document.pdf",
        "Annual report for 2024"
    ).await;

    assert!(result.is_ok());
    let filename = result.unwrap();
    assert!(filename.contains("Annual"));
    assert!(filename.ends_with(".pdf"));
}
```

## 🔧 Hardware Requirements

### Minimum Requirements

- **CPU**: Modern x64 processor
- **RAM**: 4GB+ (depends on model size)
- **Storage**: Space for GGUF model files (1GB-100GB+)

### Recommended for GPU Acceleration

- **CUDA**: NVIDIA GPU with CUDA 11.0+ support
- **Vulkan**: Modern GPU with Vulkan 1.2+ support
- **VRAM**: 6GB+ for larger models

### Model Compatibility

Supports GGUF models from various sources:

- **Hugging Face**: Quantized GGUF models
- **llama.cpp**: Native GGUF format
- **Custom Models**: Any llama.cpp compatible GGUF

## 🔌 Integration Examples

### With File Processing Pipeline

```rust
use file_fairy_llm::{LocalLlmClient, LlmConfig, FileLlm};
use file_fairy_extractors::{extractor_from_file_path, Extractor};

async fn smart_file_rename(file_path: &Path) -> Result<String> {
    // Extract content
    let extractor = extractor_from_file_path(file_path)
        .ok_or("Unsupported file format")?;
    let content = extractor.extract(file_path).await?;

    // Generate filename
    let llm_client = LocalLlmClient::with_model_path(
        PathBuf::from("models/model.gguf")
    )?;

    let original_name = file_path.file_name()
        .unwrap()
        .to_string_lossy();

    let new_name = llm_client
        .suggest_filename(&original_name, &content)
        .await?;

    Ok(new_name)
}
```

### Web Service Integration

```rust
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct FilenameRequest {
    original: String,
    content: String,
}

async fn suggest_filename_handler(
    State(client): State<Arc<LocalLlmClient>>,
    Json(request): Json<FilenameRequest>,
) -> Result<Json<String>, StatusCode> {
    let filename = client
        .suggest_filename(&request.original, &request.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(filename))
}
```

## 📊 Benchmarks

Performance characteristics (approximate, varies by hardware):

| Operation                   | First Call | Subsequent Calls | Memory Usage |
| --------------------------- | ---------- | ---------------- | ------------ |
| Model Loading               | 2-10s      | Cached           | 2-8GB        |
| Filename Generation         | 1-5s       | 100-500ms        | Stable       |
| Batch Processing (10 files) | 10-30s     | 2-8s             | Stable       |

## 🤝 Contributing

Contributions are welcome! Areas for improvement:

### Priority Areas

- **Model Support**: Add support for more model architectures
- **Performance**: Optimize inference speed and memory usage
- **Features**: Extend prompting capabilities and output formats
- **Testing**: Add comprehensive benchmarks and integration tests
- **Documentation**: Improve examples and use case guides

### Development Setup

```bash
# Clone the repository
git clone https://github.com/ben-park2001/file-fairy.git
cd file-fairy/file-fairy-llm

# Install dependencies (requires Rust 1.70+)
cargo build

# Run tests
cargo test

# Run examples (requires model file)
cargo run --example local_client_example
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- **[llama-cpp-2](https://crates.io/crates/llama-cpp-2)** - Rust bindings for llama.cpp
- **[llama.cpp](https://github.com/ggerganov/llama.cpp)** - High-performance LLM inference
- **[Tokio](https://tokio.rs/)** - Async runtime for Rust
- **GGUF Format** - Efficient model serialization format

---

For more information about the File Fairy ecosystem, visit the main repository or check out the other crates:

- **file-fairy-cli** - Command-line interface
- **file-fairy-core** - Core organization logic
- **file-fairy-extractors** - File content extraction
