# File Fairy

An intelligent file organization system powered by AI and advanced content analysis. File Fairy automatically organizes, renames, and categorizes your files based on their actual content rather than just file extensions, using local Large Language Models to generate meaningful, descriptive filenames.

## 🌟 Overview

File Fairy transforms chaotic file collections into well-organized, intelligently named archives. By analyzing the actual content of documents using AI, it provides context-aware organization that goes far beyond traditional rule-based systems.

### ✨ What Makes File Fairy Special

- **Content-Aware Organization**: Analyzes actual file content, not just extensions
- **AI-Powered Naming**: Uses local LLMs to generate meaningful, descriptive filenames
- **Multi-Language Support**: Handles documents in multiple languages including Korean (HWP)
- **Safety First**: Dry-run mode by default with interactive approval options
- **Privacy Focused**: Everything runs locally - no data leaves your machine
- **Extensible Architecture**: Modular design makes it easy to add new file types and features

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/ben-park2001/file-fairy.git
cd file-fairy

# Build all components
cargo build --release

# The main CLI will be available at:
./target/release/file-fairy-cli
```

### Your First Organization

```bash
# See what File Fairy would do (dry run - safe!)
./target/release/file-fairy-cli organize ~/Downloads

# If you like what you see, apply the changes
./target/release/file-fairy-cli organize ~/Downloads --apply

# For step-by-step control
./target/release/file-fairy-cli organize ~/Downloads --apply --interactive
```

### Example Transformation

**Before:**

```
Downloads/
├── document.pdf (Financial report for Q3 2024)
├── temp123.docx (Meeting notes from board meeting)
└── file_final.xlsx (Sales data analysis)
```

**After:**

```
Downloads/
├── Q3_Financial_Report_2024.pdf
├── Board_Meeting_Notes_2024.docx
└── Sales_Data_Analysis_Q3_2024.xlsx
```

### 📦 Crate Overview

| Crate                                               | Purpose                      | Key Features                                   |
| --------------------------------------------------- | ---------------------------- | ---------------------------------------------- |
| **[file-fairy-cli](file-fairy-cli/)**               | Command-line interface       | User interaction, safety features, rich output |
| **[file-fairy-core](file-fairy-core/)**             | Business logic orchestration | Service coordination, workflow management      |
| **[file-fairy-extractors](file-fairy-extractors/)** | Content extraction           | Multi-format support, async processing         |
| **[file-fairy-llm](file-fairy-llm/)**               | AI-powered analysis          | Local LLM inference, intelligent naming        |

## 🎯 Core Features

### 🤖 Intelligent File Analysis

File Fairy doesn't just look at file extensions - it actually reads and understands your files:

- **Deep Content Analysis**: Extracts and analyzes the actual text content from documents
- **Context Understanding**: Comprehends document purpose, key topics, and organizational context
- **Multi-Language Support**: Handles documents in various languages with appropriate naming
- **Format Preservation**: Maintains original file extensions and handles special characters safely

### 📂 Smart Organization

Beyond simple filename generation, File Fairy provides comprehensive organization:

- **Content-Based Categorization**: Groups files by actual content type and purpose
- **Intelligent Naming**: Generates descriptive, consistent filenames based on document content
- **Flexible Options**: Organize in-place or move to target directories
- **Batch Processing**: Efficiently handle large collections of files

### 🛡️ Safety & Control

File Fairy puts you in control with multiple safety layers:

- **Dry Run by Default**: Preview all changes before any files are touched
- **Interactive Mode**: Approve, reject, or edit each proposed change individually
- **Safety Confirmations**: Clear warnings and confirmations for destructive operations
- **Comprehensive Logging**: Detailed information about what was done and why

### ⚡ Performance & Efficiency

Designed for real-world use with large file collections:

- **Async Processing**: Non-blocking operations for responsive performance
- **Memory Efficient**: Smart caching and streaming to handle large files
- **Concurrent Operations**: Parallel processing where safe and beneficial
- **Resource Management**: Intelligent model loading and GPU utilization

## 🗂️ Supported File Formats

### Documents

- **PDF** - Adobe Portable Document Format
- **Microsoft Office** - DOCX (Word), PPTX (PowerPoint)
- **HWP** - Hancom Office (Korean word processor) with full formatting support
- **Plain Text** - TXT, Markdown (MD)

### Data Files

- **Microsoft Excel** - XLSX spreadsheets with multi-sheet support

### Extensible Design

The modular architecture makes it easy to add support for new formats. Each format has dedicated extraction logic that understands the specific structure and content organization of that file type.

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **GGUF Model** - Download a compatible language model (e.g., from Hugging Face)
- **Storage Space** - Models typically range from 1GB to 100GB+

### Installation Options

#### Option 1: From Source (Recommended)

```bash
git clone https://github.com/ben-park2001/file-fairy.git
cd file-fairy
cargo build --release
```

#### Option 2: Individual Crates

```bash
# Just the CLI tool
cargo install --path file-fairy-cli

# Or use as a library in your project
# Add to Cargo.toml:
# file-fairy-core = { path = "path/to/file-fairy/file-fairy-core" }
```

### Model Setup

File Fairy uses local GGUF models for privacy and performance:

```bash
# Create models directory
mkdir models

# Download a model (example)
# You can get models from Hugging Face, e.g.:
# https://huggingface.co/microsoft/DialoGPT-medium-ggml
wget -O models/model.gguf "https://example.com/model.gguf"

# Update model path in your commands
file-fairy-cli organize ~/Documents --model models/model.gguf
```

## 📖 Usage Examples

### Basic Organization

```bash
# See what File Fairy would do (safe preview)
file-fairy-cli organize ~/Downloads

# Apply changes after reviewing
file-fairy-cli organize ~/Downloads --apply
```

### Advanced Organization

```bash
# Recursive organization with size limits
file-fairy-cli organize ~/Documents \
  --recursive \
  --max-size 50 \
  --max-depth 3 \
  --apply

# Interactive mode for fine control
file-fairy-cli organize ~/Pictures \
  --interactive \
  --apply
```

### Directory Analysis

```bash
# Get comprehensive directory statistics
file-fairy-cli scan ~/Downloads --recursive

# Quick overview with size limits
file-fairy-cli scan ~/Documents --max-size 100
```

### AI-Powered Suggestions

```bash
# Get filename suggestion for a single file
file-fairy-cli suggest important_document.pdf

# Use specific model and settings
file-fairy-cli suggest report.docx \
  --model models/large-model.gguf \
  --max-tokens 64
```

### System Information

```bash
# Check what formats are supported
file-fairy-cli info formats

# View system capabilities
file-fairy-cli info system

# Complete overview
file-fairy-cli info summary
```

## 🔧 Configuration

### Command-Line Configuration

Most settings can be configured via command-line arguments:

```bash
file-fairy-cli organize ~/Documents \
  --model models/model.gguf \
  --max-tokens 128 \
  --threads 8 \
  --recursive \
  --max-depth 5 \
  --max-size 100 \
  --apply
```

### Library Integration

For programmatic use, File Fairy provides rich configuration APIs:

```rust
use file_fairy_core::{FileOrganizeService, OrganizeOptions, AppConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::new()
        .with_model_path("models/model.gguf")
        .with_max_tokens(64)
        .with_threads(8);

    let service = FileOrganizeService::new(config)?;

    let options = OrganizeOptions::new()
        .with_recursive(true)
        .with_apply(false); // Dry run

    let results = service.organize(Path::new("~/Documents"), options).await?;
    println!("{}", results.format_preview());

    Ok(())
}
```

## 🔒 Privacy & Security

### Local-First Architecture

File Fairy is designed with privacy as a core principle:

- **No Network Calls**: All processing happens locally on your machine
- **No Data Collection**: Your files never leave your system
- **Local Models**: Uses local GGUF models - no cloud APIs required
- **Transparent Processing**: Open source - you can see exactly what it does

### Safety Features

Multiple layers of protection prevent accidental data loss:

- **Dry Run Default**: All operations preview-only by default
- **Interactive Confirmation**: Step-by-step approval for each change
- **Safety Warnings**: Clear confirmations for destructive operations
- **Comprehensive Logging**: Detailed logs of all operations
- **Atomic Operations**: File operations are atomic where possible

## 📄 License

File Fairy is open source software licensed under the MIT License. See [LICENSE](LICENSE) for the full license text.

## 🙏 Acknowledgments

File Fairy builds on the excellent work of many open source projects:

- **[llama.cpp](https://github.com/ggerganov/llama.cpp)** - High-performance LLM inference
- **[Tokio](https://tokio.rs/)** - Async runtime for Rust
- **[Clap](https://clap.rs/)** - Command-line argument parsing
- **[Serde](https://serde.rs/)** - Serialization framework
- **[pdf-extract](https://crates.io/crates/pdf-extract)** - PDF text extraction
- **[hwp](https://crates.io/crates/hwp)** - HWP file format support
- **[dotext](https://crates.io/crates/dotext)** - Office document extraction

Special thanks to the broader Rust community for creating such an excellent ecosystem for building reliable, performant tools.

**Built with ❤️ in Rust for intelligent, privacy-focused file organization.**
