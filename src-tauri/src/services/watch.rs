use crate::services::{DocumentEmbedding, ExtractionService, VectorDbService};
use crate::types::{WatchState, WatchedFolder, WatchedFolderInfo};
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
    vector_db: Arc<VectorDbService>,
}

impl WatchService {
    /// Create a new watch service with default configuration
    pub fn new() -> Self {
        Self::with_config(WatchConfig::default())
    }

    /// Create a new watch service with custom configuration
    pub fn with_config(config: WatchConfig) -> Self {
        let vector_db = Arc::new(VectorDbService::default());
        Self {
            watched: Arc::new(Mutex::new(HashMap::new())),
            watchers: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(config)),
            vector_db,
        }
    }

    /// Create a new watch service with custom vector database path
    pub fn with_vector_db(config: WatchConfig, db_path: PathBuf) -> Self {
        let vector_db = Arc::new(VectorDbService::new(db_path));
        Self {
            watched: Arc::new(Mutex::new(HashMap::new())),
            watchers: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(Mutex::new(config)),
            vector_db,
        }
    }

    /// Initialize the watch service and vector database
    pub async fn initialize(&self) -> Result<(), String> {
        self.vector_db.initialize().await.map_err(|e| e.to_string())
    }

    /// Update the watch configuration
    pub fn update_config(&self, config: WatchConfig) -> Result<(), String> {
        let mut current_config = self.config.lock().map_err(|_| "Failed to lock config")?;
        *current_config = config;
        Ok(())
    }

    /// Get current watch configuration
    pub fn get_config(&self) -> Result<WatchConfig, String> {
        let config = self.config.lock().map_err(|_| "Failed to lock config")?;
        Ok(config.clone())
    }

    /// Register a folder to watch for embedding-based semantic search
    pub async fn register_folder(&self, folder: &Path) -> Result<(), String> {
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
                    watch_service.create_embedding_for_file(path).await;
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
                                watch_service.create_embedding_for_file(path).await;
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        for path in event.paths {
                            if Self::is_supported_file_static(&path) {
                                println!("Removing embedding for deleted file: {}", path.display());
                                watch_service.remove_embedding_for_file(path).await;
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
                                watch_service.update_embedding_for_file(path).await;
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
    async fn create_embedding_for_file(&self, path: PathBuf) {
        println!("Creating embedding for file: {}", path.display());

        let vector_db = Arc::clone(&self.vector_db);
        task::spawn(async move {
            // Small delay to ensure file is fully written
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            match ExtractionService::extract_file_content(&path).await {
                Ok(content) => {
                    println!(
                        "✅ Content extracted for embedding (length: {}): {}",
                        content.len(),
                        path.display()
                    );

                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let directory = path
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let file_extension = path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // For now, use placeholder embedding - in real implementation,
                    // this would be generated by an embedding model
                    let placeholder_embedding = vec![0.5; 384]; // Typical embedding dimension

                    let embedding = DocumentEmbedding::new(
                        path.to_string_lossy().to_string(),
                        file_name,
                        directory,
                        file_extension,
                        content.clone(),
                        placeholder_embedding,
                    );

                    if let Err(e) = vector_db.upsert_embedding(embedding).await {
                        eprintln!("❌ Failed to store embedding for {}: {}", path.display(), e);
                    } else {
                        println!("✅ Embedding stored for: {}", path.display());
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

    /// Update embedding for a modified file
    async fn update_embedding_for_file(&self, path: PathBuf) {
        println!("Updating embedding for modified file: {}", path.display());

        let vector_db = Arc::clone(&self.vector_db);
        task::spawn(async move {
            // Small delay to ensure file modifications are complete
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            match ExtractionService::extract_file_content(&path).await {
                Ok(content) => {
                    println!(
                        "✅ Content extracted for embedding update (length: {}): {}",
                        content.len(),
                        path.display()
                    );

                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let directory = path
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    let file_extension = path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // For now, use placeholder embedding - in real implementation,
                    // this would be generated by an embedding model
                    let placeholder_embedding = vec![0.5; 384]; // Typical embedding dimension

                    let embedding = DocumentEmbedding::new(
                        path.to_string_lossy().to_string(),
                        file_name,
                        directory,
                        file_extension,
                        content.clone(),
                        placeholder_embedding,
                    );

                    if let Err(e) = vector_db.upsert_embedding(embedding).await {
                        eprintln!(
                            "❌ Failed to update embedding for {}: {}",
                            path.display(),
                            e
                        );
                    } else {
                        println!("✅ Embedding updated for: {}", path.display());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "❌ Failed to extract content for embedding update from {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        });
    }

    /// Remove embedding for a deleted file
    async fn remove_embedding_for_file(&self, path: PathBuf) {
        println!("Removing embedding for deleted file: {}", path.display());

        let vector_db = Arc::clone(&self.vector_db);
        task::spawn(async move {
            let file_path = path.to_string_lossy().to_string();

            if let Err(e) = vector_db.delete_embedding(&file_path).await {
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

    /// Clean up old embeddings for files that no longer exist
    pub async fn cleanup_stale_embeddings(&self) -> Result<u64, String> {
        self.vector_db
            .cleanup_stale_embeddings()
            .await
            .map_err(|e| e.to_string())
    }

    /// Get vector database statistics
    pub async fn get_embedding_stats(&self) -> Result<HashMap<String, u64>, String> {
        self.vector_db.get_stats().await.map_err(|e| e.to_string())
    }

    /// Search for similar documents based on content
    pub async fn search_similar_documents(
        &self,
        _query_text: &str,
        limit: usize,
    ) -> Result<Vec<DocumentEmbedding>, String> {
        // For now, use a placeholder embedding - in real implementation,
        // this would be generated by the same embedding model used for documents
        let query_embedding = vec![0.5; 384]; // Typical embedding dimension

        self.vector_db
            .search_similar(query_embedding, limit)
            .await
            .map_err(|e| e.to_string())
    }

    /// Get all embeddings for documents in a specific directory
    pub async fn get_documents_in_directory(
        &self,
        directory_path: &str,
    ) -> Result<Vec<DocumentEmbedding>, String> {
        self.vector_db
            .get_embeddings_by_directory(directory_path)
            .await
            .map_err(|e| e.to_string())
    }

    /// Get a specific document embedding by file path
    pub async fn get_document_embedding(
        &self,
        file_path: &str,
    ) -> Result<Option<DocumentEmbedding>, String> {
        self.vector_db
            .get_embedding(file_path)
            .await
            .map_err(|e| e.to_string())
    }
}

impl Default for WatchService {
    fn default() -> Self {
        Self::new()
    }
}
