//! Directory scanning module
//!
//! This module provides functionality for scanning directories and analyzing files.
//! It's organized into several submodules for better separation of concerns:
//!
//! - `stats`: File category statistics and metrics
//! - `results`: Scan results and reporting
//! - `options`: Configuration options for scanning
//! - `scanner`: The main directory scanning implementation

pub mod options;
pub mod results;
pub mod scanner;
pub mod stats;

pub use options::ScanOptions;
pub use results::ScanResults;
pub use scanner::DirectoryScanner;
pub use stats::CategoryStats;
