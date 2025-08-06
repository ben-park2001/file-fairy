use super::action::OrganizeAction;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Results of an organization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeResults {
    /// The path that was organized
    pub organized_path: PathBuf,
    /// All proposed actions
    pub actions: Vec<OrganizeAction>,
    /// Number of files that were successfully processed
    pub successful_actions: usize,
    /// Number of files that had errors
    pub failed_actions: usize,
    /// Number of files that were skipped
    pub skipped_actions: usize,
    /// Errors encountered during processing
    pub errors: Vec<String>,
    /// Timestamp when organization was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether this was a dry run (preview) or actual execution
    pub dry_run: bool,
}

impl OrganizeResults {
    /// Creates new organize results
    pub fn new(path: PathBuf, dry_run: bool) -> Self {
        Self {
            organized_path: path,
            actions: Vec::new(),
            successful_actions: 0,
            failed_actions: 0,
            skipped_actions: 0,
            errors: Vec::new(),
            timestamp: chrono::Utc::now(),
            dry_run,
        }
    }

    /// Adds an action to the results
    pub fn add_action(&mut self, action: OrganizeAction) {
        self.actions.push(action);
    }

    /// Records a successful action
    pub fn record_success(&mut self) {
        self.successful_actions += 1;
    }

    /// Records a failed action
    pub fn record_failure(&mut self, error: String) {
        self.failed_actions += 1;
        self.errors.push(error);
    }

    /// Records a skipped action
    pub fn record_skip(&mut self) {
        self.skipped_actions += 1;
    }

    /// Returns the total number of actions
    pub fn total_actions(&self) -> usize {
        self.actions.len()
    }

    /// Returns the number of approved actions
    pub fn approved_actions(&self) -> usize {
        self.actions.iter().filter(|a| a.approved).count()
    }

    /// Formats the results as a preview table
    pub fn format_preview(&self) -> String {
        let formatter = ResultsFormatter::new(self);
        formatter.format()
    }
}

/// Handles formatting of organize results for display
struct ResultsFormatter<'a> {
    results: &'a OrganizeResults,
}

impl<'a> ResultsFormatter<'a> {
    fn new(results: &'a OrganizeResults) -> Self {
        Self { results }
    }

    fn format(&self) -> String {
        let mut output = String::new();

        self.add_header(&mut output);
        self.add_summary(&mut output);
        self.add_actions_table(&mut output);
        self.add_next_steps(&mut output);
        self.add_errors(&mut output);

        output
    }

    fn add_header(&self, output: &mut String) {
        if self.results.dry_run {
            output.push_str("🔍 ORGANIZATION PREVIEW (DRY RUN)\n");
        } else {
            output.push_str("✨ ORGANIZATION RESULTS\n");
        }
        output.push_str("═══════════════════════════════════\n\n");
    }

    fn add_summary(&self, output: &mut String) {
        output.push_str(&format!(
            "📁 Path: {}\n",
            self.results.organized_path.display()
        ));
        output.push_str(&format!(
            "🕒 Time: {}\n",
            self.results.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!(
            "📊 Total Files: {}\n",
            self.results.total_actions()
        ));

        if self.results.dry_run {
            output.push_str(&format!(
                "✅ Approved: {}\n",
                self.results.approved_actions()
            ));
        } else {
            output.push_str(&format!(
                "✅ Successful: {}\n",
                self.results.successful_actions
            ));
            output.push_str(&format!("❌ Failed: {}\n", self.results.failed_actions));
            output.push_str(&format!("⏭️  Skipped: {}\n", self.results.skipped_actions));
        }
    }

    fn add_actions_table(&self, output: &mut String) {
        if self.results.actions.is_empty() {
            return;
        }

        output.push_str("\n📋 ACTIONS\n");
        self.add_table_header(output);

        for action in &self.results.actions {
            self.add_table_row(output, action);
        }

        self.add_table_footer(output);
    }

    fn add_table_header(&self, output: &mut String) {
        output.push_str("┌──┬─────────────────────────────┬──────────┬─────────────────────────────┬──────────┐\n");
        output.push_str("│  │ Original                    │   Size   │ Suggested                   │ Category │\n");
        output.push_str("├──┼─────────────────────────────┼──────────┼─────────────────────────────┼──────────┤\n");
    }

    fn add_table_row(&self, output: &mut String, action: &OrganizeAction) {
        let status = self.get_action_status(action);
        let original_display = self.truncate_filename(&action.original_filename, 27);
        let suggested_display = self.truncate_filename(&action.suggested_filename, 27);
        let size_display = format_file_size(action.size);

        output.push_str(&format!(
            "│{:>2}│ {:<27} │ {:>8} │ {:<27} │ {:>8} │\n",
            status,
            original_display,
            size_display,
            suggested_display,
            action.category.as_ref()
        ));
    }

    fn add_table_footer(&self, output: &mut String) {
        output.push_str("└──┴─────────────────────────────┴──────────┴─────────────────────────────┴──────────┘\n");
    }

    fn get_action_status(&self, action: &OrganizeAction) -> &str {
        if self.results.dry_run {
            if action.approved { "✓" } else { " " }
        } else {
            "✓" // In actual run, we only show executed actions
        }
    }

    fn truncate_filename(&self, filename: &str, max_len: usize) -> String {
        if filename.len() > max_len {
            format!(
                "{}...",
                &filename.chars().take(max_len - 3).collect::<String>()
            )
        } else {
            filename.to_string()
        }
    }

    fn add_next_steps(&self, output: &mut String) {
        if self.results.dry_run && self.results.total_actions() > 0 {
            output.push_str("\n💡 NEXT STEPS\n");
            output.push_str("• Review the suggested changes above\n");
            output.push_str("• Run with --apply to execute the approved actions\n");
            output.push_str("• Use --interactive for step-by-step control\n");
        }
    }

    fn add_errors(&self, output: &mut String) {
        if !self.results.errors.is_empty() {
            output.push_str("\n❌ ERRORS\n");
            for error in &self.results.errors {
                output.push_str(&format!("• {}\n", error));
            }
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

    #[test]
    fn test_results_creation() {
        let results = OrganizeResults::new(PathBuf::from("/test"), true);
        assert!(results.dry_run);
        assert_eq!(results.total_actions(), 0);
        assert_eq!(results.approved_actions(), 0);
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn test_record_operations() {
        let mut results = OrganizeResults::new(PathBuf::from("/test"), false);

        results.record_success();
        results.record_failure("test error".to_string());
        results.record_skip();

        assert_eq!(results.successful_actions, 1);
        assert_eq!(results.failed_actions, 1);
        assert_eq!(results.skipped_actions, 1);
        assert_eq!(results.errors.len(), 1);
    }
}
