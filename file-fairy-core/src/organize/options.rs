use crate::file_category::FileCategory;
use std::path::PathBuf;

/// Options for organizing files
#[derive(Debug, Clone)]
pub struct OrganizeOptions {
    /// Whether to actually perform the actions (false = dry run)
    pub apply: bool,
    /// Whether to use interactive mode
    pub interactive: bool,
    /// Whether to scan subdirectories recursively
    pub recursive: bool,
    /// Maximum depth to scan (None for unlimited)
    pub max_depth: Option<usize>,
    /// Target directory for organized files (None = organize in place)
    pub target_dir: Option<PathBuf>,
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
    /// File size limit (skip files larger than this, in bytes)
    pub max_file_size: Option<u64>,
    /// Categories to include (None = all supported categories)
    pub include_categories: Option<Vec<FileCategory>>,
}

impl Default for OrganizeOptions {
    fn default() -> Self {
        Self {
            apply: false, // Safety first - default to dry run
            interactive: false,
            recursive: true,
            max_depth: None,
            target_dir: None,
            follow_symlinks: false,
            max_file_size: None,
            include_categories: None,
        }
    }
}

impl OrganizeOptions {
    /// Creates new organize options with safe defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to actually apply changes
    pub fn with_apply(mut self, apply: bool) -> Self {
        self.apply = apply;
        self
    }

    /// Sets whether to use interactive mode
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Sets whether to scan recursively
    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// Sets the maximum depth for recursive scanning
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Sets the target directory for organized files
    pub fn with_target_dir(mut self, target_dir: PathBuf) -> Self {
        self.target_dir = Some(target_dir);
        self
    }

    /// Sets whether to follow symbolic links
    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Sets the maximum file size to process
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = Some(size);
        self
    }

    /// Sets which categories to include
    pub fn with_categories(mut self, categories: Vec<FileCategory>) -> Self {
        self.include_categories = Some(categories);
        self
    }

    /// Checks if a category should be included based on options
    pub fn should_include_category(&self, category: &FileCategory) -> bool {
        if let Some(ref include_cats) = self.include_categories {
            include_cats.contains(category)
        } else {
            true
        }
    }

    /// Checks if a file size should be processed based on options
    pub fn should_process_file_size(&self, size: u64) -> bool {
        if let Some(max_size) = self.max_file_size {
            size <= max_size
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = OrganizeOptions::default();
        assert!(!options.apply);
        assert!(!options.interactive);
        assert!(options.recursive);
        assert!(options.max_depth.is_none());
    }

    #[test]
    fn test_category_filtering() {
        let options = OrganizeOptions::new().with_categories(vec![FileCategory::Documents]);

        assert!(options.should_include_category(&FileCategory::Documents));
        assert!(!options.should_include_category(&FileCategory::Data));
    }

    #[test]
    fn test_size_filtering() {
        let options = OrganizeOptions::new().with_max_file_size(1024);

        assert!(options.should_process_file_size(512));
        assert!(options.should_process_file_size(1024));
        assert!(!options.should_process_file_size(2048));
    }
}
