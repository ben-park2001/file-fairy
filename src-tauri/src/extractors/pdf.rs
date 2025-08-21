use super::error::ExtractorError;
use super::traits::Extractor;
use async_trait::async_trait;
use std::path::Path;

/// An extractor for PDF files.
///
/// Uses the `pdf-extract` crate to extract text content from PDF documents.
#[derive(Debug, Default)]
pub struct PdfExtractor;

impl PdfExtractor {
    /// Creates a new PDF extractor
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Extractor for PdfExtractor {
    /// Extracts text from a PDF file.
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| ExtractorError::Io {
                path: path.to_path_buf(),
                source: e,
            })?;

        pdf_extract::extract_text_from_mem(&bytes).map_err(|e| {
            ExtractorError::format_error(
                path.to_path_buf(),
                format!("PDF extraction failed: {}", e),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn instantiate_pdf() {
        let extractor = PdfExtractor;
        let path = PathBuf::from("tests/test_files/sample.pdf");

        // Ensure the extractor can be instantiated and used
        let result = extractor.extract(&path).await;

        assert!(result.is_ok());
        let text = result.unwrap();
        println!("Extracted text: {}", text);
        assert!(!text.is_empty(), "Extracted text should not be empty");
    }
}
