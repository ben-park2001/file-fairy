//! File system operations service module
//!
//! Provides utilities for reading directory structures, path manipulation,
//! and file system queries with proper error handling.

use crate::{
    error::AppResult,
    types::{DirectoryItem, PathInfo},
};
use std::{fs, path::Path};

/// Service for file system operations
pub struct FileSystemService;

impl FileSystemService {
    /// Get information about a path (name and whether it's a directory)
    pub fn get_path_info(path: &str) -> AppResult<PathInfo> {
        PathInfo::from_path(path).map_err(|e| crate::error::AppError::Path { message: e })
    }

    /// Get the folder name from a path
    pub fn get_folder_name(path: &str) -> AppResult<String> {
        let path_obj = Path::new(path);

        path_obj
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .ok_or_else(|| crate::error::AppError::Path {
                message: "Could not get folder name".to_string(),
            })
    }

    /// Check if a folder exists
    pub fn folder_exists(path: &str) -> bool {
        Path::new(path).is_dir()
    }

    /// Read directory structure with depth limit
    pub fn read_directory_structure(path: &str) -> AppResult<DirectoryItem> {
        Self::read_directory_recursive(path, 0, 2) // Limit to 2 levels deep
    }

    /// Recursively read directory structure
    fn read_directory_recursive(
        path: &str,
        current_depth: u32,
        max_depth: u32,
    ) -> AppResult<DirectoryItem> {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(crate::error::AppError::Path {
                message: format!("Path does not exist: {}", path),
            });
        }

        let name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        if !path_obj.is_dir() {
            return Ok(DirectoryItem::new_file(name, path.to_string()));
        }

        let mut children = Vec::new();

        if current_depth < max_depth {
            let entries = fs::read_dir(path)?;

            for entry in entries.flatten() {
                if let Some(entry_path) = entry.path().to_str() {
                    if let Ok(item) =
                        Self::read_directory_recursive(entry_path, current_depth + 1, max_depth)
                    {
                        children.push(item);
                    }
                }
            }
        }

        // Sort: directories first, then files, both alphabetically
        children.sort_by(|a, b| match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        Ok(DirectoryItem::new_directory(
            name,
            path.to_string(),
            children,
        ))
    }
}
