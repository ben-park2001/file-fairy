//! Type definitions for the File Fairy application
//!
//! Contains data structures for representing files, directories,
//! organization results, and other core application entities.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents the state of a watched folder
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchState {
    Active,
    Paused,
}

/// Information about a watched folder
#[derive(Debug, Clone)]
pub struct WatchedFolder {
    pub path: std::path::PathBuf,
    pub state: WatchState,
}

/// Represents a watched folder in the application for API responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WatchedFolderInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub is_active: bool,
}

/// Represents an item in a directory structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryItem {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Option<Vec<DirectoryItem>>,
}

impl DirectoryItem {
    /// Create a new file item
    pub fn new_file(name: String, path: String) -> Self {
        Self {
            name,
            path,
            is_directory: false,
            children: None,
        }
    }

    /// Create a new directory item with children
    pub fn new_directory(name: String, path: String, children: Vec<DirectoryItem>) -> Self {
        Self {
            name,
            path,
            is_directory: true,
            children: Some(children),
        }
    }
}

/// Result of file analysis and organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrganizationResult {
    pub summary: String,
    pub new_filename: String,
}

impl OrganizationResult {
    /// Create a new organization result
    pub fn new(summary: String, new_filename: String) -> Self {
        Self {
            summary,
            new_filename,
        }
    }
}

/// Path information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathInfo {
    pub name: String,
    pub is_directory: bool,
}

impl PathInfo {
    /// Create new path info
    pub fn new(name: String, is_directory: bool) -> Self {
        Self { name, is_directory }
    }

    /// Create path info from a path
    pub fn from_path(path: &str) -> Result<Self, String> {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        let name = path_obj
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let is_directory = path_obj.is_dir();

        Ok(Self::new(name, is_directory))
    }
}
