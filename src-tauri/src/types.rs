//! Type definitions for the File Fairy application
//!
//! Contains data structures for representing files, directories,
//! organization results, and other core application entities.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents the state of a watched folder
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchState {
    Active,
    Paused,
}

/// Information about a watched folder
#[derive(Debug, Clone)]
pub struct WatchedFolder {
    pub path: PathBuf,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunkSchema {
    /// Unique identifier for the chunk
    pub chunk_id: u64,
    /// Timestamp when the chunk was created
    pub created_at: u64,
    /// Path to the original file
    pub file_path: String,
    /// Name of the original file
    pub file_name: String,
    /// Text content of the chunk
    pub text: String,
    /// Vector representation of the file chunk
    pub vector: Vec<f32>,
}

impl FileChunkSchema {
    /// Create a new file chunk schema
    pub fn new(chunk_id: u64, file_path: String, text: String, vector: Vec<f32>) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file_name = Path::new(&file_path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            chunk_id,
            created_at,
            file_path,
            file_name,
            text,
            vector,
        }
    }
}
