//! Services module
//!
//! Provides high-level business logic services for file operations,
//! content extraction, and AI-powered file organization.

pub mod extraction;
pub mod filesystem;
pub mod organization;
pub mod watch;

pub use extraction::ExtractionService;
pub use filesystem::FileSystemService;
pub use organization::OrganizationService;
pub use watch::WatchService;
