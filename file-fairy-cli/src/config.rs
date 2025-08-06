use crate::error::{CliError, CliResult};
use file_fairy_core::config::AppConfig;
use std::path::PathBuf;

/// CLI-specific configuration and utilities
pub struct CliConfig;

impl CliConfig {
    /// Validates that a model file exists and is readable
    pub fn validate_model_path(path: &PathBuf) -> CliResult<()> {
        if !path.exists() {
            return Err(CliError::file_not_found(path.clone()));
        }

        if path.is_dir() {
            return Err(CliError::invalid_argument(format!(
                "Model path is a directory, expected a file: {}",
                path.display()
            )));
        }

        Ok(())
    }

    /// Creates an AppConfig with validation
    pub fn create_app_config(
        model_path: PathBuf,
        max_tokens: u32,
        threads: u32,
    ) -> CliResult<AppConfig> {
        Self::validate_model_path(&model_path)?;

        let config = AppConfig::new()
            .with_model_path(model_path)
            .with_max_tokens(max_tokens)
            .with_threads(threads);

        config.validate().map_err(CliError::config)?;

        Ok(config)
    }
}
