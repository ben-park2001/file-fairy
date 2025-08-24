//! Services module
//!
//! Provides high-level business logic services for file operations,
//! content extraction, and AI-powered file organization.

pub mod database;
pub mod extraction;
pub mod filesystem;
pub mod ollama;
pub mod renamer;
pub mod watch;

pub use database::DatabaseService;
pub use extraction::ExtractionService;
pub use filesystem::FileSystemService;
pub use ollama::OllamaService;
pub use renamer::RenamerService;
pub use watch::WatchService;
