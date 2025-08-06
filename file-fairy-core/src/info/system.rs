use serde::{Deserialize, Serialize};

/// System and runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system
    pub os: String,
    /// CPU architecture
    pub arch: String,
    /// Number of available CPU cores
    pub cpu_cores: usize,
}

impl SystemInfo {
    /// Creates new system information from current environment
    pub fn new() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_cores: num_cpus::get(),
        }
    }

    /// Returns a human-readable description of the system
    pub fn description(&self) -> String {
        format!("{} {} ({} cores)", self.os, self.arch, self.cpu_cores)
    }

    /// Checks if the system is a specific OS
    pub fn is_os(&self, os: &str) -> bool {
        self.os.eq_ignore_ascii_case(os)
    }

    /// Checks if the system is Unix-like (Linux, macOS, etc.)
    pub fn is_unix_like(&self) -> bool {
        matches!(self.os.as_str(), "linux" | "macos" | "freebsd" | "openbsd" | "netbsd")
    }

    /// Checks if the system is Windows
    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }

    /// Returns whether the system likely supports parallel processing well
    pub fn supports_parallel_processing(&self) -> bool {
        self.cpu_cores > 1
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_creation() {
        let system = SystemInfo::new();
        assert!(!system.os.is_empty());
        assert!(!system.arch.is_empty());
        assert!(system.cpu_cores > 0);
    }

    #[test]
    fn test_system_description() {
        let system = SystemInfo::new();
        let desc = system.description();
        assert!(desc.contains(&system.os));
        assert!(desc.contains(&system.arch));
        assert!(desc.contains("cores"));
    }

    #[test]
    fn test_os_detection() {
        let system = SystemInfo::new();
        
        // Test case-insensitive matching
        if system.os == "macos" {
            assert!(system.is_os("macos"));
            assert!(system.is_os("MacOS"));
            assert!(system.is_unix_like());
            assert!(!system.is_windows());
        } else if system.os == "linux" {
            assert!(system.is_os("linux"));
            assert!(system.is_os("Linux"));
            assert!(system.is_unix_like());
            assert!(!system.is_windows());
        } else if system.os == "windows" {
            assert!(system.is_os("windows"));
            assert!(system.is_os("Windows"));
            assert!(!system.is_unix_like());
            assert!(system.is_windows());
        }
    }

    #[test]
    fn test_parallel_processing_support() {
        let system = SystemInfo::new();
        // Most modern systems should support parallel processing
        assert!(system.supports_parallel_processing());
    }
}
