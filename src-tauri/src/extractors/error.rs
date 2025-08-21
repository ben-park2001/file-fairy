use std::path::PathBuf;
use thiserror::Error;

/// A specific error type for the `file-fairy-extractors` crate.
///
/// This enum covers potential issues like I/O errors, parsing failures,
/// or problems with external script execution (like the HWP extractor).
#[derive(Error, Debug)]
pub enum ExtractorError {
    #[error("File I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("File format error for '{path}': {details}")]
    FormatError { path: PathBuf, details: String },
}

impl ExtractorError {
    /// Convenience constructor for format errors
    pub fn format_error(path: PathBuf, details: impl Into<String>) -> Self {
        Self::FormatError {
            path,
            details: details.into(),
        }
    }
}
