//! Information system for File Fairy capabilities and system details.
//!
//! This module provides comprehensive information about File Fairy's:
//! - Version and build details
//! - Supported file formats and categories
//! - Available features and capabilities
//! - System and runtime information

mod features;
mod formats;
mod formatter;
mod models;
mod system;
mod version;

// Public API
pub use features::FeatureInfo;
pub use formats::{CategoryInfo, SupportedFormats};
pub use formatter::InfoFormatter;
pub use models::FileFairyInfo;
pub use system::SystemInfo;
pub use version::VersionInfo;
