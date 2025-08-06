use serde::{Deserialize, Serialize};

/// Represents statistics for a single file category
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryStats {
    /// Number of files in this category
    pub count: usize,
    /// Total size of all files in this category (in bytes)
    pub total_size: u64,
    /// Number of files that can be processed by File Fairy
    pub supported_count: usize,
    /// Size of supported files (in bytes)  
    pub supported_size: u64,
}

impl CategoryStats {
    /// Creates a new empty CategoryStats
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file to the statistics
    pub fn add_file(&mut self, size: u64, is_supported: bool) {
        self.count += 1;
        self.total_size += size;

        if is_supported {
            self.supported_count += 1;
            self.supported_size += size;
        }
    }

    /// Returns the percentage of supported files in this category
    pub fn support_percentage(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            (self.supported_count as f64 / self.count as f64) * 100.0
        }
    }

    /// Returns true if this category has any files
    pub fn has_files(&self) -> bool {
        self.count > 0
    }

    /// Returns true if this category has any supported files
    pub fn has_supported_files(&self) -> bool {
        self.supported_count > 0
    }

    /// Returns the average file size in this category
    pub fn average_file_size(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.total_size as f64 / self.count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_stats() {
        let stats = CategoryStats::new();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.supported_count, 0);
        assert_eq!(stats.support_percentage(), 0.0);
        assert!(!stats.has_files());
        assert!(!stats.has_supported_files());
    }

    #[test]
    fn test_add_files() {
        let mut stats = CategoryStats::new();

        stats.add_file(1024, true);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.supported_count, 1);
        assert_eq!(stats.support_percentage(), 100.0);

        stats.add_file(2048, false);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.supported_count, 1);
        assert_eq!(stats.support_percentage(), 50.0);
    }

    #[test]
    fn test_average_file_size() {
        let mut stats = CategoryStats::new();
        stats.add_file(1000, true);
        stats.add_file(2000, true);

        assert_eq!(stats.average_file_size(), 1500.0);
    }
}
