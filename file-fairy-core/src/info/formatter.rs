use crate::info::{FeatureInfo, FileFairyInfo, SupportedFormats, SystemInfo, VersionInfo};

/// Provides formatting capabilities for File Fairy information
pub struct InfoFormatter;

impl InfoFormatter {
    /// Formats comprehensive File Fairy information as a human-readable string
    pub fn format_summary(info: &FileFairyInfo) -> String {
        let mut output = String::new();

        output.push_str("🧚 FILE FAIRY INFORMATION\n");
        output.push_str("═══════════════════════════\n\n");

        output.push_str(&Self::format_version_section(&info.version));
        output.push_str(&Self::format_formats_section(&info.supported_formats));
        output.push_str(&Self::format_features_section(&info.features));
        output.push_str(&Self::format_system_section(&info.system));
        output.push_str(&Self::format_usage_examples());

        output
    }

    /// Formats just the supported formats information
    pub fn format_formats(formats: &SupportedFormats) -> String {
        let mut output = String::new();

        output.push_str("📄 SUPPORTED FILE FORMATS\n");
        output.push_str("═══════════════════════════\n\n");

        for (category_name, info) in &formats.categories {
            output.push_str(&format!("🏷️  {}\n", category_name.to_uppercase()));
            output.push_str(&format!("Description: {}\n", info.description));
            output.push_str(&format!(
                "Extractable: {}\n",
                if info.extractable {
                    "✓ Yes"
                } else {
                    "✗ No"
                }
            ));
            output.push_str(&format!("Extensions: {}\n", info.extensions.join(", ")));

            if let Some(ref notes) = info.notes {
                output.push_str(&format!("Notes: {}\n", notes));
            }

            output.push_str("\n");
        }

        output.push_str(&format!(
            "Total supported extensions: {}\n",
            formats.total_extensions
        ));

        output
    }

    /// Formats system and version information
    pub fn format_system(system: &SystemInfo, version: &VersionInfo) -> String {
        let mut output = String::new();

        output.push_str("💻 SYSTEM INFORMATION\n");
        output.push_str("═══════════════════════\n\n");

        output.push_str(&format!("Operating System: {}\n", system.os));
        output.push_str(&format!("Architecture: {}\n", system.arch));
        output.push_str(&format!("CPU Cores: {}\n", system.cpu_cores));

        output.push_str(&format!("File Fairy Version: {}\n", version.version));
        output.push_str(&format!("Build Date: {}\n", version.build_date));

        if let Some(ref hash) = version.git_hash {
            output.push_str(&format!("Git Commit: {}\n", hash));
        }

        output
    }

    /// Formats version information section
    fn format_version_section(version: &VersionInfo) -> String {
        let mut output = String::new();

        output.push_str("📦 VERSION INFO\n");
        output.push_str(&format!("Version: {}\n", version.version));
        output.push_str(&format!("Build Date: {}\n", version.build_date));
        if let Some(ref hash) = version.git_hash {
            output.push_str(&format!("Git Hash: {}\n", hash));
        }

        output
    }

    /// Formats supported formats section with table
    fn format_formats_section(formats: &SupportedFormats) -> String {
        let mut output = String::new();

        output.push_str("\n📄 SUPPORTED FORMATS\n");
        output.push_str(&format!("Total Categories: {}\n", formats.categories.len()));
        output.push_str(&format!(
            "Total Extensions: {}\n\n",
            formats.total_extensions
        ));

        output.push_str("┌─────────────────────┬─────────┬─────────────┬─────────────────────┐\n");
        output.push_str("│ Category            │  Count  │ Extractable │ Extensions          │\n");
        output.push_str("├─────────────────────┼─────────┼─────────────┼─────────────────────┤\n");

        for (category_name, info) in &formats.categories {
            let extractable = if info.extractable { "✓" } else { "✗" };
            let extensions = info.extensions.join(", ");
            let extensions_display = if extensions.len() > 17 {
                format!("{}...", &extensions[..14])
            } else {
                extensions
            };

            output.push_str(&format!(
                "│ {:<19} │ {:>7} │ {:>11} │ {:<19} │\n",
                &category_name[..category_name.len().min(19)],
                info.extensions.len(),
                extractable,
                extensions_display
            ));
        }

        output.push_str("└─────────────────────┴─────────┴─────────────┴─────────────────────┘\n");

        output
    }

    /// Formats features section
    fn format_features_section(features: &FeatureInfo) -> String {
        let mut output = String::new();

        output.push_str("\n⚡ FEATURES\n");

        let feature_list = [
            ("Text Extraction", features.text_extraction),
            ("LLM Integration", features.llm_integration),
            ("Directory Scanning", features.directory_scanning),
            ("Recursive Scanning", features.recursive_scanning),
            ("Symlink Support", features.symlink_support),
            ("Async Processing", features.async_processing),
        ];

        for (name, enabled) in feature_list {
            output.push_str(&format!(
                "{}: {}\n",
                name,
                if enabled {
                    "✓ Enabled"
                } else {
                    "✗ Disabled"
                }
            ));
        }

        output
    }

    /// Formats system information section
    fn format_system_section(system: &SystemInfo) -> String {
        let mut output = String::new();

        output.push_str("\n💻 SYSTEM INFO\n");
        output.push_str(&format!("Operating System: {}\n", system.os));
        output.push_str(&format!("Architecture: {}\n", system.arch));
        output.push_str(&format!("CPU Cores: {}\n", system.cpu_cores));

        output
    }

    /// Formats usage examples section
    fn format_usage_examples() -> String {
        let mut output = String::new();

        output.push_str("\n💡 USAGE EXAMPLES\n");
        output.push_str("• Scan current directory: file-fairy scan .\n");
        output.push_str("• Scan recursively: file-fairy scan . --recursive\n");
        output.push_str("• Generate filename: file-fairy suggest document.pdf\n");
        output.push_str("• Show supported formats: file-fairy info formats\n");
        output.push_str("• Show system info: file-fairy info system\n");

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::info::{FeatureInfo, FileFairyInfo, SupportedFormats, SystemInfo, VersionInfo};

    #[test]
    fn test_format_summary() {
        let info = FileFairyInfo::new();
        let summary = InfoFormatter::format_summary(&info);

        assert!(summary.contains("FILE FAIRY INFORMATION"));
        assert!(summary.contains("VERSION INFO"));
        assert!(summary.contains("SUPPORTED FORMATS"));
        assert!(summary.contains("FEATURES"));
        assert!(summary.contains("SYSTEM INFO"));
        assert!(summary.contains("USAGE EXAMPLES"));
    }

    #[test]
    fn test_format_formats() {
        let formats = SupportedFormats::new();
        let output = InfoFormatter::format_formats(&formats);

        assert!(output.contains("SUPPORTED FILE FORMATS"));
        assert!(output.contains("DOCUMENTS"));
        assert!(output.contains("Total supported extensions"));
    }

    #[test]
    fn test_format_system() {
        let system = SystemInfo::new();
        let version = VersionInfo::new();
        let output = InfoFormatter::format_system(&system, &version);

        assert!(output.contains("SYSTEM INFORMATION"));
        assert!(output.contains("Operating System"));
        assert!(output.contains("File Fairy Version"));
    }

    #[test]
    fn test_format_features_section() {
        let features = FeatureInfo::new();
        let output = InfoFormatter::format_features_section(&features);

        assert!(output.contains("FEATURES"));
        assert!(output.contains("Text Extraction: ✓ Enabled"));
        assert!(output.contains("LLM Integration: ✓ Enabled"));
    }
}
