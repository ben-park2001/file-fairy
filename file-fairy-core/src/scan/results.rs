use super::stats::CategoryStats;
use crate::file_category::FileCategory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete scan results for a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    /// Path that was scanned
    pub scanned_path: PathBuf,
    /// Statistics by file category
    pub categories: HashMap<FileCategory, CategoryStats>,
    /// Total number of files scanned
    pub total_files: usize,
    /// Total size of all files (in bytes)
    pub total_size: u64,
    /// Total number of supported files
    pub total_supported: usize,
    /// Total size of supported files (in bytes)
    pub total_supported_size: u64,
    /// Files that couldn't be processed due to errors
    pub error_count: usize,
    /// Timestamp when scan was performed
    pub scan_timestamp: chrono::DateTime<chrono::Utc>,
}

impl ScanResults {
    /// Creates a new ScanResults for the given path
    pub fn new(path: PathBuf) -> Self {
        Self {
            scanned_path: path,
            categories: HashMap::new(),
            total_files: 0,
            total_size: 0,
            total_supported: 0,
            total_supported_size: 0,
            error_count: 0,
            scan_timestamp: chrono::Utc::now(),
        }
    }

    /// Adds a file to the scan results
    pub fn add_file(&mut self, category: FileCategory, size: u64, is_supported: bool) {
        // Update category stats
        let stats = self
            .categories
            .entry(category)
            .or_insert_with(CategoryStats::new);
        stats.add_file(size, is_supported);

        // Update totals
        self.total_files += 1;
        self.total_size += size;

        if is_supported {
            self.total_supported += 1;
            self.total_supported_size += size;
        }
    }

    /// Increments the error count
    pub fn add_error(&mut self) {
        self.error_count += 1;
    }

    /// Returns the overall support percentage
    pub fn overall_support_percentage(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.total_supported as f64 / self.total_files as f64) * 100.0
        }
    }

    /// Returns true if any supported files were found
    pub fn has_supported_files(&self) -> bool {
        self.total_supported > 0
    }

    /// Returns true if any errors occurred during scanning
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Returns the categories sorted by file count (descending)
    pub fn categories_by_count(&self) -> Vec<(&FileCategory, &CategoryStats)> {
        let mut sorted: Vec<_> = self.categories.iter().collect();
        sorted.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        sorted
    }

    /// Returns a formatted summary table as a string
    pub fn format_summary(&self) -> String {
        let formatter = ScanResultsFormatter::new(self);
        formatter.format()
    }

    /// Formats file size in human-readable format
    pub fn format_size(size: u64) -> String {
        format_file_size(size)
    }
}

/// Handles formatting of scan results for display
struct ScanResultsFormatter<'a> {
    results: &'a ScanResults,
}

impl<'a> ScanResultsFormatter<'a> {
    fn new(results: &'a ScanResults) -> Self {
        Self { results }
    }

    fn format(&self) -> String {
        let mut output = String::new();
        
        self.add_header(&mut output);
        self.add_overall_summary(&mut output);
        self.add_category_table(&mut output);
        self.add_conclusion(&mut output);
        
        output
    }

    fn add_header(&self, output: &mut String) {
        output.push_str(&format!(
            "📁 Scan Results for: {}\n",
            self.results.scanned_path.display()
        ));
        output.push_str(&format!(
            "🕒 Scanned at: {}\n\n",
            self.results.scan_timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
    }

    fn add_overall_summary(&self, output: &mut String) {
        output.push_str("📊 OVERALL SUMMARY\n");
        output.push_str(&format!("Total Files: {}\n", self.results.total_files));
        output.push_str(&format!(
            "Total Size: {}\n",
            format_file_size(self.results.total_size)
        ));
        output.push_str(&format!(
            "Supported Files: {} ({:.1}%)\n",
            self.results.total_supported,
            self.results.overall_support_percentage()
        ));
        output.push_str(&format!(
            "Supported Size: {}\n",
            format_file_size(self.results.total_supported_size)
        ));

        if self.results.has_errors() {
            output.push_str(&format!("⚠️  Errors encountered: {}\n", self.results.error_count));
        }
    }

    fn add_category_table(&self, output: &mut String) {
        if self.results.categories.is_empty() {
            return;
        }

        output.push_str("\n📋 BY CATEGORY\n");
        self.add_table_header(output);
        
        for (category, stats) in self.results.categories_by_count() {
            self.add_table_row(output, category, stats);
        }
        
        self.add_table_footer(output);
    }

    fn add_table_header(&self, output: &mut String) {
        output.push_str(
            "┌─────────────────────┬─────────┬──────────┬─────────────┬───────────────┐\n",
        );
        output.push_str(
            "│ Category            │  Count  │   Size   │  Supported  │  Support %    │\n",
        );
        output.push_str(
            "├─────────────────────┼─────────┼──────────┼─────────────┼───────────────┤\n",
        );
    }

    fn add_table_row(&self, output: &mut String, category: &FileCategory, stats: &CategoryStats) {
        output.push_str(&format!(
            "│ {:<19} │ {:>7} │ {:>8} │ {:>11} │ {:>12.1}% │\n",
            category.description(),
            stats.count,
            format_file_size(stats.total_size),
            stats.supported_count,
            stats.support_percentage()
        ));
    }

    fn add_table_footer(&self, output: &mut String) {
        output.push_str(
            "└─────────────────────┴─────────┴──────────┴─────────────┴───────────────┘\n",
        );
    }

    fn add_conclusion(&self, output: &mut String) {
        if self.results.has_supported_files() {
            output.push_str("\n✨ File Fairy can help organize ");
            output.push_str(&format!(
                "{} files ({})!\n",
                self.results.total_supported,
                format_file_size(self.results.total_supported_size)
            ));
        } else {
            output.push_str("\n😔 No supported files found in this directory.\n");
        }
    }
}

/// Formats file size in human-readable format
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_scan_results_creation() {
        let path = PathBuf::from("/test");
        let results = ScanResults::new(path.clone());
        
        assert_eq!(results.scanned_path, path);
        assert_eq!(results.total_files, 0);
        assert!(!results.has_supported_files());
        assert!(!results.has_errors());
    }

    #[test]
    fn test_add_files() {
        let mut results = ScanResults::new(PathBuf::from("/test"));
        
        results.add_file(FileCategory::Documents, 1024, true);
        results.add_file(FileCategory::Documents, 2048, false);
        
        assert_eq!(results.total_files, 2);
        assert_eq!(results.total_supported, 1);
        assert_eq!(results.overall_support_percentage(), 50.0);
        assert!(results.has_supported_files());
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_categories_by_count() {
        let mut results = ScanResults::new(PathBuf::from("/test"));
        
        results.add_file(FileCategory::Documents, 1024, true);
        results.add_file(FileCategory::Documents, 1024, true);
        results.add_file(FileCategory::Data, 1024, true);
        
        let sorted = results.categories_by_count();
        assert_eq!(sorted.len(), 2);
        // Documents should come first (2 files vs 1 file)
        assert_eq!(*sorted[0].0, FileCategory::Documents);
        assert_eq!(sorted[0].1.count, 2);
    }
}
