# File Fairy 🧚

![logo](./logo.png)
File Fairy is an AI-powered file organization tool that intelligently categorizes, renames, and organizes files based on their content and metadata.

## Features

- **Multi-format Support**: Process documents (PDF, DOCX, PPTX, HWP), images, audio files, and data files
- **AI-Powered Organization**: Intelligent file naming and categorization using AI analysis
- **Clean Architecture**: Well-organized codebase following Python best practices
- **Command-Line Interface**: Easy-to-use CLI for file organization tasks
- **Extensible Design**: Modular architecture for easy extension and customization

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd file-fairy

# Install dependencies
pip install -e .
```

## Quick Start

### Basic Usage

```python
from file_fairy import create_message_from_file, is_supported_file

# Check if a file is supported
if is_supported_file("document.pdf"):
    # Extract content from file
    message = create_message_from_file("document.pdf")
    print(message)
```

### Configuration

```python
from file_fairy.core.config import AppConfig, FileCategory

# Load configuration (supports environment variables)
config = AppConfig.load()

# Check supported file types
print('PDF category:', FileCategory.get_category_for_extension('.pdf'))
print('All extensions:', FileCategory.get_all_extensions())
```

### File Organization

```bash
# Organize files in a directory (without AI)
python -m file_fairy.cli organize /path/to/directory --no-ai

# Preview organization (shows what would be done)
python -m file_fairy.cli organize /path/to/directory --preview

# Scan directory contents
python -m file_fairy.cli scan /path/to/directory

# Show supported formats and info
python -m file_fairy.cli info
```

### Advanced Usage with AI

```python
from file_fairy.core import FileOrganizer

# Create organizer with AI capabilities
organizer = FileOrganizer(use_ai=True, model_path="path/to/ai/model")

# Organize directory
results = organizer.organize_directory(
    source_dir=Path("source"),
    target_dir=Path("organized"),
    dry_run=False
)

print(f"Processed {results['processed_files']} files")
```

## Architecture

The codebase follows a clean, modular architecture:

```
file_fairy/
├── __init__.py           # Main package exports and convenience functions
├── cli.py               # Command-line interface
├── constants.py         # Application constants and enums
├── message_creator.py   # File content extraction and message creation
├── utils.py            # General utility functions
├── core/               # Core business logic
│   ├── __init__.py
│   ├── ai_processor.py # AI model integration
│   ├── config.py       # Configuration management
│   ├── file_utils.py   # File system operations
│   └── organizer.py    # Main file organization logic
└── extractors/         # File content extractors
    ├── __init__.py     # Extractor registry
    ├── base.py         # Base extractor interface
    ├── data_extractors.py
    ├── hwp_extractor.py
    ├── office_extractors.py
    ├── pdf_extractor.py
    └── text_extractor.py
```

### Key Components

#### Core Modules

- **`FileOrganizer`**: Main orchestrator for file organization tasks
- **`AppConfig`**: Centralized configuration management with environment variable support
- **`FileCategory`**: Enum-based file type categorization (single source of truth)
- **`FileUtils`**: File system operations and utilities
- **`AIProcessor`**: AI model integration for content analysis

#### Extractors

- **`ExtractorRegistry`**: Factory for content extractors
- **`BaseExtractor`**: Abstract base class for extractors
- **Specific Extractors**: PDF, Office, HWP, text, and data file extractors

#### Message Creation

- **`InputMessageCreator`**: Creates structured messages from file content

## Configuration

### File Categories

Files are automatically categorized into:

- **문서 (Documents)**: PDF, DOCX, PPTX, HWP, TXT, MD
- **이미지 (Images)**: JPG, PNG, GIF, BMP, TIFF
- **데이터 (Data)**: CSV, XLSX, JSON, XLS
- **음악 (Audio)**: MP3, WAV, FLAC, M4A, AAC, OGG

### AI Model Configuration

Set the AI model path in the configuration:

```python
from file_fairy.core.config import AppConfig, AIConfig

# Custom AI configuration
ai_config = AIConfig(model_path="custom-model-path")
config = AppConfig(ai=ai_config)

# Or use environment variables
# FILE_FAIRY_MODEL_PATH=custom-model-path
config = AppConfig.load()  # Will pick up environment variables
```

### Environment Variables

You can configure File Fairy using environment variables:

```bash
export FILE_FAIRY_MODEL_PATH="/path/to/ai/model"
export FILE_FAIRY_LOG_FILE="production.log"
export FILE_FAIRY_MAX_TOKENS="128"
export FILE_FAIRY_ENABLE_VISION="true"
export FILE_FAIRY_ENABLE_AUDIO="false"
```

## CLI Commands

### Organize Files

```bash
# Basic organization
python -m file_fairy.cli organize /path/to/files

# With specific output directory
python -m file_fairy.cli organize /path/to/files -o /path/to/organized

# Preview mode (shows what would be done)
python -m file_fairy.cli organize /path/to/files --preview

# Dry run (shows operations without executing)
python -m file_fairy.cli organize /path/to/files --dry-run

# Disable AI features
python -m file_fairy.cli organize /path/to/files --no-ai
```

### Scan Directory

```bash
# Basic scan
python -m file_fairy.cli scan /path/to/directory

# Include subdirectories
python -m file_fairy.cli scan /path/to/directory --recursive

# Show only file extensions
python -m file_fairy.cli scan /path/to/directory --by-ext

# Show only file dates
python -m file_fairy.cli scan /path/to/directory --by-date
```

### Show Information

```bash
# Show all information
python -m file_fairy.cli info

# Show only supported formats
python -m file_fairy.cli info --supported-formats

# Show only file categories
python -m file_fairy.cli info --categories

# Show model path information
python -m file_fairy.cli info --model-path
```

## Development

### Code Style

The codebase follows these principles:

- **Clean Code**: Self-documenting code with clear naming
- **SOLID Principles**: Single responsibility, dependency injection, etc.
- **Type Hints**: Comprehensive type annotations
- **Error Handling**: Graceful error handling with informative messages
- **Modularity**: Separated concerns with clear interfaces

### Adding New Extractors

To add support for a new file type:

1. Create a new extractor class inheriting from `BaseExtractor`:

```python
from .base import BaseExtractor
from pathlib import Path

class MyExtractor(BaseExtractor):
    def extract(self, file_path: Path) -> str:
        # Implementation here
        pass
```

2. Register it in `extractors/__init__.py`:

```python
from .my_extractor import MyExtractor

class ExtractorRegistry:
    _extractors = {
        # ... existing extractors
        '.myext': MyExtractor,
    }
```

3. Add the file extension to `constants.py` if needed.

### Testing

```bash
# Test basic imports
python -c "import file_fairy; print('Import successful')"

# Test CLI
python -m file_fairy.cli --help

# Test specific functionality
python -c "
from file_fairy import is_supported_file
print('PDF supported:', is_supported_file('test.pdf'))
"
```

## Contributing

1. Follow the existing code style and architecture
2. Add type hints for all functions and methods
3. Include docstrings for all public APIs
4. Test your changes thoroughly
5. Update documentation as needed

## License

[Add your license information here]
