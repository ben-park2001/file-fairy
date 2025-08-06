use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about supported file formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedFormats {
    /// File categories and their details
    pub categories: HashMap<String, CategoryInfo>,
    /// Total number of supported extensions
    pub total_extensions: usize,
    /// All supported file extensions
    pub extensions: Vec<String>,
}

/// Information about a specific file category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    /// Human-readable description
    pub description: String,
    /// File extensions in this category
    pub extensions: Vec<String>,
    /// Whether files in this category can be processed for content extraction
    pub extractable: bool,
    /// Additional notes about this category
    pub notes: Option<String>,
}

impl SupportedFormats {
    /// Creates supported formats information
    pub fn new() -> Self {
        let mut categories = HashMap::new();
        let mut all_extensions = Vec::new();

        // Define file categories and their extensions
        let category_data = Self::get_category_data();

        for (name, info) in category_data {
            if name != "Unknown" {
                all_extensions.extend(info.extensions.clone());
            }
            categories.insert(name.to_string(), info);
        }

        all_extensions.sort();
        all_extensions.dedup();

        Self {
            categories,
            total_extensions: all_extensions.len(),
            extensions: all_extensions,
        }
    }

    /// Returns the predefined category data
    fn get_category_data() -> Vec<(&'static str, CategoryInfo)> {
        vec![
            (
                "Documents",
                CategoryInfo {
                    description: "Text documents and presentations".to_string(),
                    extensions: vec![
                        "txt".to_string(),
                        "md".to_string(),
                        "pdf".to_string(),
                        "docx".to_string(),
                        "pptx".to_string(),
                        "hwp".to_string(),
                    ],
                    extractable: true,
                    notes: Some("Supports text extraction for AI analysis".to_string()),
                },
            ),
            (
                "Data",
                CategoryInfo {
                    description: "Structured data files".to_string(),
                    extensions: vec!["xlsx".to_string()],
                    extractable: true,
                    notes: Some("Spreadsheet data can be extracted as text".to_string()),
                },
            ),
            (
                "Unknown",
                CategoryInfo {
                    description: "Unknown or unsupported file types".to_string(),
                    extensions: vec!["*".to_string()],
                    extractable: false,
                    notes: Some("Files not matching any known category".to_string()),
                },
            ),
        ]
    }

    /// Gets category information by name
    pub fn get_category(&self, name: &str) -> Option<&CategoryInfo> {
        self.categories.get(name)
    }

    /// Checks if an extension is supported
    pub fn is_extension_supported(&self, extension: &str) -> bool {
        self.extensions.contains(&extension.to_lowercase())
    }
}

impl Default for SupportedFormats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_formats_creation() {
        let formats = SupportedFormats::new();
        assert!(formats.total_extensions > 0);
        assert!(!formats.extensions.is_empty());
        assert!(formats.categories.contains_key("Documents"));
        assert!(formats.categories.contains_key("Data"));
    }

    #[test]
    fn test_extension_support_check() {
        let formats = SupportedFormats::new();
        assert!(formats.is_extension_supported("pdf"));
        assert!(formats.is_extension_supported("docx"));
        assert!(!formats.is_extension_supported("xyz"));
    }

    #[test]
    fn test_category_retrieval() {
        let formats = SupportedFormats::new();
        let docs = formats.get_category("Documents").unwrap();
        assert!(docs.extractable);
        assert!(docs.extensions.contains(&"pdf".to_string()));
    }

    #[test]
    fn test_unknown_category_not_in_extensions() {
        let formats = SupportedFormats::new();
        // The "*" from Unknown category should not be in extensions list
        assert!(!formats.extensions.contains(&"*".to_string()));
    }
}
