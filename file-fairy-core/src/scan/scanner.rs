use super::{options::ScanOptions, results::ScanResults};
use crate::{error::CoreError, file_category::FileCategory};
use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};
use tokio::fs;

/// The main directory scanner implementation
pub struct DirectoryScanner {
    options: ScanOptions,
}

impl DirectoryScanner {
    /// Creates a new scanner with default options
    pub fn new() -> Self {
        Self {
            options: ScanOptions::default(),
        }
    }

    /// Creates a new scanner with custom options
    pub fn with_options(options: ScanOptions) -> Self {
        Self { options }
    }

    /// Scans a directory and returns the results
    pub async fn scan(&self, path: &Path) -> Result<ScanResults, CoreError> {
        self.validate_scan_path(path)?;

        let mut results = ScanResults::new(path.to_path_buf());
        let mut scanner = FilesystemScanner::new(&self.options, &mut results);

        scanner.scan_directory(path).await?;

        Ok(results)
    }

    /// Validates that the scan path is suitable for scanning
    fn validate_scan_path(&self, path: &Path) -> Result<(), CoreError> {
        if !path.exists() {
            return Err(CoreError::path_not_found(path));
        }

        if !path.is_dir() {
            return Err(CoreError::generic(format!(
                "Path '{}' is not a directory",
                path.display()
            )));
        }

        Ok(())
    }
}

impl Default for DirectoryScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal filesystem scanner that handles the actual directory traversal
struct FilesystemScanner<'a> {
    options: &'a ScanOptions,
    results: &'a mut ScanResults,
}

impl<'a> FilesystemScanner<'a> {
    fn new(options: &'a ScanOptions, results: &'a mut ScanResults) -> Self {
        Self { options, results }
    }

    /// Scans a directory recursively using a stack-based approach
    async fn scan_directory(&mut self, root_path: &Path) -> Result<(), CoreError> {
        let mut stack = vec![(root_path.to_path_buf(), 0)];

        while let Some((current_path, depth)) = stack.pop() {
            if !self.options.should_process_depth(depth) {
                continue;
            }

            let entries = match fs::read_dir(&current_path).await {
                Ok(entries) => entries,
                Err(_) => {
                    self.results.add_error();
                    continue;
                }
            };

            self.process_directory_entries(entries, depth, &mut stack)
                .await;
        }

        Ok(())
    }

    /// Processes all entries in a directory
    async fn process_directory_entries(
        &mut self,
        mut entries: fs::ReadDir,
        depth: usize,
        stack: &mut Vec<(PathBuf, usize)>,
    ) {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let entry_path = entry.path();

            match self.get_entry_metadata(&entry_path).await {
                Ok(metadata) => {
                    self.process_entry(&entry_path, &metadata, depth, stack)
                        .await;
                }
                Err(_) => {
                    self.results.add_error();
                }
            }
        }
    }

    /// Gets metadata for an entry, respecting symlink handling options
    async fn get_entry_metadata(&self, path: &Path) -> Result<Metadata, std::io::Error> {
        if self.options.follow_symlinks {
            fs::metadata(path).await
        } else {
            fs::symlink_metadata(path).await
        }
    }

    /// Processes a single directory entry
    async fn process_entry(
        &mut self,
        path: &Path,
        metadata: &Metadata,
        depth: usize,
        stack: &mut Vec<(PathBuf, usize)>,
    ) {
        if metadata.is_dir() {
            self.handle_directory(path, metadata, depth, stack);
        } else if metadata.is_file() {
            self.handle_file(path, metadata);
        }
        // Ignore other file types (devices, pipes, etc.)
    }

    /// Handles directory entries
    fn handle_directory(
        &mut self,
        path: &Path,
        metadata: &Metadata,
        depth: usize,
        stack: &mut Vec<(PathBuf, usize)>,
    ) {
        // Only recurse if recursive scanning is enabled
        if !self.options.recursive {
            return;
        }

        // Skip symbolic links to directories unless explicitly following them
        if !self.options.should_follow_symlink(metadata.is_symlink()) {
            return;
        }

        // Add to stack for processing
        stack.push((path.to_path_buf(), depth + 1));
    }

    /// Handles file entries
    fn handle_file(&mut self, path: &Path, metadata: &Metadata) {
        let file_size = metadata.len();

        // Check file size constraints
        if !self.options.should_process_file_size(file_size) {
            return;
        }

        let category = FileCategory::from_path(path);
        let is_supported = category.is_extractable();

        self.results.add_file(category, file_size, is_supported);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_scanner_creation() {
        let scanner = DirectoryScanner::new();
        assert!(scanner.options.recursive);

        let custom_options = ScanOptions::new().with_recursive(false);
        let custom_scanner = DirectoryScanner::with_options(custom_options);
        assert!(!custom_scanner.options.recursive);
    }

    #[test]
    fn test_validate_scan_path() {
        let scanner = DirectoryScanner::new();

        // Test with non-existent path
        let non_existent = PathBuf::from("/non/existent/path");
        let result = scanner.validate_scan_path(&non_existent);
        assert!(result.is_err());
    }

    // Note: More comprehensive tests would require setting up actual test directories
    // which is beyond the scope of this refactoring exercise
}
