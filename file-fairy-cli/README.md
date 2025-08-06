# File Fairy CLI

A powerful command-line interface for intelligent file organization and management. File Fairy CLI is part of the File Fairy ecosystem, providing automated file organization capabilities powered by AI and advanced file analysis.

## 🚀 Features

- **Smart File Organization**: Automatically categorize and organize files based on type, content, and metadata
- **Directory Scanning**: Analyze directories to understand file distribution and statistics
- **Dry Run Mode**: Preview all changes before applying them to ensure safety
- **Interactive Mode**: Step-by-step approval of proposed file operations
- **AI-Powered Suggestions**: Generate intelligent filename suggestions using local LLM models
- **Comprehensive Info System**: View supported file formats, system capabilities, and configuration
- **Safety First**: Built-in warnings and confirmations for destructive operations

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/ben-park2001/file-fairy.git
cd file-fairy

# Build the CLI
cargo build --release -p file-fairy-cli

# Binary will be available at target/release/file-fairy-cli
```

### Using Cargo

```bash
cargo install --path file-fairy-cli
```

## 🎯 Quick Start

### Basic Directory Organization (Dry Run)

```bash
# Preview organization of current directory
file-fairy organize .

# Preview with recursive scanning
file-fairy organize /path/to/directory --recursive
```

### Apply Changes

```bash
# Actually perform the organization (with safety warning)
file-fairy organize /path/to/directory --apply

# Interactive mode - approve each change
file-fairy organize /path/to/directory --apply --interactive
```

### Directory Analysis

```bash
# Scan directory for statistics
file-fairy scan /path/to/directory

# Recursive scan with size limits
file-fairy scan /path/to/directory --recursive --max-size 100
```

## 📋 Commands

### `organize` - File Organization

Intelligently organize files into categorized folders.

```bash
file-fairy organize [PATH] [OPTIONS]
```

**Options:**

- `--apply` - Actually perform file operations (default is dry-run)
- `--interactive, -i` - Prompt for each file with proposed changes
- `--recursive, -r` - Scan subdirectories recursively
- `--max-depth DEPTH` - Maximum depth for recursive scanning
- `--follow-symlinks` - Follow symbolic links
- `--max-size SIZE_MB` - Maximum file size to process (in MB)
- `--target-dir DIR` - Target directory for organized files

**Examples:**

```bash
# Preview organization
file-fairy organize ~/Downloads

# Apply changes with confirmation
file-fairy organize ~/Downloads --apply

# Interactive organization
file-fairy organize ~/Documents --apply --interactive --recursive
```

### `scan` - Directory Analysis

Analyze directories without making any changes.

```bash
file-fairy scan [PATH] [OPTIONS]
```

**Options:**

- `--recursive, -r` - Scan subdirectories recursively
- `--max-depth DEPTH` - Maximum depth for recursive scanning
- `--follow-symlinks` - Follow symbolic links
- `--max-size SIZE_MB` - Maximum file size to include (in MB)

**Example:**

```bash
file-fairy scan ~/Downloads --recursive --max-depth 3
```

### `info` - System Information

Display information about File Fairy's capabilities and configuration.

```bash
file-fairy info [SUBCOMMAND]
```

**Subcommands:**

- `summary` - Show comprehensive information (default)
- `formats` - Show supported file formats
- `system` - Show system information

**Examples:**

```bash
file-fairy info
file-fairy info formats
file-fairy info system
```

### `suggest` - AI-Powered Filename Suggestions

Generate intelligent filename suggestions using local LLM models.

```bash
file-fairy suggest FILE [OPTIONS]
```

**Options:**

- `--model, -m MODEL_PATH` - Path to LLM model file
- `--max-tokens TOKENS` - Maximum tokens to generate (default: 32)
- `--threads, -t THREADS` - Number of threads to use (default: 8)

**Example:**

```bash
file-fairy suggest document.pdf --model models/gemma-3n-E2B-it-Q4_K_M.gguf
```

### `registry` - Registry Management

View and manage the registry of supported file types and extractors.

```bash
file-fairy registry [SUBCOMMAND]
```

## 🔧 Configuration

### Command Line Options

Most configuration is done via command-line arguments. Use `--help` with any command for detailed options.

### Global Options

- `--verbose, -v` - Enable verbose output
- `--config CONFIG_FILE` - Use custom configuration file

## 📊 Output Format

### Organization Preview

File Fairy shows a detailed table preview of all proposed changes:

```
🔍 ORGANIZATION PREVIEW (DRY RUN)
═══════════════════════════════════

📁 Path: /Users/example/Downloads
🕒 Time: 2025-01-15 10:30:45 UTC
📊 Total Files: 25
✅ Approved: 23

📋 ACTIONS
┌──┬─────────────────────────────┬──────────┬─────────────────────────────┬──────────┐
│  │ Original                    │   Size   │ Suggested                   │ Category │
├──┼─────────────────────────────┼──────────┼─────────────────────────────┼──────────┤
│✓ │ document.pdf               │ 2.5 MB   │ Documents/document.pdf      │ document │
│✓ │ photo.jpg                  │ 1.2 MB   │ Images/photo.jpg            │ image    │
│✓ │ song.mp3                   │ 4.1 MB   │ Audio/song.mp3              │ audio    │
└──┴─────────────────────────────┴──────────┴─────────────────────────────┴──────────┘

💡 NEXT STEPS
• Review the suggested changes above
• Run with --apply to execute the approved actions
• Use --interactive for step-by-step control
```

### Scan Results

Directory scan provides comprehensive statistics:

```
📊 DIRECTORY SCAN RESULTS
═══════════════════════════

📁 Path: /Users/example/Downloads
🕒 Scanned: 2025-01-15 10:30:45 UTC
📈 Total Files: 156 files in 12 directories
💾 Total Size: 2.3 GB

📋 BY CATEGORY
┌─────────────┬───────┬──────────┬─────────────┐
│ Category    │ Count │ Size     │ Percentage  │
├─────────────┼───────┼──────────┼─────────────┤
│ Documents   │    45 │ 892.5 MB │      38.8%  │
│ Images      │    67 │ 1.2 GB   │      51.3%  │
│ Audio       │    23 │ 156.7 MB │       6.8%  │
│ Archives    │    21 │  67.8 MB │       2.9%  │
└─────────────┴───────┴──────────┴─────────────┘
```

## 🛡️ Safety Features

### Dry Run by Default

All organization commands run in preview mode by default. Use `--apply` to actually perform file operations.

### Safety Warnings

When using `--apply`, File Fairy shows a warning and requires confirmation:

```
⚠️  WARNING: --apply flag detected. This will actually move and rename files!
   Make sure you have backups and are in the correct directory.
   Continue? [y/N]:
```

### Interactive Mode

Use `--interactive` to approve each file operation individually.

## 🏗️ Architecture

File Fairy CLI is built on a modular architecture:

- **file-fairy-cli** - Command-line interface (this crate)
- **file-fairy-core** - Core organization and scanning logic
- **file-fairy-extractors** - File metadata extraction
- **file-fairy-llm** - Local LLM integration

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/ben-park2001/file-fairy.git
cd file-fairy

# Run tests
cargo test

# Run the CLI in development
cargo run -p file-fairy-cli -- --help
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Clap](https://github.com/clap-rs/clap) for command-line parsing
- Powered by [Tokio](https://tokio.rs/) for async runtime
- File analysis by custom extractors supporting various formats

---

For more information about the File Fairy ecosystem, visit the main repository or check out the individual crate documentation.
