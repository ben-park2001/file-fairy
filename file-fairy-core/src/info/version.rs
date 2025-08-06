use serde::{Deserialize, Serialize};

/// Version and build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// File Fairy version
    pub version: String,
    /// Build date
    pub build_date: String,
    /// Git commit hash (if available)
    pub git_hash: Option<String>,
}

impl VersionInfo {
    /// Creates new version information from environment variables
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_date: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            git_hash: option_env!("GIT_HASH").map(|s| s.to_string()),
        }
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_creation() {
        let version = VersionInfo::new();
        assert!(!version.version.is_empty());
        assert!(!version.build_date.is_empty());
    }

    #[test]
    fn test_version_format() {
        let version = VersionInfo::new();
        // Version should follow semantic versioning pattern
        assert!(version.version.contains('.'));
        // Build date should contain UTC
        assert!(version.build_date.contains("UTC"));
    }
}
