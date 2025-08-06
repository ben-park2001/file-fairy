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

/// Factory function to create an extractor based on file extension.
///
/// # Arguments
/// * `file_path` - A path reference to determine the extractor type
///
/// # Returns
/// An optional boxed extractor matching the file extension
pub fn extractor_from_file_path(file_path: &Path) -> Option<Box<dyn Extractor>> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("txt" | "md") => Some(Box::new(text::TextExtractor::new())),
        Some("pdf") => Some(Box::new(pdf::PdfExtractor::new())),
        Some("hwp") => Some(Box::new(hwp::HwpExtractor::new())),
        Some("docx") => Some(Box::new(office::DocxExtractor::new())),
        Some("xlsx") => Some(Box::new(office::XlsxExtractor::new())),
        Some("pptx") => Some(Box::new(office::PptxExtractor::new())),
        _ => None,
    }
}
