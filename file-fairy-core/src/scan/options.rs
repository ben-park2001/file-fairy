/// Options for customizing the scan behavior
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Whether to scan subdirectories recursively
    pub recursive: bool,
    /// Maximum depth to scan (None for unlimited)
    pub max_depth: Option<usize>,
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
    /// File size limit (skip files larger than this, in bytes)
    pub max_file_size: Option<u64>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            recursive: true,
            max_depth: None,
            follow_symlinks: false,
            max_file_size: None,
        }
    }
}

impl ScanOptions {
    /// Creates new scan options with default values
    pub fn new() -> Self {
        Self::default()
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

    /// Checks if a file should be processed based on size constraints
    pub fn should_process_file_size(&self, size: u64) -> bool {
        if let Some(max_size) = self.max_file_size {
            size <= max_size
        } else {
            true
        }
    }

    /// Checks if we should process a directory at the given depth
    pub fn should_process_depth(&self, depth: usize) -> bool {
        if let Some(max_depth) = self.max_depth {
            depth < max_depth
        } else {
            true
        }
    }

    /// Checks if we should follow a symbolic link
    pub fn should_follow_symlink(&self, is_symlink: bool) -> bool {
        !is_symlink || self.follow_symlinks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let options = ScanOptions::default();
        assert!(options.recursive);
        assert!(options.max_depth.is_none());
        assert!(!options.follow_symlinks);
        assert!(options.max_file_size.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let options = ScanOptions::new()
            .with_recursive(false)
            .with_max_depth(3)
            .with_follow_symlinks(true)
            .with_max_file_size(1024);

        assert!(!options.recursive);
        assert_eq!(options.max_depth, Some(3));
        assert!(options.follow_symlinks);
        assert_eq!(options.max_file_size, Some(1024));
    }

    #[test]
    fn test_size_filtering() {
        let options = ScanOptions::new().with_max_file_size(1024);

        assert!(options.should_process_file_size(512));
        assert!(options.should_process_file_size(1024));
        assert!(!options.should_process_file_size(2048));
    }

    #[test]
    fn test_depth_filtering() {
        let options = ScanOptions::new().with_max_depth(2);

        assert!(options.should_process_depth(0));
        assert!(options.should_process_depth(1));
        assert!(!options.should_process_depth(2));
        assert!(!options.should_process_depth(3));
    }

    #[test]
    fn test_symlink_handling() {
        let follow = ScanOptions::new().with_follow_symlinks(true);
        let no_follow = ScanOptions::new().with_follow_symlinks(false);

        assert!(follow.should_follow_symlink(true));
        assert!(follow.should_follow_symlink(false));

        assert!(!no_follow.should_follow_symlink(true));
        assert!(no_follow.should_follow_symlink(false));
    }
}
