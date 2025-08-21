//! # File Fairy Extractors Library
//!
//! This crate provides a modular system for extracting text content from
//! various file types. It uses a trait-based design to allow for easy
//! extension with new file formats.

// Declare the modules
pub mod error;
pub mod hwp;
pub mod office;
pub mod pdf;
pub mod text;
pub mod traits;

use std::path::Path;

// Re-export key types
pub use error::ExtractorError;
pub use traits::Extractor;

// Enum to hold all extractor types
pub enum ExtractorType {
    Text(text::TextExtractor),
    Pdf(pdf::PdfExtractor),
    Hwp(hwp::HwpExtractor),
    Docx(office::DocxExtractor),
    Xlsx(office::XlsxExtractor),
    Pptx(office::PptxExtractor),
}

impl ExtractorType {
    pub async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        match self {
            ExtractorType::Text(extractor) => extractor.extract(path).await,
            ExtractorType::Pdf(extractor) => extractor.extract(path).await,
            ExtractorType::Hwp(extractor) => extractor.extract(path).await,
            ExtractorType::Docx(extractor) => extractor.extract(path).await,
            ExtractorType::Xlsx(extractor) => extractor.extract(path).await,
            ExtractorType::Pptx(extractor) => extractor.extract(path).await,
        }
    }
}

/// Factory function to create an extractor based on file extension.
///
/// # Arguments
/// * `file_path` - A path reference to determine the extractor type
///
/// # Returns
/// An optional extractor matching the file extension
pub fn extractor_from_file_path(file_path: &Path) -> Option<ExtractorType> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("txt" | "md") => Some(ExtractorType::Text(text::TextExtractor::new())),
        Some("pdf") => Some(ExtractorType::Pdf(pdf::PdfExtractor::new())),
        Some("hwp") => Some(ExtractorType::Hwp(hwp::HwpExtractor::new())),
        Some("docx") => Some(ExtractorType::Docx(office::DocxExtractor::new())),
        Some("xlsx") => Some(ExtractorType::Xlsx(office::XlsxExtractor::new())),
        Some("pptx") => Some(ExtractorType::Pptx(office::PptxExtractor::new())),
        _ => None,
    }
}
