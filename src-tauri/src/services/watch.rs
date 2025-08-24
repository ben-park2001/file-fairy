use crate::error::AppResult;
use crate::services::{DatabaseService, ExtractionService};
use crate::types::{FileChunkSchema, WatchState, WatchedFolder, WatchedFolderInfo};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::{self, JoinHandle};

/// Configuration for watch operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WatchConfig {
    pub recursive: bool,
    pub process_existing_files: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            recursive: true,
            process_existing_files: true,
        }
    }
}

/// Embedding-based folder watch service for semantic search
#[derive(Clone)]
pub struct WatchService {
    watched: Arc<Mutex<HashMap<PathBuf, WatchedFolder>>>,
    watchers: Arc<Mutex<HashMap<PathBuf, JoinHandle<()>>>>,
    config: Arc<Mutex<WatchConfig>>,
    database: Arc<DatabaseService>,
}

impl WatchService {
    /// Create a new watch service with default configuration
    pub fn new() -> Self {
        Self::with_config(WatchConfig::default())
    }

    /// Create a new watch service with custom configuration
    pub fn with_config(config: WatchConfig) -> Self {
        let database = Arc::new(DatabaseService::default());
        Self {
            watched: Arc::new(Mutex::new(HashMap::new())),
            watchers: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(config)),
            database,
        }
    }

    /// Initialize the watch service and database
    pub async fn initialize(&self) -> Result<(), String> {
        self.database.initialize().await.map_err(|e| e.to_string())
    }

    /// Register a folder to watch for embedding-based semantic search
    pub async fn register_folder(&self, folder: &Path) -> Result<(), String> {
        // Ensure database is initialized before registering
        if let Err(e) = self.database.initialize().await {
            return Err(format!("Failed to initialize database: {}", e));
        }

        let folder = folder
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path '{}': {}", folder.display(), e))?;

        // Check if already watching
        {
            let watched = self.watched.lock().unwrap();
            if watched.contains_key(&folder) {
                return Err("Folder is already being watched".to_string());
            }
        }

        // Check if folder exists and is a directory
        if !folder.exists() {
            return Err(format!("Folder does not exist: {}", folder.display()));
        }
        if !folder.is_dir() {
            return Err(format!("Path is not a directory: {}", folder.display()));
        }

        // Add to watched folders
        {
            let mut watched = self.watched.lock().unwrap();
            watched.insert(
                folder.clone(),
                WatchedFolder {
                    path: folder.clone(),
                    state: WatchState::Active,
                },
            );
        }

        println!(
            "Registered folder for embedding watch: {}",
            folder.display()
        );

        // Spawn watcher for embeddings
        let handle = self.spawn_watcher(folder.clone()).await;
        {
            let mut watchers = self.watchers.lock().unwrap();
            watchers.insert(folder, handle);
        }

        Ok(())
    }

    /// Process existing files in a folder to create embeddings
    async fn process_existing_files_for_embeddings(
        &self,
        folder: &PathBuf,
        config: &Arc<Mutex<WatchConfig>>,
    ) -> Result<(), String> {
        let recursive = {
            let config = config.lock().map_err(|_| "Failed to lock config")?;
            config.recursive
        };

        println!(
            "Creating embeddings for existing files in folder: {}",
            folder.display()
        );

        let files = Self::collect_files_static(folder, recursive)?;
        let supported_files = files
            .into_iter()
            .filter(|path| Self::is_supported_file_static(path))
            .collect::<Vec<_>>();

        println!(
            "Found {} supported files to create embeddings for",
            supported_files.len()
        );

        // Process files in batches to avoid overwhelming the system
        const BATCH_SIZE: usize = 10;
        for batch in supported_files.chunks(BATCH_SIZE) {
            let mut tasks = Vec::new();

            for file_path in batch {
                let path = file_path.clone();
                let watch_service = self.clone();

                let task = tokio::spawn(async move {
                    watch_service.upsert_file_to_database(path).await;
                });
                tasks.push(task);
            }

            // Wait for the batch to complete before starting the next one
            for task in tasks {
                let _ = task.await;
            }

            // Small delay between batches
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        println!(
            "Finished creating embeddings for existing files in folder: {}",
            folder.display()
        );
        Ok(())
    }

    /// Collect all files in a directory (recursively or not based on config) - public interface
    fn collect_files_static(folder: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();

        let read_dir = std::fs::read_dir(folder)
            .map_err(|e| format!("Failed to read directory {}: {}", folder.display(), e))?;

        for entry in read_dir {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && Self::is_supported_file_static(&path) {
                files.push(path);
            } else if path.is_dir() && recursive {
                // Recursively collect files from subdirectories
                let mut sub_files = Self::collect_files_static(&path, recursive)?;
                files.append(&mut sub_files);
            }
        }

        Ok(files)
    }

    /// Check if a file has a supported extension for embedding creation
    fn is_supported_file_static(path: &PathBuf) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            let ext_lower = extension.to_lowercase();
            matches!(
                ext_lower.as_str(),
                // Text files
                "txt" | "md" | "rst" | "log" |
                // Document files
                "pdf" | "docx" |
                // Spreadsheet files
                "xlsx" |
                // Presentation files
                "pptx" |
                // Korean document files
                "hwp"
            )
        } else {
            false
        }
    }

    /// Pause watching a folder
    pub fn pause_folder(&self, folder: &Path) -> Result<(), String> {
        let mut watched = self.watched.lock().unwrap();
        if let Some(w) = watched.get_mut(folder) {
            println!("Pausing watch for folder: {}", folder.display());
            w.state = WatchState::Paused;
            Ok(())
        } else {
            Err("Folder is not being watched".to_string())
        }
    }

    /// Resume watching a folder
    pub fn resume_folder(&self, folder: &Path) -> Result<(), String> {
        let mut watched = self.watched.lock().unwrap();
        if let Some(w) = watched.get_mut(folder) {
            println!("Resuming watch for folder: {}", folder.display());
            w.state = WatchState::Active;
            Ok(())
        } else {
            Err("Folder is not being watched".to_string())
        }
    }

    /// Remove a folder from being watched
    pub fn remove_folder(&self, folder: &Path) -> Result<(), String> {
        // Remove from watched folders
        let removed = {
            let mut watched = self.watched.lock().unwrap();
            watched.remove(folder).is_some()
        };

        if !removed {
            return Err("Folder is not being watched".to_string());
        }

        // Cancel the watcher task
        {
            let mut watchers = self.watchers.lock().unwrap();
            if let Some(handle) = watchers.remove(folder) {
                handle.abort();
            }
        }

        println!("Stopped watching folder: {}", folder.display());

        Ok(())
    }

    /// List all watched folders
    pub fn list_watched_folders(&self) -> Vec<WatchedFolderInfo> {
        let watched = self.watched.lock().unwrap();
        watched
            .values()
            .map(|folder| {
                let path_str = folder.path.to_string_lossy().to_string();
                let name = folder
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                WatchedFolderInfo {
                    id: path_str.clone(),
                    path: path_str,
                    name,
                    is_active: folder.state == WatchState::Active,
                }
            })
            .collect()
    }

    /// Internal: spawn a watcher for a folder to manage embeddings
    async fn spawn_watcher(&self, folder: PathBuf) -> JoinHandle<()> {
        let watched = self.watched.clone();
        let config = self.config.clone();
        let watch_service = self.clone();

        task::spawn(async move {
            // First, create embeddings for existing files if configured to do so
            let should_process_existing = {
                let config = config.lock().unwrap();
                config.process_existing_files
            };

            if should_process_existing {
                if let Err(e) = watch_service
                    .process_existing_files_for_embeddings(&folder, &config)
                    .await
                {
                    eprintln!(
                        "Warning: Failed to create embeddings for some existing files in {}: {}",
                        folder.display(),
                        e
                    );
                }
            } else {
                println!(
                    "Skipping existing files in {} (disabled in configuration)",
                    folder.display()
                );
            }

            // Now set up the file watcher for new files
            let (tx, mut rx) = mpsc::channel(100);

            let watcher_result = RecommendedWatcher::new(
                move |res: Result<Event, notify::Error>| {
                    if let Ok(event) = res {
                        let _ = tx.blocking_send(event);
                    }
                },
                notify::Config::default(),
            );

            let mut watcher = match watcher_result {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("Failed to create watcher for {}: {}", folder.display(), e);
                    return;
                }
            };

            // Use recursive mode based on config
            let recursive_mode = {
                let config = config.lock().unwrap();
                if config.recursive {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                }
            };

            if let Err(e) = watcher.watch(&folder, recursive_mode) {
                eprintln!("Failed to watch folder {}: {}", folder.display(), e);
                return;
            }

            println!(
                "Started watching folder for embedding updates: {}",
                folder.display()
            );

            while let Some(event) = rx.recv().await {
                if !Self::should_process_event(&watched, &folder) {
                    continue;
                }

                // Check if folder was removed from watch list
                if !Self::is_folder_watched(&watched, &folder) {
                    println!(
                        "Folder {} removed from watch list, stopping watcher",
                        folder.display()
                    );
                    break;
                }

                // Handle file events for embedding management
                match event.kind {
                    EventKind::Create(_) => {
                        for path in event.paths {
                            if path.exists()
                                && path.is_file()
                                && Self::is_supported_file_static(&path)
                            {
                                println!("Creating embedding for new file: {}", path.display());
                                watch_service.upsert_file_to_database(path).await;
                            }
                        }
                    }
                    EventKind::Modify(_) => {
                        for path in event.paths {
                            if path.exists()
                                && path.is_file()
                                && Self::is_supported_file_static(&path)
                            {
                                println!(
                                    "Updating embedding for modified file: {}",
                                    path.display()
                                );
                                watch_service.upsert_file_to_database(path).await;
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        for path in event.paths {
                            if Self::is_supported_file_static(&path) {
                                println!("Removing embedding for deleted file: {}", path.display());
                                watch_service.remove_file_from_database(path).await;
                            }
                        }
                    }
                    _ => {
                        // Ignore other event types
                    }
                }
            }
        })
    }

    /// Check if an event should be processed for the given folder
    fn should_process_event(
        watched: &Arc<Mutex<HashMap<PathBuf, WatchedFolder>>>,
        folder: &PathBuf,
    ) -> bool {
        let watched = watched.lock().unwrap();
        watched
            .get(folder)
            .map(|w| w.state == WatchState::Active)
            .unwrap_or(false)
    }

    /// Check if a folder is still being watched
    fn is_folder_watched(
        watched: &Arc<Mutex<HashMap<PathBuf, WatchedFolder>>>,
        folder: &PathBuf,
    ) -> bool {
        let watched = watched.lock().unwrap();
        watched.contains_key(folder)
    }

    /// Create embedding for a new file
    async fn upsert_file_to_database(&self, path: PathBuf) {
        println!("Creating embedding for file: {}", path.display());

        let database = Arc::clone(&self.database);
        task::spawn(async move {
            // Small delay to ensure file is fully written
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            match ExtractionService::extract_file_content(&path).await {
                Ok(content) => {
                    let file_path = path.to_string_lossy().to_string();
                    if let Err(e) = database.upsert_file(&file_path, &content).await {
                        eprintln!(
                            "❌ Failed to create embedding for {}: {}",
                            path.display(),
                            e
                        );
                    } else {
                        println!("✅ Embedding created for: {}", path.display());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "❌ Failed to extract content for embedding from {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        });
    }

    /// Remove embedding for a deleted file
    async fn remove_file_from_database(&self, path: PathBuf) {
        println!("Removing embedding for deleted file: {}", path.display());

        let database = Arc::clone(&self.database);
        task::spawn(async move {
            let file_path = path.to_string_lossy().to_string();

            if let Err(e) = database.remove_file(&file_path).await {
                eprintln!(
                    "❌ Failed to remove embedding for {}: {}",
                    path.display(),
                    e
                );
            } else {
                println!("✅ Embedding removed for: {}", path.display());
            }
        });
    }

    /// Search for documents based on content
    pub async fn search_documents(
        &self,
        query: &str,
        limit: usize,
    ) -> AppResult<Vec<FileChunkSchema>> {
        println!("Searching documents for query: {}", query);
        // Ensure database is initialized before search
        if let Err(e) = self.database.initialize().await {
            return Err(crate::error::AppError::Other(format!(
                "Failed to initialize database: {}",
                e
            )));
        }

        let results = self.database.search_files(query, limit).await?;
        println!("Found {} documents for query: {}", results.len(), query);
        Ok(results)
    }
}

impl Default for WatchService {
    fn default() -> Self {
        Self::new()
    }
}
