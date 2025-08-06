use serde::{Deserialize, Serialize};

use crate::info::{FeatureInfo, InfoFormatter, SupportedFormats, SystemInfo, VersionInfo};

/// Information about File Fairy's capabilities and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFairyInfo {
    /// Version information
    pub version: VersionInfo,
    /// Supported file formats
    pub supported_formats: SupportedFormats,
    /// Feature capabilities
    pub features: FeatureInfo,
    /// System information
    pub system: SystemInfo,
}

impl FileFairyInfo {
    /// Creates comprehensive File Fairy information
    pub fn new() -> Self {
        Self {
            version: VersionInfo::new(),
            supported_formats: SupportedFormats::new(),
            features: FeatureInfo::new(),
            system: SystemInfo::new(),
        }
    }

    /// Formats the information as a human-readable string
    pub fn format_summary(&self) -> String {
        InfoFormatter::format_summary(self)
    }

    /// Formats just the supported formats information
    pub fn format_formats(&self) -> String {
        InfoFormatter::format_formats(&self.supported_formats)
    }

    /// Formats just the system information
    pub fn format_system(&self) -> String {
        InfoFormatter::format_system(&self.system, &self.version)
    }
}

impl Default for FileFairyInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_creation() {
        let info = FileFairyInfo::new();
        assert!(!info.version.version.is_empty());
        assert!(!info.system.os.is_empty());
        assert!(info.supported_formats.total_extensions > 0);
    }

    #[test]
    fn test_default_implementation() {
        let info = FileFairyInfo::default();
        assert!(!info.version.version.is_empty());
    }
}
