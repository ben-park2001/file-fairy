# File Fairy Extractors

A Rust library for extracting text content from various file formats. This crate provides a modular, trait-based architecture that makes it easy to add support for new file types while maintaining a consistent interface.

## 🚀 Features

- **Multi-format Support**: Extract text from PDF, Microsoft Office documents (DOCX, XLSX, PPTX), HWP files, and plain text
- **Async/Await Support**: All extractors are fully async for non-blocking I/O operations
- **Trait-based Architecture**: Extensible design allows easy addition of new file format extractors
- **Korean HWP Support**: Full support for Hancom Office HWP documents with complex content extraction
- **Error Handling**: Comprehensive error types with detailed context information
- **Memory Efficient**: Streaming-based extraction where possible to handle large files

## 📦 Supported File Formats

| Format                   | Extensions    | Description                        | Status       |
| ------------------------ | ------------- | ---------------------------------- | ------------ |
| **PDF**                  | `.pdf`        | Adobe Portable Document Format     | ✅ Supported |
| **Microsoft Word**       | `.docx`       | Microsoft Word documents           | ✅ Supported |
| **Microsoft Excel**      | `.xlsx`       | Microsoft Excel spreadsheets       | ✅ Supported |
| **Microsoft PowerPoint** | `.pptx`       | Microsoft PowerPoint presentations | ✅ Supported |
| **HWP**                  | `.hwp`        | Hancom Office documents (Korean)   | ✅ Supported |
| **Plain Text**           | `.txt`, `.md` | Text and Markdown files            | ✅ Supported |

## 🏗️ Architecture

The library is built around the `Extractor` trait, which provides a consistent interface for all file format extractors:

```rust
#[async_trait]
pub trait Extractor: Send + Sync {
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError>;
}
```

### Key Components

- **`traits::Extractor`** - Core trait defining the extraction interface
- **`error::ExtractorError`** - Comprehensive error handling with detailed context
- **Format-specific extractors** - Individual modules for each supported format
- **Factory function** - Automatic extractor selection based on file extension

## 🎯 Quick Start

### Basic Usage

```rust
use file_fairy_extractors::{extractor_from_file_path, Extractor};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Path::new("document.pdf");

    if let Some(extractor) = extractor_from_file_path(file_path) {
        match extractor.extract(file_path).await {
            Ok(text) => println!("Extracted text: {}", text),
            Err(e) => println!("Extraction failed: {}", e),
        }
    } else {
        println!("Unsupported file format");
    }

    Ok(())
}
```

### Direct Extractor Usage

```rust
use file_fairy_extractors::pdf::PdfExtractor;
use file_fairy_extractors::Extractor;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let extractor = PdfExtractor::new();
    let text = extractor.extract(Path::new("document.pdf")).await?;
    println!("PDF content: {}", text);
    Ok(())
}
```

### Batch Processing

```rust
use file_fairy_extractors::{extractor_from_file_path, ExtractorError};
use std::path::Path;
use tokio::fs;

async fn extract_from_directory(dir_path: &Path) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let mut entries = fs::read_dir(dir_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if let Some(extractor) = extractor_from_file_path(&path) {
            match extractor.extract(&path).await {
                Ok(text) => {
                    results.push((path.display().to_string(), text));
                }
                Err(e) => {
                    eprintln!("Failed to extract from {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(results)
}
```

## 📋 Detailed Extractor Information

### PDF Extractor (`pdf::PdfExtractor`)

Extracts text content from PDF documents using the `pdf-extract` crate.

**Features:**

- Handles text-based PDFs
- Preserves basic text structure
- Memory efficient for large documents

**Limitations:**

- Does not extract text from images or scanned PDFs
- Complex layouts may have formatting issues

### Microsoft Office Extractors

#### DOCX Extractor (`office::DocxExtractor`)

- Extracts text from paragraphs and tables
- Handles formatted text content
- Supports modern .docx format only

#### XLSX Extractor (`office::XlsxExtractor`)

- Extracts data from all worksheets
- Combines cell values into readable text
- Handles formulas by extracting calculated values

#### PPTX Extractor (`office::PptxExtractor`)

- Extracts text from all slides
- Includes slide titles and content
- Handles text boxes and shapes

### HWP Extractor (`hwp::HwpExtractor`)

Advanced extractor for Korean HWP (Hancom Office) documents with comprehensive content extraction.

**Features:**

- **Complete Document Structure**: Extracts from paragraphs, tables, headers, footers
- **Complex Content Support**: Handles equations, pictures, OLE objects, containers
- **Shape Object Extraction**: Extracts text from various shape types (rectangles, ellipses, polygons, etc.)
- **Nested Content Handling**: Recursively processes nested paragraph lists and containers
- **Caption Support**: Extracts captions from tables, figures, and other objects
- **Footnote/Endnote Support**: Includes footnotes and endnotes in extraction

**Technical Details:**

- Uses the `hwp` crate for binary format parsing
- Handles both distributed and non-distributed document layouts
- Recursive extraction ensures no content is missed
- Comprehensive control type support including tables, shapes, and embedded objects

### Text Extractor (`text::TextExtractor`)

Simple extractor for plain text files.

**Supported formats:**

- `.txt` - Plain text files
- `.md` - Markdown files
- Any UTF-8 encoded text file

## 🛠️ Error Handling

The library provides comprehensive error handling through the `ExtractorError` enum:

```rust
pub enum ExtractorError {
    Io { path: PathBuf, source: std::io::Error },
    Utf8Decoding { path: PathBuf },
    DependencyUnavailable { feature_name: String, details: String },
    FormatError { path: PathBuf, details: String },
    UnsupportedFormat { path: PathBuf },
}
```

### Error Types

- **`Io`** - File system errors (file not found, permission denied, etc.)
- **`Utf8Decoding`** - Text encoding issues
- **`DependencyUnavailable`** - Missing required dependencies
- **`FormatError`** - File format parsing errors
- **`UnsupportedFormat`** - Unsupported file types

### Error Handling Example

```rust
use file_fairy_extractors::{extractor_from_file_path, ExtractorError};

match extractor.extract(path).await {
    Ok(text) => println!("Success: {}", text),
    Err(ExtractorError::Io { path, source }) => {
        eprintln!("IO error for {}: {}", path.display(), source);
    }
    Err(ExtractorError::FormatError { path, details }) => {
        eprintln!("Format error in {}: {}", path.display(), details);
    }
    Err(ExtractorError::UnsupportedFormat { path }) => {
        eprintln!("Unsupported format: {}", path.display());
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## 🧪 Testing

The library includes comprehensive tests with sample files for each supported format.

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific extractor
cargo test pdf
cargo test hwp
cargo test office

# Run with output
cargo test -- --nocapture
```

### Test Files

Sample files are should be placed in `tests/test_files/` for testing:

- `sample.pdf` - PDF test document
- `sample.docx` - Word document test file
- `sample.hwp` - HWP test document

## 🔧 Dependencies

The library uses the following key dependencies:

- **`pdf-extract`** - PDF text extraction
- **`hwp`** - HWP file format parsing
- **`dotext`** - Microsoft Office document extraction
- **`tokio`** - Async runtime
- **`async-trait`** - Async trait support
- **`thiserror`** - Error handling

## 🚀 Performance Considerations

### Memory Usage

- Extractors are designed to be memory efficient
- Small objects are preferred over large buffer allocations
- Streaming extraction is used where possible

### Async Benefits

- Non-blocking I/O prevents thread blocking
- Suitable for concurrent extraction of multiple files
- Integrates well with async web servers and applications

### File Size Limits

- No hard limits imposed by the library
- Large files are handled efficiently
- Consider implementing application-level size limits for production use

## 🔌 Extending the Library

### Adding a New Extractor

1. **Create the extractor module**:

```rust
// src/my_format.rs
use crate::{error::ExtractorError, traits::Extractor};
use async_trait::async_trait;
use std::path::Path;

#[derive(Debug, Default)]
pub struct MyFormatExtractor;

impl MyFormatExtractor {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Extractor for MyFormatExtractor {
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        // Implementation here
        todo!()
    }
}
```

2. **Add to the factory function**:

```rust
// src/lib.rs
pub fn extractor_from_file_path(file_path: &Path) -> Option<Box<dyn Extractor>> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        // ... existing formats
        Some("myext") => Some(Box::new(my_format::MyFormatExtractor::new())),
        _ => None,
    }
}
```

3. **Add comprehensive tests** with sample files

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Guidelines

- Follow Rust naming conventions
- Add comprehensive tests for new extractors
- Include sample files for testing (where legally permissible)
- Update documentation and README
- Ensure async compatibility

### Areas for Contribution

- **New file formats**: Add support for additional document types
- **Performance improvements**: Optimize extraction algorithms
- **Error handling**: Improve error messages and recovery
- **Testing**: Add more comprehensive test cases
- **Documentation**: Improve examples and guides

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- **`pdf-extract`** - Robust PDF text extraction
- **`hwp`** - Comprehensive HWP format support
- **`dotext`** - Microsoft Office document parsing
- **Tokio ecosystem** - Async runtime and utilities

---

For more information about the File Fairy ecosystem, visit the main repository or check out the other crates:

- **file-fairy-cli** - Command-line interface
- **file-fairy-core** - Core organization logic
- **file-fairy-llm** - Local LLM integration
