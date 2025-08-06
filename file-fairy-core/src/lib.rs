//! # File Fairy Core Library
//!
//! This crate contains the core business logic for the File Fairy application.
//! It handles configuration, file system operations, and the main organization
//! orchestration, but remains decoupled from the command-line interface.

// Declare the modules that make up this crate.
// The content of each module is in a file with the same name.
pub mod config;
pub mod error;
pub mod file_category;
pub mod info;
pub mod organize;
pub mod scan;

// Export the most important public types for consumers of this library.
// This allows a consumer to just `use file_fairy_core::FileOrganizeService;`
// instead of `use file_fairy_core::organize::FileOrganizeService;`.
pub use error::CoreError;
pub use file_category::FileCategory;
pub use info::FileFairyInfo;
pub use organize::{FileOrganizeService, OrganizeOptions, OrganizeResults};
pub use scan::{DirectoryScanner, ScanOptions, ScanResults};
