//! Update installation with rollback support

use crate::UpdateError;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tar::Archive;

/// Installation progress
#[derive(Debug, Clone)]
pub struct InstallProgress {
    /// Current step description
    pub step: String,
    /// Current step number
    pub current_step: u32,
    /// Total steps
    pub total_steps: u32,
    /// Files processed
    pub files_processed: u32,
    /// Total files
    pub total_files: u32,
}

impl InstallProgress {
    /// Get progress as percentage
    pub fn percent(&self) -> u8 {
        if self.total_steps == 0 {
            0
        } else {
            ((self.current_step as f64 / self.total_steps as f64) * 100.0) as u8
        }
    }
}

/// Installation result
#[derive(Debug)]
pub struct InstallResult {
    /// Version installed
    pub version: String,
    /// Files updated
    pub files_updated: u32,
    /// Files added
    pub files_added: u32,
    /// Files removed
    pub files_removed: u32,
    /// Requires reboot
    pub needs_reboot: bool,
}

/// Installs updates with rollback support
pub struct UpdateInstaller {
    staging_dir: PathBuf,
    backup_dir: PathBuf,
    progress: Arc<Mutex<Option<InstallProgress>>>,
}

impl UpdateInstaller {
    /// Create a new installer
    pub fn new(staging_dir: PathBuf) -> Self {
        let backup_dir = staging_dir
            .parent()
            .unwrap_or(Path::new("/tmp"))
            .join("rexos-backup");

        Self {
            staging_dir,
            backup_dir,
            progress: Arc::new(Mutex::new(None)),
        }
    }

    /// Install an update package
    pub async fn install(&self, package_path: &PathBuf) -> Result<InstallResult, UpdateError> {
        // Initialize progress
        self.set_progress("Preparing installation", 1, 6, 0, 0);

        // Create staging directory
        fs::create_dir_all(&self.staging_dir)?;

        // Step 1: Extract package
        self.set_progress("Extracting update package", 2, 6, 0, 0);
        let files = self.extract_package(package_path)?;

        // Step 2: Verify extracted files
        self.set_progress("Verifying files", 3, 6, 0, files.len() as u32);
        self.verify_extracted_files(&files)?;

        // Step 3: Create backup of current files
        self.set_progress("Creating backup", 4, 6, 0, files.len() as u32);
        self.create_backup(&files)?;

        // Step 4: Apply update
        self.set_progress("Installing files", 5, 6, 0, files.len() as u32);
        let (updated, added, removed) = self.apply_update(&files)?;

        // Step 5: Run post-install scripts
        self.set_progress("Running post-install scripts", 6, 6, 0, 0);
        let needs_reboot = self.run_post_install()?;

        // Clean up staging
        fs::remove_dir_all(&self.staging_dir).ok();

        // Parse version from package name
        let version = package_path
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix("rexos-"))
            .and_then(|s| s.strip_suffix(".tar"))
            .unwrap_or("unknown")
            .to_string();

        Ok(InstallResult {
            version,
            files_updated: updated,
            files_added: added,
            files_removed: removed,
            needs_reboot,
        })
    }

    /// Extract update package to staging directory
    fn extract_package(&self, package_path: &PathBuf) -> Result<Vec<PathBuf>, UpdateError> {
        let file = File::open(package_path)?;
        let gz = GzDecoder::new(BufReader::new(file));
        let mut archive = Archive::new(gz);

        let mut files = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            let dest = self.staging_dir.join(&path);

            // Create parent directories
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            // Extract file
            entry.unpack(&dest)?;
            files.push(path);
        }

        tracing::info!("Extracted {} files to staging", files.len());
        Ok(files)
    }

    /// Verify extracted files match manifest
    fn verify_extracted_files(&self, _files: &[PathBuf]) -> Result<(), UpdateError> {
        // Check for manifest
        let manifest_path = self.staging_dir.join("manifest.json");

        if !manifest_path.exists() {
            tracing::warn!("No manifest found, skipping file verification");
            return Ok(());
        }

        // Read and verify manifest
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| UpdateError::InvalidManifest(e.to_string()))?;

        if let Some(file_hashes) = manifest.get("files").and_then(|f| f.as_object()) {
            for (file, expected_hash) in file_hashes {
                let file_path = self.staging_dir.join(file);

                if file_path.exists() {
                    let actual_hash = self.compute_sha256(&file_path)?;

                    // Avoid if-let chains for MSRV 1.85 compatibility
                    #[allow(clippy::collapsible_if)]
                    if let Some(expected) = expected_hash.as_str() {
                        if actual_hash != expected {
                            return Err(UpdateError::VerificationFailed(format!(
                                "Hash mismatch for {}",
                                file
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Compute SHA256 hash of a file
    fn compute_sha256(&self, path: &PathBuf) -> Result<String, UpdateError> {
        use sha2::{Digest, Sha256};

        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    /// Create backup of files that will be updated
    fn create_backup(&self, files: &[PathBuf]) -> Result<(), UpdateError> {
        // Clean previous backup
        if self.backup_dir.exists() {
            fs::remove_dir_all(&self.backup_dir)?;
        }
        fs::create_dir_all(&self.backup_dir)?;

        let root = PathBuf::from("/");

        for file in files {
            let source = root.join(file);

            if source.exists() {
                let dest = self.backup_dir.join(file);

                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::copy(&source, &dest)?;
            }
        }

        // Write backup manifest
        let manifest = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "files": files.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()
        });

        fs::write(
            self.backup_dir.join("backup-manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )?;

        tracing::info!("Created backup of {} files", files.len());
        Ok(())
    }

    /// Apply the update
    fn apply_update(&self, files: &[PathBuf]) -> Result<(u32, u32, u32), UpdateError> {
        let root = PathBuf::from("/");
        let mut updated = 0u32;
        let mut added = 0u32;

        for (i, file) in files.iter().enumerate() {
            // Skip manifest and metadata files
            if file.to_string_lossy().ends_with("manifest.json")
                || file.to_string_lossy().ends_with(".meta")
            {
                continue;
            }

            let source = self.staging_dir.join(file);
            let dest = root.join(file);

            // Update progress
            {
                let mut progress = self.progress.lock().unwrap();
                if let Some(ref mut p) = *progress {
                    p.files_processed = i as u32 + 1;
                }
            }

            if source.is_dir() {
                fs::create_dir_all(&dest)?;
                continue;
            }

            // Create parent directories
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }

            let existed = dest.exists();

            // Copy file with proper permissions
            fs::copy(&source, &dest)?;

            // Preserve permissions from staging
            if let Ok(metadata) = fs::metadata(&source) {
                #[cfg(unix)]
                {
                    fs::set_permissions(&dest, metadata.permissions())?;
                }
            }

            if existed {
                updated += 1;
            } else {
                added += 1;
            }
        }

        // Handle file removals (from manifest)
        let removed = self.process_removals()?;

        tracing::info!(
            "Applied update: {} updated, {} added, {} removed",
            updated,
            added,
            removed
        );
        Ok((updated, added, removed))
    }

    /// Process file removals from update manifest
    fn process_removals(&self) -> Result<u32, UpdateError> {
        let manifest_path = self.staging_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Ok(0);
        }

        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| UpdateError::InvalidManifest(e.to_string()))?;

        let mut removed = 0u32;
        let root = PathBuf::from("/");

        if let Some(removals) = manifest.get("remove").and_then(|r| r.as_array()) {
            for file in removals {
                if let Some(path_str) = file.as_str() {
                    let path = root.join(path_str);

                    if path.exists() {
                        if path.is_dir() {
                            fs::remove_dir_all(&path)?;
                        } else {
                            fs::remove_file(&path)?;
                        }
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
    }

    /// Run post-install scripts
    fn run_post_install(&self) -> Result<bool, UpdateError> {
        let script_path = self.staging_dir.join("post-install.sh");

        if !script_path.exists() {
            return Ok(false);
        }

        let output = Command::new("sh").arg(&script_path).output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Post-install script failed: {}", stderr);
            return Err(UpdateError::InstallFailed(format!(
                "Post-install script failed: {}",
                stderr
            )));
        }

        // Check if reboot is needed
        let reboot_flag = self.staging_dir.join(".needs-reboot");
        Ok(reboot_flag.exists())
    }

    /// Rollback to previous version
    pub async fn rollback(&self) -> Result<(), UpdateError> {
        if !self.backup_dir.exists() {
            return Err(UpdateError::RollbackFailed("No backup available".into()));
        }

        let manifest_path = self.backup_dir.join("backup-manifest.json");

        if !manifest_path.exists() {
            return Err(UpdateError::RollbackFailed(
                "Backup manifest not found".into(),
            ));
        }

        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)
            .map_err(|e| UpdateError::RollbackFailed(e.to_string()))?;

        let root = PathBuf::from("/");

        if let Some(files) = manifest.get("files").and_then(|f| f.as_array()) {
            for file in files {
                if let Some(path_str) = file.as_str() {
                    let path = PathBuf::from(path_str);
                    let backup = self.backup_dir.join(&path);
                    let dest = root.join(&path);

                    if backup.exists() {
                        if let Some(parent) = dest.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::copy(&backup, &dest)?;
                    }
                }
            }
        }

        tracing::info!("Rollback completed successfully");
        Ok(())
    }

    /// Get current progress
    pub fn progress(&self) -> Option<InstallProgress> {
        self.progress.lock().unwrap().clone()
    }

    /// Set progress
    fn set_progress(&self, step: &str, current: u32, total: u32, files: u32, total_files: u32) {
        let mut progress = self.progress.lock().unwrap();
        *progress = Some(InstallProgress {
            step: step.to_string(),
            current_step: current,
            total_steps: total,
            files_processed: files,
            total_files,
        });
    }
}

// Chrono for timestamps
mod chrono {
    pub struct Utc;

    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }

    pub struct DateTime;

    impl DateTime {
        pub fn to_rfc3339(&self) -> String {
            // Simple timestamp - production would use actual chrono crate
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            format!("{}", now.as_secs())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_progress_percent() {
        let progress = InstallProgress {
            step: "Testing".to_string(),
            current_step: 3,
            total_steps: 6,
            files_processed: 10,
            total_files: 20,
        };

        assert_eq!(progress.percent(), 50);
    }
}
