use super::{action::OrganizeAction, options::OrganizeOptions, results::OrganizeResults};
use crate::{config::AppConfig, error::CoreError, file_category::FileCategory};
use anyhow::Result;
use file_fairy_extractors::extractor_from_file_path;
use file_fairy_llm::{FileLlm, LlmConfig, LocalLlmClient};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};
use tokio::fs;

/// The main file organizer service that coordinates scanning, analysis, and organization
pub struct FileOrganizeService {
    llm_client: LocalLlmClient,
}

impl FileOrganizeService {
    /// Creates a new file organize service
    pub fn new(config: AppConfig) -> Result<Self, CoreError> {
        config.validate().map_err(CoreError::generic)?;

        let llm_config = LlmConfig::new(config.model_path)
            .with_max_tokens(config.max_tokens)
            .with_context_size(config.context_size)
            .with_threads(config.threads)
            .with_seed(config.seed);

        let llm_client = LocalLlmClient::new(llm_config)
            .map_err(|e| CoreError::generic(format!("Failed to create LLM client: {}", e)))?;

        Ok(Self { llm_client })
    }

    /// Organizes files in the given path with the specified options
    pub async fn organize(
        &self,
        path: &Path,
        options: OrganizeOptions,
    ) -> Result<OrganizeResults, CoreError> {
        self.validate_path(path)?;

        let mut results = OrganizeResults::new(path.to_path_buf(), !options.apply);

        // Scan directory and process files
        let file_processor = FileProcessor::new(&self.llm_client, &options);
        let files = self.discover_files(path, &options).await?;

        for file_path in files {
            file_processor.process_file(&file_path, &mut results).await;
        }

        // Handle user interaction if needed
        if options.interactive && !results.actions.is_empty() {
            let interaction_handler = InteractionHandler::new();
            interaction_handler
                .run_interactive_mode(&mut results)
                .await?;
        }

        // Execute actions if not a dry run
        if options.apply {
            let executor = ActionExecutor::new();
            executor.execute_actions(&mut results).await?;
        }

        Ok(results)
    }

    /// Validates that the input path is suitable for organization
    fn validate_path(&self, path: &Path) -> Result<(), CoreError> {
        if !path.exists() {
            return Err(CoreError::path_not_found(path));
        }

        if !path.is_dir() {
            return Err(CoreError::generic(format!(
                "Path '{}' is not a directory",
                path.display()
            )));
        }

        Ok(())
    }

    /// Discovers all files to be processed based on options
    async fn discover_files(
        &self,
        path: &Path,
        options: &OrganizeOptions,
    ) -> Result<Vec<PathBuf>, CoreError> {
        let file_scanner = FileScanner::new(options);
        file_scanner.scan_directory(path).await
    }
}

/// Handles processing of individual files
struct FileProcessor<'a> {
    llm_client: &'a LocalLlmClient,
    options: &'a OrganizeOptions,
}

impl<'a> FileProcessor<'a> {
    fn new(llm_client: &'a LocalLlmClient, options: &'a OrganizeOptions) -> Self {
        Self {
            llm_client,
            options,
        }
    }

    /// Generates a suggested filename based on file content.
    async fn generate_filename(&self, file_path: &Path) -> Result<String, CoreError> {
        let file_info = FileInfo::from_path(file_path)?;
        let content = self.extract_content(&file_info).await?;

        self.llm_client
            .suggest_filename(&file_info.original_filename, &content)
            .await
            .map_err(CoreError::Llm)
    }

    /// Extracts content from a file using the appropriate extractor.
    async fn extract_content(&self, file_info: &FileInfo) -> Result<String, CoreError> {
        let extractor = extractor_from_file_path(&file_info.path)
            .ok_or_else(|| CoreError::unsupported_file_type(&file_info.path))?;

        extractor
            .extract(&file_info.path)
            .await
            .map_err(|e| CoreError::extractor_error(&file_info.path, Box::new(e)))
    }

    async fn process_file(&self, file_path: &Path, results: &mut OrganizeResults) {
        match self.create_organize_action(file_path).await {
            Ok(Some(action)) => {
                results.add_action(action);
            }
            Ok(None) => {
                // File was skipped (unsupported, too large, etc.)
            }
            Err(e) => {
                results.record_failure(format!("Failed to process {}: {}", file_path.display(), e));
            }
        }
    }

    async fn create_organize_action(
        &self,
        file_path: &Path,
    ) -> Result<Option<OrganizeAction>, CoreError> {
        let category = FileCategory::from_path(file_path);

        // Check if we should process this file
        if !self.should_process_file(file_path, &category).await? {
            return Ok(None);
        }

        let metadata = fs::metadata(file_path).await.map_err(CoreError::Io)?;
        let original_filename = self.extract_filename(file_path);

        // Generate suggested filename using LLM
        let suggested_filename = self.generate_filename(file_path).await?;

        // Determine destination path
        let destination = self.determine_destination_path(file_path, &suggested_filename)?;

        let mut action = OrganizeAction::new(
            file_path.to_path_buf(),
            destination,
            original_filename,
            suggested_filename,
            category,
            metadata.len(),
        );

        // Auto-approve if not in interactive mode
        if !self.options.interactive {
            action.approve();
        }

        Ok(Some(action))
    }

    async fn should_process_file(
        &self,
        file_path: &Path,
        category: &FileCategory,
    ) -> Result<bool, CoreError> {
        // Check if category is extractable
        if !category.is_extractable() {
            return Ok(false);
        }

        // Check if category should be included
        if !self.options.should_include_category(category) {
            return Ok(false);
        }

        // Check file size limits
        let metadata = fs::metadata(file_path).await.map_err(CoreError::Io)?;
        if !self.options.should_process_file_size(metadata.len()) {
            return Ok(false);
        }

        Ok(true)
    }

    fn extract_filename(&self, file_path: &Path) -> String {
        file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    fn determine_destination_path(
        &self,
        source: &Path,
        suggested_filename: &str,
    ) -> Result<PathBuf, CoreError> {
        let target_dir = match &self.options.target_dir {
            Some(dir) => dir.clone(),
            None => source
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
        };

        Ok(target_dir.join(suggested_filename))
    }
}

/// Handles file system scanning to discover files to process
struct FileScanner<'a> {
    options: &'a OrganizeOptions,
}

impl<'a> FileScanner<'a> {
    fn new(options: &'a OrganizeOptions) -> Self {
        Self { options }
    }

    async fn scan_directory(&self, path: &Path) -> Result<Vec<PathBuf>, CoreError> {
        let mut files = Vec::new();
        let mut stack = vec![(path.to_path_buf(), 0)];

        while let Some((current_path, depth)) = stack.pop() {
            if let Some(max_depth) = self.options.max_depth {
                if depth >= max_depth {
                    continue;
                }
            }

            let mut entries = fs::read_dir(&current_path).await.map_err(CoreError::Io)?;

            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();
                let metadata = fs::metadata(&entry_path).await.map_err(CoreError::Io)?;

                if metadata.is_dir() && self.options.recursive {
                    stack.push((entry_path, depth + 1));
                } else if metadata.is_file() {
                    files.push(entry_path);
                }
            }
        }

        Ok(files)
    }
}

/// Handles user interaction for approval of actions
struct InteractionHandler;

impl InteractionHandler {
    fn new() -> Self {
        Self
    }

    async fn run_interactive_mode(&self, results: &mut OrganizeResults) -> Result<(), CoreError> {
        println!("\n🤝 INTERACTIVE MODE");
        println!("Review each suggested change. Choose: [y]es, [n]o, [s]kip, [e]dit, [q]uit\n");

        let total_actions = results.actions.len();
        for i in 0..total_actions {
            self.review_action(&mut results.actions[i], i + 1, total_actions)?;
        }

        Ok(())
    }

    fn review_action(
        &self,
        action: &mut OrganizeAction,
        current: usize,
        total: usize,
    ) -> Result<(), CoreError> {
        self.display_action_info(action, current, total);

        loop {
            let choice = self.get_user_choice()?;

            match self.handle_user_choice(&choice, action)? {
                UserChoiceResult::Continue => break,
                UserChoiceResult::Quit => return Ok(()),
                UserChoiceResult::Retry => continue,
            }
        }

        Ok(())
    }

    fn display_action_info(&self, action: &OrganizeAction, current: usize, total: usize) {
        println!("📁 File {}/{}", current, total);
        println!("   Original: {}", action.original_filename);
        println!("   Suggested: {}", action.suggested_filename);
        println!("   Size: {}", format_file_size(action.size));
        println!("   Action: {}", action.describe_action());
    }

    fn get_user_choice(&self) -> Result<String, CoreError> {
        print!("\nApprove this change? [y/n/s/e/q]: ");
        io::stdout()
            .flush()
            .map_err(|e| CoreError::generic(e.to_string()))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| CoreError::generic(e.to_string()))?;

        Ok(input.trim().to_lowercase())
    }

    fn handle_user_choice(
        &self,
        choice: &str,
        action: &mut OrganizeAction,
    ) -> Result<UserChoiceResult, CoreError> {
        match choice {
            "y" | "yes" => {
                action.approve();
                println!("✅ Approved");
                Ok(UserChoiceResult::Continue)
            }
            "n" | "no" => {
                println!("❌ Rejected");
                Ok(UserChoiceResult::Continue)
            }
            "s" | "skip" => {
                println!("⏭️  Skipped");
                Ok(UserChoiceResult::Continue)
            }
            "e" | "edit" => {
                self.handle_edit_action(action)?;
                Ok(UserChoiceResult::Continue)
            }
            "q" | "quit" => {
                println!("🛑 Quit - no further changes will be made");
                Ok(UserChoiceResult::Quit)
            }
            _ => {
                println!("Please enter y, n, s, e, or q");
                Ok(UserChoiceResult::Retry)
            }
        }
    }

    fn handle_edit_action(&self, action: &mut OrganizeAction) -> Result<(), CoreError> {
        print!("Enter new filename: ");
        io::stdout()
            .flush()
            .map_err(|e| CoreError::generic(e.to_string()))?;

        let mut new_name = String::new();
        io::stdin()
            .read_line(&mut new_name)
            .map_err(|e| CoreError::generic(e.to_string()))?;
        let new_name = new_name.trim();

        if !new_name.is_empty() {
            action.update_suggested_filename(new_name.to_string());
            action.approve();
            println!("✏️  Edited and approved: {}", new_name);
        } else {
            println!("❌ Invalid filename, skipping");
        }

        Ok(())
    }
}

enum UserChoiceResult {
    Continue,
    Quit,
    Retry,
}

/// Executes approved organize actions
struct ActionExecutor;

impl ActionExecutor {
    fn new() -> Self {
        Self
    }

    async fn execute_actions(&self, results: &mut OrganizeResults) -> Result<(), CoreError> {
        println!("\n⚡ EXECUTING APPROVED ACTIONS...\n");

        // Collect indices of approved actions to avoid borrowing issues
        let approved_indices: Vec<usize> = results
            .actions
            .iter()
            .enumerate()
            .filter(|(_, action)| action.approved)
            .map(|(i, _)| i)
            .collect();

        for i in 0..results.actions.len() {
            if !approved_indices.contains(&i) {
                results.record_skip();
                continue;
            }

            match self.execute_single_action(&results.actions[i]).await {
                Ok(_) => {
                    results.record_success();
                    println!("✅ {}", results.actions[i].describe_action());
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to execute {}: {}",
                        results.actions[i].describe_action(),
                        e
                    );
                    results.record_failure(error_msg.clone());
                    println!("❌ {}", error_msg);
                }
            }
        }

        Ok(())
    }

    async fn execute_single_action(&self, action: &OrganizeAction) -> Result<(), CoreError> {
        // Ensure destination directory exists
        if let Some(dest_dir) = action.destination.parent() {
            fs::create_dir_all(dest_dir).await.map_err(CoreError::Io)?;
        }

        // Check if destination already exists
        if action.destination.exists() {
            return Err(CoreError::generic(format!(
                "Destination already exists: {}",
                action.destination.display()
            )));
        }

        // Move/rename the file
        fs::rename(&action.source, &action.destination)
            .await
            .map_err(CoreError::Io)?;

        Ok(())
    }
}

/// Encapsulates validated file information for processing
struct FileInfo {
    path: std::path::PathBuf,
    original_filename: String,
}

impl FileInfo {
    /// Creates FileInfo from a path, performing all necessary validations
    fn from_path(file_path: &Path) -> Result<Self, CoreError> {
        Self::validate_path(file_path)?;

        let category = FileCategory::from_path(file_path);
        if !category.is_extractable() {
            return Err(CoreError::unsupported_file_type(file_path));
        }

        let original_filename = file_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            path: file_path.to_path_buf(),
            original_filename,
        })
    }

    /// Validates that a file path exists and is a regular file
    fn validate_path(file_path: &Path) -> Result<(), CoreError> {
        if !file_path.exists() {
            return Err(CoreError::path_not_found(file_path));
        }

        if file_path.is_dir() {
            return Err(CoreError::unsupported_file_type(file_path));
        }

        Ok(())
    }
}

// Helper function for file size formatting (moved from results.rs to avoid duplication)
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
