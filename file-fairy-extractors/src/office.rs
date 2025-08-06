use crate::error::ExtractorError;
use crate::traits::Extractor;
use async_trait::async_trait;
use dotext::{Docx, MsDoc, Pptx, Xlsx};
use std::io::Read;
use std::path::Path;

// --- DOCX Extractor ---

/// An extractor for .docx (Microsoft Word) files.
///
/// Reads text from paragraphs and tables within the document.
#[derive(Debug, Default)]
pub struct DocxExtractor;

impl DocxExtractor {
    /// Creates a new DOCX extractor
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Extractor for DocxExtractor {
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        let mut file = Docx::open(path).map_err(|e| ExtractorError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let mut text = String::new();
        file.read_to_string(&mut text)
            .map_err(|e| ExtractorError::Io {
                path: path.to_path_buf(),
                source: e,
            })?;

        Ok(text)
    }
}

// --- XLSX Extractor ---

/// An extractor for .xlsx (Microsoft Excel) files.
///
/// Reads data from all cells in all worksheets and combines them into text.
#[derive(Debug, Default)]
pub struct XlsxExtractor;

impl XlsxExtractor {
    /// Creates a new XLSX extractor
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Extractor for XlsxExtractor {
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        let mut file = Xlsx::open(path).map_err(|e| ExtractorError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let mut text = String::new();
        file.read_to_string(&mut text)
            .map_err(|e| ExtractorError::Io {
                path: path.to_path_buf(),
                source: e,
            })?;

        Ok(text)
    }
}

// --- PPTX Extractor ---

/// An extractor for .pptx (Microsoft PowerPoint) files.
///
/// Works by unzipping the .pptx archive and parsing the XML of each slide.
#[derive(Debug, Default)]
pub struct PptxExtractor;

impl PptxExtractor {
    /// Creates a new PPTX extractor
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Extractor for PptxExtractor {
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        let mut file = Pptx::open(path).map_err(|e| ExtractorError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        let mut text = String::new();
        file.read_to_string(&mut text)
            .map_err(|e| ExtractorError::Io {
                path: path.to_path_buf(),
                source: e,
            })?;

        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_docx_extractor() {
        let extractor = DocxExtractor;
        let path = PathBuf::from("tests/test_files/sample.docx");
        let result = extractor.extract(&path).await;
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("Extracted text from DOCX: {}", text);
        assert!(!text.is_empty(), "Extracted text should not be empty");
    }
}
