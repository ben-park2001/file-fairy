use serde::{Deserialize, Serialize};
use std::path::Path;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

/// Represents the primary categories for file organization.
///
/// Using an enum ensures type safety and prevents the use of invalid category strings.
/// Various derive macros provide useful functionality:
/// - `Serialize`/`Deserialize`: JSON/config file support
/// - `AsRefStr`: Get string representation easily
/// - `Display`: String formatting
/// - `EnumIter`: Iterate over all variants
/// - `EnumString`: Parse from strings
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum FileCategory {
    Documents,
    // Images,
    Data,
    // Audio,
    Unsupported,
}

impl FileCategory {
    /// Converts a file extension to its corresponding `FileCategory`.
    ///
    /// # Arguments
    /// * `file_path` - The path to analyze
    ///
    /// # Returns
    /// The detected file category based on extension
    pub fn from_path(file_path: &Path) -> Self {
        match file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            // Document formats
            Some("txt" | "md" | "pdf" | "docx" | "pptx" | "hwp") => Self::Documents,
            // Data formats
            Some("xlsx") => Self::Data,
            _ => Self::Unsupported,
        }
    }

    /// Returns whether this category represents a supported file type for extraction
    pub fn is_extractable(&self) -> bool {
        matches!(self, Self::Documents | Self::Data)
    }

    /// Returns a human-readable description of the category
    pub fn description(&self) -> &'static str {
        match self {
            Self::Documents => "Document files",
            // Self::Images => "Image files",
            Self::Data => "Structured data files",
            // Self::Audio => "Audio files",
            Self::Unsupported => "Unsupported files",
        }
    }
}
