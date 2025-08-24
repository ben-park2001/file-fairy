//! File Fairy - An intelligent file organization application
//!
//! This application provides automated file organization using AI to analyze,
//! categorize, and intelligently place files in appropriate directories.

mod error;
mod extractors;
mod services;
mod types;

use services::{ExtractionService, FileSystemService, RenamerService, WatchService};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use types::{DirectoryItem, PathInfo, WatchedFolderInfo};

use crate::types::FileChunkSchema;

// Global watch service instance
static WATCH_SERVICE: OnceLock<Arc<WatchService>> = OnceLock::new();

fn get_watch_service() -> Arc<WatchService> {
    WATCH_SERVICE
        .get_or_init(|| Arc::new(WatchService::new()))
        .clone()
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

/// Get the folder name from a path
#[tauri::command]
fn get_folder_name(path: &str) -> Result<String, String> {
    FileSystemService::get_folder_name(path).map_err(|e| e.to_string())
}

/// Get path information (name and whether it's a directory)
#[tauri::command]
fn get_path_info(path: &str) -> Result<PathInfo, String> {
    FileSystemService::get_path_info(path).map_err(|e| e.to_string())
}

/// Check if a folder exists
#[tauri::command]
fn folder_exists(path: &str) -> bool {
    FileSystemService::folder_exists(path)
}

/// Read directory structure with depth limit
#[tauri::command]
fn read_directory_structure(path: &str) -> Result<DirectoryItem, String> {
    FileSystemService::read_directory_structure(path).map_err(|e| e.to_string())
}

/// Extract content from a file
#[tauri::command]
async fn extract_file_content(file_path: &str) -> Result<String, String> {
    let path = Path::new(file_path);
    ExtractionService::extract_file_content(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Generate filename using Ollama
#[tauri::command]
async fn generate_new_filename(file_path: &str) -> Result<String, String> {
    let renamer_service = RenamerService::new();
    renamer_service
        .generate_new_filename(file_path)
        .await
        .map_err(|e| e.to_string())
}

/// Rename/move a file
#[tauri::command]
async fn rename_file(old_path: &str, new_path: &str) -> Result<(), String> {
    let renamer_service = RenamerService::new();
    renamer_service.rename_file(old_path, new_path).await
}

// Watch Service Commands

/// Register a folder to watch for automatic file organization
#[tauri::command]
async fn watch_register_folder(folder_path: &str) -> Result<(), String> {
    let watch_service = get_watch_service();
    let path = Path::new(folder_path);
    watch_service.register_folder(path).await
}

/// Pause watching a folder
#[tauri::command]
async fn watch_pause_folder(folder_path: &str) -> Result<(), String> {
    let watch_service = get_watch_service();
    let path = Path::new(folder_path);
    watch_service.pause_folder(path)
}

/// Resume watching a folder
#[tauri::command]
async fn watch_resume_folder(folder_path: &str) -> Result<(), String> {
    let watch_service = get_watch_service();
    let path = Path::new(folder_path);
    watch_service.resume_folder(path)
}

/// Remove a folder from watch list
#[tauri::command]
async fn watch_remove_folder(folder_path: &str) -> Result<(), String> {
    let watch_service = get_watch_service();
    let path = Path::new(folder_path);
    watch_service.remove_folder(path)
}

/// List all watched folders
#[tauri::command]
async fn watch_list_folders() -> Result<Vec<WatchedFolderInfo>, String> {
    let watch_service = get_watch_service();
    Ok(watch_service.list_watched_folders())
}

#[tauri::command]
async fn search_documents(query: &str, limit: usize) -> Result<Vec<FileChunkSchema>, String> {
    let watch_service = get_watch_service();
    watch_service
        .search_documents(query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|_app| {
            // Initialize the watch service and database on application startup
            tauri::async_runtime::spawn(async {
                let watch_service = get_watch_service();
                if let Err(e) = watch_service.initialize().await {
                    eprintln!("Failed to initialize watch service: {}", e);
                } else {
                    println!("Watch service and database initialized successfully");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_folder_name,
            get_path_info,
            folder_exists,
            read_directory_structure,
            extract_file_content,
            generate_new_filename,
            rename_file,
            watch_register_folder,
            watch_pause_folder,
            watch_resume_folder,
            watch_remove_folder,
            watch_list_folders,
            search_documents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
