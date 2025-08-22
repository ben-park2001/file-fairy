//! File Fairy - An intelligent file organization application
//!
//! This application provides automated file organization using AI to analyze,
//! categorize, and intelligently place files in appropriate directories.

mod error;
mod extractors;
mod ollama;
mod services;
mod types;

use services::{ExtractionService, FileSystemService, OrganizationService, WatchService};
use std::path::Path;
use std::sync::{Arc, OnceLock};
use types::{DirectoryItem, OrganizationResult, PathInfo, WatchedFolderInfo};

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

/// Check Ollama service health
#[tauri::command]
async fn ollama_health_check() -> Result<bool, String> {
    let organization_service = OrganizationService::new();
    organization_service.health_check().await
}

/// List available Ollama models
#[tauri::command]
async fn ollama_list_models() -> Result<Vec<String>, String> {
    let organization_service = OrganizationService::new();
    organization_service.list_models().await
}

/// Generate summary using Ollama
#[tauri::command]
async fn ollama_generate_summary(content: &str, model: &str) -> Result<String, String> {
    let organization_service = OrganizationService::new();
    organization_service.generate_summary(content, model).await
}

/// Generate filename using Ollama
#[tauri::command]
async fn ollama_generate_filename(summary: &str, model: &str) -> Result<String, String> {
    let organization_service = OrganizationService::new();
    organization_service.generate_filename(summary, model).await
}

/// Generate organization path using Ollama
#[tauri::command]
async fn ollama_generate_organization_path(
    summary: &str,
    file_path: &str,
    folder_structure: Vec<String>,
    model: &str,
) -> Result<String, String> {
    let organization_service = OrganizationService::new();
    organization_service
        .generate_organization_path(summary, file_path, &folder_structure, model)
        .await
}

/// Analyze and organize a file
#[tauri::command]
async fn analyze_and_organize_file(
    file_path: &str,
    summary_model: &str,
    filename_model: &str,
) -> Result<OrganizationResult, String> {
    let organization_service = OrganizationService::new();
    organization_service
        .analyze_and_organize_file(file_path, summary_model, filename_model)
        .await
}

/// Rename/move a file
#[tauri::command]
async fn rename_file(old_path: &str, new_path: &str) -> Result<(), String> {
    let organization_service = OrganizationService::new();
    organization_service.rename_file(old_path, new_path).await
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

/// Update watch configuration
#[tauri::command]
async fn watch_update_config(recursive: bool, process_existing_files: bool) -> Result<(), String> {
    let watch_service = get_watch_service();
    let config = services::watch::WatchConfig {
        recursive,
        process_existing_files,
    };
    watch_service.update_config(config)
}

/// Get current watch configuration
#[tauri::command]
async fn watch_get_config() -> Result<services::watch::WatchConfig, String> {
    let watch_service = get_watch_service();
    watch_service.get_config()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_folder_name,
            get_path_info,
            folder_exists,
            read_directory_structure,
            extract_file_content,
            ollama_health_check,
            ollama_list_models,
            ollama_generate_summary,
            ollama_generate_filename,
            ollama_generate_organization_path,
            analyze_and_organize_file,
            rename_file,
            watch_register_folder,
            watch_pause_folder,
            watch_resume_folder,
            watch_remove_folder,
            watch_list_folders,
            watch_update_config,
            watch_get_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
