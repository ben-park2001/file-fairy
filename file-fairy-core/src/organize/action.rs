use crate::file_category::FileCategory;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a single organization action to be performed on a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeAction {
    /// The source file path
    pub source: PathBuf,
    /// The suggested destination path
    pub destination: PathBuf,
    /// The original filename
    pub original_filename: String,
    /// The suggested new filename
    pub suggested_filename: String,
    /// The file category
    pub category: FileCategory,
    /// The file size in bytes
    pub size: u64,
    /// Whether this action has been approved by the user
    pub approved: bool,
}

impl OrganizeAction {
    /// Creates a new organize action
    pub fn new(
        source: PathBuf,
        destination: PathBuf,
        original_filename: String,
        suggested_filename: String,
        category: FileCategory,
        size: u64,
    ) -> Self {
        Self {
            source,
            destination,
            original_filename,
            suggested_filename,
            category,
            size,
            approved: false,
        }
    }

    /// Approves this action for execution
    pub fn approve(&mut self) {
        self.approved = true;
    }

    /// Returns whether this action would move the file to a different directory
    pub fn changes_directory(&self) -> bool {
        self.source.parent() != self.destination.parent()
    }

    /// Returns whether this action would rename the file
    pub fn changes_filename(&self) -> bool {
        self.source.file_name() != self.destination.file_name()
    }

    /// Returns a human-readable description of what this action will do
    pub fn describe_action(&self) -> String {
        let moves_dir = self.changes_directory();
        let renames = self.changes_filename();

        match (moves_dir, renames) {
            (true, true) => format!(
                "Move and rename: {} -> {}",
                self.source.display(),
                self.destination.display()
            ),
            (true, false) => format!(
                "Move: {} -> {}",
                self.source.display(),
                self.destination.parent().unwrap().display()
            ),
            (false, true) => format!(
                "Rename: {} -> {}",
                self.original_filename, self.suggested_filename
            ),
            (false, false) => "No changes".to_string(),
        }
    }

    /// Updates the suggested filename and recalculates the destination
    pub fn update_suggested_filename(&mut self, new_filename: String) {
        self.suggested_filename = new_filename.clone();
        if let Some(parent) = self.source.parent() {
            self.destination = parent.join(new_filename);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_organize_action_creation() {
        let action = OrganizeAction::new(
            PathBuf::from("/test/old.txt"),
            PathBuf::from("/test/new.txt"),
            "old.txt".to_string(),
            "new.txt".to_string(),
            FileCategory::Documents,
            1024,
        );

        assert!(!action.approved);
        assert_eq!(action.size, 1024);
        assert!(action.changes_filename());
        assert!(!action.changes_directory());
    }

    #[test]
    fn test_action_description() {
        let action = OrganizeAction::new(
            PathBuf::from("/test/old.txt"),
            PathBuf::from("/other/new.txt"),
            "old.txt".to_string(),
            "new.txt".to_string(),
            FileCategory::Documents,
            1024,
        );

        let description = action.describe_action();
        assert!(description.contains("Move and rename"));
    }
}
