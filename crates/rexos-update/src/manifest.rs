//! Update manifest format

use serde::{Deserialize, Serialize};

/// Update manifest containing all update metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    /// Manifest format version
    pub manifest_version: u32,

    /// RexOS version being installed
    pub version: String,

    /// Build timestamp
    pub build_date: String,

    /// Build number
    pub build_number: Option<u64>,

    /// Git commit hash
    pub commit: Option<String>,

    /// Minimum version required to apply this update
    pub min_version: Option<String>,

    /// Maximum version (for rollback detection)
    pub max_version: Option<String>,

    /// Target architecture
    pub architecture: String,

    /// Target devices (empty = all)
    #[serde(default)]
    pub target_devices: Vec<String>,

    /// Release notes
    pub release_notes: ReleaseNotes,

    /// Files in this update
    #[serde(default)]
    pub files: Vec<FileEntry>,

    /// Files to remove
    #[serde(default)]
    pub remove: Vec<String>,

    /// Pre-install scripts
    #[serde(default)]
    pub pre_install: Vec<ScriptEntry>,

    /// Post-install scripts
    #[serde(default)]
    pub post_install: Vec<ScriptEntry>,

    /// Dependencies (other packages required)
    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    /// Conflicts (packages that must be removed)
    #[serde(default)]
    pub conflicts: Vec<String>,

    /// Whether this update requires a reboot
    #[serde(default)]
    pub requires_reboot: bool,

    /// Whether this is a critical security update
    #[serde(default)]
    pub critical: bool,

    /// Size of the compressed update
    pub compressed_size: u64,

    /// Size after extraction
    pub uncompressed_size: u64,

    /// SHA256 of the compressed package
    pub sha256: String,

    /// Ed25519 signature of the manifest
    pub signature: String,
}

/// Release notes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseNotes {
    /// Title/headline
    pub title: String,

    /// Summary (one paragraph)
    pub summary: String,

    /// Full description (markdown)
    pub description: String,

    /// Breaking changes
    #[serde(default)]
    pub breaking_changes: Vec<String>,

    /// New features
    #[serde(default)]
    pub features: Vec<String>,

    /// Bug fixes
    #[serde(default)]
    pub fixes: Vec<String>,

    /// Known issues
    #[serde(default)]
    pub known_issues: Vec<String>,

    /// Upgrade instructions
    pub upgrade_notes: Option<String>,
}

/// A file entry in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Path relative to root
    pub path: String,

    /// File size in bytes
    pub size: u64,

    /// SHA256 hash
    pub sha256: String,

    /// Unix permissions (octal)
    pub mode: Option<String>,

    /// Owner (user:group)
    pub owner: Option<String>,

    /// File type
    #[serde(default)]
    pub file_type: FileType,

    /// Action (add, update, config)
    #[serde(default)]
    pub action: FileAction,
}

/// File types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    #[default]
    Regular,
    Directory,
    Symlink,
    Config,
}

/// File actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileAction {
    /// New file
    #[default]
    Add,
    /// Updated file
    Update,
    /// Configuration file (preserve user changes)
    Config,
    /// Symlink
    Link,
}

/// A script to run during installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptEntry {
    /// Script name
    pub name: String,

    /// Script content (inline) or path
    pub script: String,

    /// Run as root
    #[serde(default = "default_true")]
    pub root: bool,

    /// Continue on failure
    #[serde(default)]
    pub ignore_errors: bool,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u32 {
    60
}

/// Package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Package name
    pub name: String,

    /// Version constraint (e.g., ">=1.0.0")
    pub version: Option<String>,

    /// Whether this is optional
    #[serde(default)]
    pub optional: bool,
}

impl UpdateManifest {
    /// Create a new empty manifest
    pub fn new(version: &str) -> Self {
        Self {
            manifest_version: 1,
            version: version.to_string(),
            build_date: String::new(),
            build_number: None,
            commit: None,
            min_version: None,
            max_version: None,
            architecture: std::env::consts::ARCH.to_string(),
            target_devices: Vec::new(),
            release_notes: ReleaseNotes::default(),
            files: Vec::new(),
            remove: Vec::new(),
            pre_install: Vec::new(),
            post_install: Vec::new(),
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            requires_reboot: false,
            critical: false,
            compressed_size: 0,
            uncompressed_size: 0,
            sha256: String::new(),
            signature: String::new(),
        }
    }

    /// Add a file to the manifest
    pub fn add_file(&mut self, entry: FileEntry) {
        self.uncompressed_size += entry.size;
        self.files.push(entry);
    }

    /// Mark a file for removal
    pub fn remove_file(&mut self, path: &str) {
        self.remove.push(path.to_string());
    }

    /// Validate manifest
    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() {
            return Err("Version is required".into());
        }

        if self.sha256.is_empty() {
            return Err("SHA256 hash is required".into());
        }

        if self.signature.is_empty() {
            return Err("Signature is required".into());
        }

        if self.files.is_empty() && self.remove.is_empty() {
            return Err("Manifest must contain files or removals".into());
        }

        Ok(())
    }

    /// Get total file count
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if device is supported
    pub fn supports_device(&self, device_id: &str) -> bool {
        self.target_devices.is_empty()
            || self
                .target_devices
                .iter()
                .any(|d| d == device_id || d == "*")
    }

    /// Check if current version can update to this version
    pub fn can_update_from(&self, current_version: &str) -> bool {
        if let Some(ref min) = self.min_version
            && let (Ok(min_ver), Ok(curr_ver)) = (
                semver::Version::parse(min),
                semver::Version::parse(current_version),
            )
            && curr_ver < min_ver
        {
            return false;
        }

        if let Some(ref max) = self.max_version
            && let (Ok(max_ver), Ok(curr_ver)) = (
                semver::Version::parse(max),
                semver::Version::parse(current_version),
            )
            && curr_ver >= max_ver
        {
            return false;
        }

        true
    }
}

impl ReleaseNotes {
    /// Create empty release notes
    pub fn new(title: &str, summary: &str) -> Self {
        Self {
            title: title.to_string(),
            summary: summary.to_string(),
            description: String::new(),
            breaking_changes: Vec::new(),
            features: Vec::new(),
            fixes: Vec::new(),
            known_issues: Vec::new(),
            upgrade_notes: None,
        }
    }

    /// Format as markdown
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n{}\n\n", self.title, self.summary);

        if !self.description.is_empty() {
            md.push_str(&format!("{}\n\n", self.description));
        }

        if !self.breaking_changes.is_empty() {
            md.push_str("## Breaking Changes\n\n");
            for item in &self.breaking_changes {
                md.push_str(&format!("- {}\n", item));
            }
            md.push('\n');
        }

        if !self.features.is_empty() {
            md.push_str("## New Features\n\n");
            for item in &self.features {
                md.push_str(&format!("- {}\n", item));
            }
            md.push('\n');
        }

        if !self.fixes.is_empty() {
            md.push_str("## Bug Fixes\n\n");
            for item in &self.fixes {
                md.push_str(&format!("- {}\n", item));
            }
            md.push('\n');
        }

        if !self.known_issues.is_empty() {
            md.push_str("## Known Issues\n\n");
            for item in &self.known_issues {
                md.push_str(&format!("- {}\n", item));
            }
            md.push('\n');
        }

        if let Some(ref notes) = self.upgrade_notes {
            md.push_str(&format!("## Upgrade Notes\n\n{}\n", notes));
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_new() {
        let manifest = UpdateManifest::new("1.0.0");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.manifest_version, 1);
    }

    #[test]
    fn test_manifest_add_file() {
        let mut manifest = UpdateManifest::new("1.0.0");

        manifest.add_file(FileEntry {
            path: "/usr/bin/test".to_string(),
            size: 1024,
            sha256: "abc123".to_string(),
            mode: Some("0755".to_string()),
            owner: None,
            file_type: FileType::Regular,
            action: FileAction::Add,
        });

        assert_eq!(manifest.file_count(), 1);
        assert_eq!(manifest.uncompressed_size, 1024);
    }

    #[test]
    fn test_device_support() {
        let mut manifest = UpdateManifest::new("1.0.0");

        // Empty target_devices means all devices supported
        assert!(manifest.supports_device("rg353m"));

        // Specific devices
        manifest.target_devices = vec!["rg353m".to_string(), "rg353v".to_string()];
        assert!(manifest.supports_device("rg353m"));
        assert!(!manifest.supports_device("rg35xx"));
    }

    #[test]
    fn test_release_notes_markdown() {
        let mut notes = ReleaseNotes::new("Version 1.0", "First stable release");
        notes.features.push("Added feature A".to_string());
        notes.fixes.push("Fixed bug B".to_string());

        let md = notes.to_markdown();
        assert!(md.contains("# Version 1.0"));
        assert!(md.contains("Added feature A"));
        assert!(md.contains("Fixed bug B"));
    }
}
