use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedFolder {
    pub id: String,
    pub path: String,
    pub name: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryItem {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Option<Vec<DirectoryItem>>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn select_folder() -> Result<Option<String>, String> {
    // For now, we'll return a simple mock implementation
    // The dialog functionality will be handled from the frontend using @tauri-apps/plugin-dialog
    Ok(Some("/Users/username/Documents".to_string()))
}

#[tauri::command]
fn get_folder_name(path: &str) -> Result<String, String> {
    let path_obj = Path::new(path);
    if let Some(name) = path_obj.file_name() {
        if let Some(name_str) = name.to_str() {
            Ok(name_str.to_string())
        } else {
            Err("Could not convert folder name to string".to_string())
        }
    } else {
        Err("Could not get folder name".to_string())
    }
}

#[tauri::command]
fn get_path_info(path: &str) -> Result<(String, bool), String> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let is_directory = path_obj.is_dir();
    let name = if is_directory {
        path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    } else {
        path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    };

    Ok((name, is_directory))
}

#[tauri::command]
fn folder_exists(path: &str) -> bool {
    Path::new(path).is_dir()
}

#[tauri::command]
fn read_directory_structure(path: &str) -> Result<DirectoryItem, String> {
    read_directory_recursive(path, 0, 2) // Limit to 2 levels deep
}

fn read_directory_recursive(
    path: &str,
    current_depth: u32,
    max_depth: u32,
) -> Result<DirectoryItem, String> {
    let path_obj = Path::new(path);

    if !path_obj.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let name = path_obj
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

    if !path_obj.is_dir() {
        return Ok(DirectoryItem {
            name,
            path: path.to_string(),
            is_directory: false,
            children: None,
        });
    }

    let mut children = Vec::new();

    if current_depth < max_depth {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(entry_path) = entry.path().to_str() {
                    if let Ok(item) =
                        read_directory_recursive(entry_path, current_depth + 1, max_depth)
                    {
                        children.push(item);
                    }
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

    Ok(DirectoryItem {
        name,
        path: path.to_string(),
        is_directory: true,
        children: Some(children),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            select_folder,
            get_folder_name,
            get_path_info,
            folder_exists,
            read_directory_structure
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
