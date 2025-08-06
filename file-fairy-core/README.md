# File Fairy Core

The core business logic library for the File Fairy ecosystem. This crate provides the fundamental services for intelligent file organization, directory scanning, and system information reporting. It orchestrates the interaction between file extractors, LLM services, and the file system while maintaining a clean, async-first API.

## 🚀 Features

- **File Organization Service**: AI-powered file organization with content-based naming
- **Directory Scanning**: Comprehensive directory analysis with detailed statistics
- **Interactive Mode**: Step-by-step user approval for file operations
- **Dry Run Mode**: Safe preview of all operations before execution
- **Flexible Configuration**: Extensive options for customizing behavior
- **Category System**: Intelligent file categorization based on type and content
- **Error Handling**: Robust error management with detailed context
- **System Information**: Comprehensive reporting of capabilities and status

## 🏗️ Architecture

File Fairy Core is designed around a modular, service-oriented architecture:

```
┌─────────────────────────────────────────────────┐
│                File Fairy Core                  │
├─────────────────────────────────────────────────┤
│ ┌─────────────────┐ ┌─────────────────────────┐ │
│ │ Organization    │ │ Directory Scanning      │ │
│ │ Service         │ │ Service                 │ │
│ └─────────────────┘ └─────────────────────────┘ │
│ ┌─────────────────┐ ┌─────────────────────────┐ │
│ │ File Category   │ │ Information System      │ │
│ │ System          │ │                         │ │
│ └─────────────────┘ └─────────────────────────┘ │
├─────────────────────────────────────────────────┤
│              External Dependencies              │
├─────────────────────────────────────────────────┤
│ file-fairy-extractors │ file-fairy-llm          │
└─────────────────────────────────────────────────┘
```

### Key Components

- **`FileOrganizeService`** - Main orchestrator for file organization operations
- **`DirectoryScanner`** - High-performance directory analysis and statistics
- **`FileCategory`** - Type-safe file categorization system
- **`FileFairyInfo`** - Comprehensive system and capability reporting
- **Configuration System** - Flexible, builder-pattern configuration
- **Error System** - Rich error types with context and recovery information

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
file-fairy-core = { path = "../file-fairy-core" }
```

## 🎯 Quick Start

### Basic File Organization

```rust
use file_fairy_core::{FileOrganizeService, OrganizeOptions, AppConfig};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the service
    let config = AppConfig::new()
        .with_model_path("models/gemma-3n-E2B-it-Q4_K_M.gguf")
        .with_max_tokens(64)
        .with_threads(8);

    let service = FileOrganizeService::new(config)?;

    // Set up organization options
    let options = OrganizeOptions::new()
        .with_recursive(true)
        .with_apply(false); // Dry run mode

    // Organize files
    let results = service.organize(Path::new("/path/to/organize"), options).await?;

    // Display results
    println!("{}", results.format_preview());

    Ok(())
}
```

### Directory Scanning

```rust
use file_fairy_core::{DirectoryScanner, ScanOptions};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = DirectoryScanner::with_options(
        ScanOptions::new()
            .with_recursive(true)
            .with_max_depth(5)
            .with_max_file_size(100 * 1024 * 1024) // 100MB limit
    );

    let results = scanner.scan(Path::new("/path/to/scan")).await?;

    println!("{}", results.format_summary());

    Ok(())
}
```

### Interactive File Organization

```rust
use file_fairy_core::{FileOrganizeService, OrganizeOptions, AppConfig};

async fn interactive_organization() -> Result<(), Box<dyn std::error::Error>> {
    let service = FileOrganizeService::new(AppConfig::new())?;

    let options = OrganizeOptions::new()
        .with_interactive(true)  // Enable step-by-step approval
        .with_apply(true)        // Actually perform operations
        .with_recursive(true);

    let results = service.organize(Path::new("./documents"), options).await?;

    println!("Organization completed!");
    println!("Successful: {}", results.successful_actions);
    println!("Failed: {}", results.failed_actions);

    Ok(())
}
```

## 📋 Core Services

### File Organization Service

The `FileOrganizeService` is the main orchestrator for intelligent file organization:

#### Features

- **Content-Based Naming**: Uses LLM analysis of file content to generate meaningful names
- **Category Organization**: Automatically categorizes files by type and purpose
- **Interactive Approval**: Optional step-by-step user approval process
- **Dry Run Mode**: Safe preview mode that shows proposed changes without executing them
- **Batch Processing**: Efficient processing of multiple files with model caching

#### Configuration Options

```rust
pub struct OrganizeOptions {
    pub apply: bool,                    // Execute actions vs. dry run
    pub interactive: bool,              // Enable user approval
    pub recursive: bool,                // Scan subdirectories
    pub max_depth: Option<usize>,       // Limit recursion depth
    pub target_dir: Option<PathBuf>,    // Target directory for organized files
    pub follow_symlinks: bool,          // Follow symbolic links
    pub max_file_size: Option<u64>,     // File size limit in bytes
    pub include_categories: Option<Vec<FileCategory>>, // Category filter
}
```

#### Example Usage Patterns

**Dry Run (Preview Mode)**:

```rust
let options = OrganizeOptions::new(); // apply: false by default
let results = service.organize(path, options).await?;
println!("{}", results.format_preview()); // Show proposed changes
```

**Interactive Mode**:

```rust
let options = OrganizeOptions::new()
    .with_interactive(true)
    .with_apply(true);
let results = service.organize(path, options).await?;
// User will be prompted for each file: [y]es, [n]o, [s]kip, [e]dit, [q]uit
```

**Filtered Organization**:

```rust
let options = OrganizeOptions::new()
    .with_categories(vec![FileCategory::Documents])
    .with_max_file_size(50 * 1024 * 1024) // 50MB limit
    .with_max_depth(3);
```

### Directory Scanner

The `DirectoryScanner` provides comprehensive directory analysis:

#### Features

- **File Statistics**: Count, size, and category breakdowns
- **Performance Metrics**: Fast, async scanning with configurable limits
- **Category Analysis**: Detailed breakdown by file type and category
- **Error Reporting**: Tracks and reports access errors during scanning

#### Scan Options

```rust
pub struct ScanOptions {
    pub recursive: bool,                // Scan subdirectories
    pub max_depth: Option<usize>,       // Recursion depth limit
    pub follow_symlinks: bool,          // Follow symbolic links
    pub max_file_size: Option<u64>,     // File size limit
    pub include_categories: Option<Vec<FileCategory>>, // Category filter
}
```

#### Results Format

```rust
pub struct ScanResults {
    pub scanned_path: PathBuf,          // Path that was scanned
    pub total_files: usize,             // Total file count
    pub total_directories: usize,       // Total directory count
    pub total_size: u64,                // Total size in bytes
    pub category_stats: CategoryStats,  // Per-category statistics
    pub timestamp: DateTime<Utc>,       // When scan was performed
    pub errors: usize,                  // Number of access errors
}
```

### File Category System

Type-safe file categorization with extensible design:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileCategory {
    Documents,    // PDF, DOCX, TXT, MD, HWP, PPTX
    Data,         // XLSX, CSV, JSON
    Unsupported,  // Everything else
}
```

#### Category Features

- **Auto-Detection**: Automatic categorization based on file extension
- **Extractability**: Built-in knowledge of which files support content extraction
- **Extensible**: Easy to add new categories and formats
- **Serializable**: Full serde support for config files and APIs

### System Information

Comprehensive reporting of File Fairy's capabilities:

```rust
let info = FileFairyInfo::new();

// Complete system overview
println!("{}", info.format_summary());

// Just supported formats
println!("{}", info.format_formats());

// System-specific information
println!("{}", info.format_system());
```

## 🛠️ Configuration

### Application Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model_path: PathBuf,      // Path to LLM model file
    pub max_tokens: u32,          // Token generation limit
    pub context_size: u32,        // LLM context window size
    pub threads: u32,             // Processing thread count
    pub seed: u32,                // Random seed for reproducibility
}
```

### Builder Pattern

All configuration uses a fluent builder pattern:

```rust
let config = AppConfig::new()
    .with_model_path("models/model.gguf")
    .with_max_tokens(128)
    .with_context_size(4096)
    .with_threads(num_cpus::get() as u32)
    .with_seed(42);
```

### Validation

All configurations include built-in validation:

```rust
config.validate()?; // Returns Result<(), String>
```

## ⚡ Performance Features

### Async-First Design

All I/O operations are fully async to prevent blocking:

```rust
// Non-blocking directory traversal
let results = scanner.scan(path).await?;

// Non-blocking file processing
let organize_results = service.organize(path, options).await?;
```

### Memory Efficiency

- **Streaming Processing**: Files processed one at a time to limit memory usage
- **Model Caching**: LLM models cached between operations for efficiency
- **Stack-Based Traversal**: Directory scanning uses iterative algorithms to prevent stack overflow

### Concurrent Safety

- **Thread-Safe**: All services can be safely shared between threads
- **Arc-Compatible**: Designed to work with `Arc<Service>` for multi-threaded access
- **No Global State**: All state encapsulated in service instances

## 🛡️ Error Handling

Comprehensive error system with rich context:

```rust
#[derive(Error, Debug)]
pub enum CoreError {
    Io(std::io::Error),                    // File system errors
    Config(config::ConfigError),           // Configuration errors
    UnsupportedFileType { path: PathBuf }, // Unsupported file formats
    PathNotFound { path: PathBuf },        // Missing files/directories
    Extractor { path: PathBuf, source: Box<dyn std::error::Error> }, // Extraction failures
    Llm(anyhow::Error),                    // LLM processing errors
    UserCancelled,                         // User-initiated cancellation
    Generic { message: String },           // Generic errors
}
```

### Error Recovery

Errors include information for recovery decisions:

```rust
match error {
    CoreError::PathNotFound { path } => {
        println!("File not found: {}", path.display());
        // Could retry with different path
    }
    CoreError::UnsupportedFileType { path } => {
        println!("Cannot process: {}", path.display());
        // Skip this file and continue
    }
    CoreError::UserCancelled => {
        println!("Operation cancelled by user");
        // Clean exit
    }
    e if e.is_recoverable() => {
        // Retry logic
    }
    _ => {
        // Fatal error
    }
}
```

## 📊 Output Formats

### Organization Preview

```
🔍 ORGANIZATION PREVIEW (DRY RUN)
═══════════════════════════════════

📁 Path: /Users/example/Documents
🕒 Time: 2025-01-15 10:30:45 UTC
📊 Total Files: 25
✅ Approved: 23

📋 ACTIONS
┌──┬─────────────────────────────┬──────────┬─────────────────────────────┬──────────┐
│  │ Original                    │   Size   │ Suggested                   │ Category │
├──┼─────────────────────────────┼──────────┼─────────────────────────────┼──────────┤
│✓ │ report.pdf                  │ 2.5 MB   │ Q3_Financial_Report_2024.pdf│ document │
│✓ │ notes.txt                   │ 1.2 KB   │ Meeting_Notes_Board_2024.txt│ document │
│✓ │ data.xlsx                   │ 4.1 MB   │ Sales_Data_Analysis_Q3.xlsx │ data     │
└──┴─────────────────────────────┴──────────┴─────────────────────────────┴──────────┘

💡 NEXT STEPS
• Review the suggested changes above
• Run with --apply to execute the approved actions
• Use --interactive for step-by-step control
```

### Scan Results

```
📊 DIRECTORY SCAN RESULTS
═══════════════════════════

📁 Path: /Users/example/Documents
🕒 Scanned: 2025-01-15 10:30:45 UTC
📈 Total Files: 156 files in 12 directories
💾 Total Size: 2.3 GB

📋 BY CATEGORY
┌─────────────┬───────┬──────────┬─────────────┐
│ Category    │ Count │ Size     │ Percentage  │
├─────────────┼───────┼──────────┼─────────────┤
│ Documents   │    89 │ 1.2 GB   │      52.1%  │
│ Data        │    34 │ 892.5 MB │      37.8%  │
│ Unsupported │    33 │ 234.2 MB │      10.1%  │
└─────────────┴───────┴──────────┴─────────────┘
```

## 🧪 Integration Patterns

### With CLI Applications

```rust
use file_fairy_core::{FileOrganizeService, OrganizeOptions};

// CLI would handle argument parsing and call:
async fn handle_organize_command(
    path: PathBuf,
    apply: bool,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let service = FileOrganizeService::new(AppConfig::new())?;

    let options = OrganizeOptions::new()
        .with_apply(apply)
        .with_interactive(interactive);

    let results = service.organize(&path, options).await?;

    if !apply {
        println!("{}", results.format_preview());
    } else {
        println!("Organization completed: {} files processed", results.total_actions());
    }

    Ok(())
}
```

### With Web Services

```rust
use axum::{extract::State, Json};
use std::sync::Arc;

async fn organize_endpoint(
    State(service): State<Arc<FileOrganizeService>>,
    Json(request): Json<OrganizeRequest>,
) -> Result<Json<OrganizeResults>, AppError> {
    let options = OrganizeOptions::new()
        .with_apply(false) // Always dry run for API
        .with_recursive(request.recursive);

    let results = service
        .organize(&request.path, options)
        .await
        .map_err(AppError::from)?;

    Ok(Json(results))
}
```

### Custom Processing Pipelines

```rust
use file_fairy_core::{DirectoryScanner, FileOrganizeService};

async fn custom_pipeline(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Scan to understand the directory
    let scanner = DirectoryScanner::new();
    let scan_results = scanner.scan(path).await?;

    println!("Found {} files totaling {}",
        scan_results.total_files,
        format_size(scan_results.total_size)
    );

    // 2. Organize based on scan results
    if scan_results.total_files > 100 {
        // Large directory - use conservative settings
        let options = OrganizeOptions::new()
            .with_max_file_size(10 * 1024 * 1024) // 10MB limit
            .with_categories(vec![FileCategory::Documents]);
    } else {
        // Small directory - process everything
        let options = OrganizeOptions::new();
    }

    let service = FileOrganizeService::new(AppConfig::new())?;
    let organize_results = service.organize(path, options).await?;

    println!("{}", organize_results.format_preview());

    Ok(())
}
```

## 🚀 Performance Tuning

### Threading Configuration

```rust
// Optimize for CPU-bound workloads
let config = AppConfig::new()
    .with_threads(num_cpus::get() as u32); // Use all available cores

// Optimize for I/O-bound workloads
let config = AppConfig::new()
    .with_threads(8); // Moderate thread count
```

### Memory Usage

```rust
// Large file handling
let options = OrganizeOptions::new()
    .with_max_file_size(100 * 1024 * 1024) // 100MB limit
    .with_max_depth(3); // Limit recursion depth

// Memory-efficient scanning
let scan_options = ScanOptions::new()
    .with_max_file_size(50 * 1024 * 1024); // Skip large files
```

### LLM Configuration

```rust
// Balance between quality and speed
let config = AppConfig::new()
    .with_max_tokens(32)      // Short responses
    .with_context_size(4096); // Moderate context

// High-quality mode
let config = AppConfig::new()
    .with_max_tokens(128)     // Longer responses
    .with_context_size(8192); // Large context
```

## 🧪 Testing

### Unit Testing

```rust
#[tokio::test]
async fn test_file_organization() {
    let config = AppConfig::new();
    let service = FileOrganizeService::new(config).unwrap();

    let options = OrganizeOptions::new().with_apply(false);
    let results = service.organize(Path::new("test_data"), options).await.unwrap();

    assert!(results.total_actions() > 0);
    assert!(results.dry_run);
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_full_pipeline() {
    // Create test directory structure
    let temp_dir = setup_test_directory().await;

    // Scan
    let scanner = DirectoryScanner::new();
    let scan_results = scanner.scan(&temp_dir).await.unwrap();
    assert_eq!(scan_results.total_files, 5);

    // Organize
    let service = FileOrganizeService::new(AppConfig::new()).unwrap();
    let organize_results = service
        .organize(&temp_dir, OrganizeOptions::new())
        .await
        .unwrap();

    assert_eq!(organize_results.total_actions(), 3); // Only extractable files
}
```

## 🤝 Contributing

Areas for contribution:

### Priority Features

- **New File Categories**: Add support for images, audio, video categories
- **Advanced Organization**: Implement folder structure generation based on content
- **Performance Optimization**: Improve scanning and processing speed
- **Configuration**: Add more flexible configuration options

### Development Setup

```bash
# Clone and build
git clone https://github.com/ben-park2001/file-fairy.git
cd file-fairy/file-fairy-core
cargo build

# Run tests
cargo test

# Run with examples
cargo run --example basic_usage
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- **[tokio](https://tokio.rs/)** - Async runtime for Rust
- **[serde](https://serde.rs/)** - Serialization framework
- **[thiserror](https://github.com/dtolnay/thiserror)** - Error handling made easy
- **[chrono](https://github.com/chronotope/chrono)** - Date and time handling
- **[strum](https://github.com/Peternator7/strum)** - Enum utilities

---

For more information about the File Fairy ecosystem, visit the main repository or check out the other crates:

- **file-fairy-cli** - Command-line interface
- **file-fairy-extractors** - File content extraction
- **file-fairy-llm** - Local LLM integration
