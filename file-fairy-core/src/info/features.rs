use serde::{Deserialize, Serialize};

/// Feature capabilities of File Fairy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    /// Text extraction capabilities
    pub text_extraction: bool,
    /// LLM integration for filename suggestions
    pub llm_integration: bool,
    /// Directory scanning
    pub directory_scanning: bool,
    /// Recursive directory processing
    pub recursive_scanning: bool,
    /// Symbolic link handling
    pub symlink_support: bool,
    /// Async processing
    pub async_processing: bool,
}

impl FeatureInfo {
    /// Creates new feature information with current capabilities
    pub fn new() -> Self {
        Self {
            text_extraction: true,
            llm_integration: true,
            directory_scanning: true,
            recursive_scanning: true,
            symlink_support: true,
            async_processing: true,
        }
    }

    /// Returns a list of enabled features
    pub fn enabled_features(&self) -> Vec<&'static str> {
        let mut features = Vec::new();

        if self.text_extraction {
            features.push("Text Extraction");
        }
        if self.llm_integration {
            features.push("LLM Integration");
        }
        if self.directory_scanning {
            features.push("Directory Scanning");
        }
        if self.recursive_scanning {
            features.push("Recursive Scanning");
        }
        if self.symlink_support {
            features.push("Symlink Support");
        }
        if self.async_processing {
            features.push("Async Processing");
        }

        features
    }

    /// Returns a list of disabled features
    pub fn disabled_features(&self) -> Vec<&'static str> {
        let mut features = Vec::new();

        if !self.text_extraction {
            features.push("Text Extraction");
        }
        if !self.llm_integration {
            features.push("LLM Integration");
        }
        if !self.directory_scanning {
            features.push("Directory Scanning");
        }
        if !self.recursive_scanning {
            features.push("Recursive Scanning");
        }
        if !self.symlink_support {
            features.push("Symlink Support");
        }
        if !self.async_processing {
            features.push("Async Processing");
        }

        features
    }

    /// Checks if a specific feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature.to_lowercase().as_str() {
            "text_extraction" | "text extraction" => self.text_extraction,
            "llm_integration" | "llm integration" => self.llm_integration,
            "directory_scanning" | "directory scanning" => self.directory_scanning,
            "recursive_scanning" | "recursive scanning" => self.recursive_scanning,
            "symlink_support" | "symlink support" => self.symlink_support,
            "async_processing" | "async processing" => self.async_processing,
            _ => false,
        }
    }
}

impl Default for FeatureInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_info_creation() {
        let features = FeatureInfo::new();
        assert!(features.text_extraction);
        assert!(features.llm_integration);
        assert!(features.directory_scanning);
        assert!(features.recursive_scanning);
        assert!(features.symlink_support);
        assert!(features.async_processing);
    }

    #[test]
    fn test_enabled_features_list() {
        let features = FeatureInfo::new();
        let enabled = features.enabled_features();
        assert_eq!(enabled.len(), 6);
        assert!(enabled.contains(&"Text Extraction"));
        assert!(enabled.contains(&"LLM Integration"));
    }

    #[test]
    fn test_feature_enabled_check() {
        let features = FeatureInfo::new();
        assert!(features.is_feature_enabled("text_extraction"));
        assert!(features.is_feature_enabled("Text Extraction"));
        assert!(!features.is_feature_enabled("unknown_feature"));
    }

    #[test]
    fn test_disabled_features_with_partial_disable() {
        let mut features = FeatureInfo::new();
        features.llm_integration = false;
        features.symlink_support = false;

        let disabled = features.disabled_features();
        assert_eq!(disabled.len(), 2);
        assert!(disabled.contains(&"LLM Integration"));
        assert!(disabled.contains(&"Symlink Support"));

        let enabled = features.enabled_features();
        assert_eq!(enabled.len(), 4);
    }
}
