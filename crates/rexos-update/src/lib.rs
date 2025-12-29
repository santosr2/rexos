//! OTA Update system for RexOS
//!
//! Provides secure over-the-air updates with delta patching support.
//! Updates are cryptographically signed and verified before installation.
//!
//! # Features
//!
//! - Secure update verification using Ed25519 signatures
//! - Delta updates for bandwidth efficiency
//! - Rollback support with A/B partitioning
//! - Background download with resume capability
//! - Update channels (stable, beta, nightly)

mod checker;
mod downloader;
mod installer;
mod manifest;
mod verification;

use std::path::{Path, PathBuf};
use thiserror::Error;

pub use checker::{UpdateChannel, UpdateChecker, UpdateInfo};
pub use downloader::{DownloadProgress, DownloadState, UpdateDownloader};
pub use installer::{InstallProgress, InstallResult, UpdateInstaller};
pub use manifest::{FileEntry, ReleaseNotes, UpdateManifest};
pub use verification::{CertificateVerifier, HashVerifier, SignatureVerifier, VerificationError};

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("No update available")]
    NoUpdate,

    #[error("Update check failed: {0}")]
    CheckFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Installation failed: {0}")]
    InstallFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Insufficient space: need {needed} bytes, have {available}")]
    InsufficientSpace { needed: u64, available: u64 },

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

/// Update system configuration
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// Update server base URL
    pub server_url: String,

    /// Update channel
    pub channel: UpdateChannel,

    /// Path to store downloaded updates
    pub download_dir: PathBuf,

    /// Path to staging directory
    pub staging_dir: PathBuf,

    /// Public key for signature verification (hex-encoded)
    pub public_key: String,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Auto-install updates
    pub auto_install: bool,

    /// Check for updates on boot
    pub check_on_boot: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            server_url: "https://updates.rexos.io".to_string(),
            channel: UpdateChannel::Stable,
            download_dir: PathBuf::from("/tmp/rexos-updates"),
            staging_dir: PathBuf::from("/tmp/rexos-staging"),
            public_key: String::new(),
            max_retries: 3,
            auto_install: false,
            check_on_boot: true,
        }
    }
}

/// Main update manager
pub struct UpdateManager {
    config: UpdateConfig,
    checker: UpdateChecker,
    downloader: UpdateDownloader,
    installer: UpdateInstaller,
}

impl UpdateManager {
    /// Create a new update manager
    pub fn new(config: UpdateConfig) -> Self {
        let checker = UpdateChecker::new(config.server_url.clone(), config.channel);

        let downloader = UpdateDownloader::new(config.download_dir.clone(), config.max_retries);

        let installer = UpdateInstaller::new(config.staging_dir.clone());

        Self {
            config,
            checker,
            downloader,
            installer,
        }
    }

    /// Check for available updates
    pub async fn check(&self) -> Result<Option<UpdateInfo>, UpdateError> {
        let current_version = self.get_current_version()?;
        self.checker.check(&current_version).await
    }

    /// Download an update
    pub async fn download(&self, update: &UpdateInfo) -> Result<PathBuf, UpdateError> {
        self.downloader.download(update).await
    }

    /// Verify a downloaded update
    ///
    /// Performs two-stage verification:
    /// 1. SHA256 hash verification to ensure file integrity
    /// 2. Ed25519 signature verification to ensure authenticity
    pub fn verify(&self, path: &Path, update: &UpdateInfo) -> Result<(), UpdateError> {
        // First, verify the SHA256 hash for integrity
        HashVerifier::verify_file(path, &update.sha256).map_err(|e| {
            UpdateError::VerificationFailed(format!("Hash verification failed: {}", e))
        })?;

        tracing::debug!("Hash verification passed for {}", path.display());

        // Then verify the Ed25519 signature for authenticity
        let verifier = SignatureVerifier::from_hex(&self.config.public_key)
            .map_err(|e| UpdateError::VerificationFailed(e.to_string()))?;

        verifier.verify_file(path, &update.signature).map_err(|e| {
            UpdateError::VerificationFailed(format!("Signature verification failed: {}", e))
        })?;

        tracing::debug!("Signature verification passed for {}", path.display());

        Ok(())
    }

    /// Install a verified update
    pub async fn install(&self, path: &PathBuf) -> Result<InstallResult, UpdateError> {
        self.installer.install(path).await
    }

    /// Perform full update cycle
    pub async fn update(&self) -> Result<InstallResult, UpdateError> {
        // Check for updates
        let update = self.check().await?.ok_or(UpdateError::NoUpdate)?;

        tracing::info!(
            "Update available: {} -> {}",
            self.get_current_version()?,
            update.version
        );

        // Download
        let path = self.download(&update).await?;
        tracing::info!("Update downloaded to {}", path.display());

        // Verify
        self.verify(&path, &update)?;
        tracing::info!("Update signature verified");

        // Install
        let result = self.install(&path).await?;
        tracing::info!("Update installed successfully");

        Ok(result)
    }

    /// Get current RexOS version
    fn get_current_version(&self) -> Result<String, UpdateError> {
        // Read from /etc/rexos-release or environment
        let version_file = PathBuf::from("/etc/rexos-release");

        if version_file.exists() {
            let contents = std::fs::read_to_string(&version_file)?;
            for line in contents.lines() {
                if line.starts_with("VERSION=") {
                    return Ok(line
                        .trim_start_matches("VERSION=")
                        .trim_matches('"')
                        .to_string());
                }
            }
        }

        // Fallback to compile-time version
        Ok(env!("CARGO_PKG_VERSION").to_string())
    }

    /// Rollback to previous version
    pub async fn rollback(&self) -> Result<(), UpdateError> {
        self.installer.rollback().await
    }

    /// Get download progress
    pub fn download_progress(&self) -> Option<DownloadProgress> {
        self.downloader.progress()
    }

    /// Get install progress
    pub fn install_progress(&self) -> Option<InstallProgress> {
        self.installer.progress()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_config_default() {
        let config = UpdateConfig::default();
        assert_eq!(config.channel, UpdateChannel::Stable);
        assert!(!config.auto_install);
    }

    #[test]
    fn test_update_manager_creation() {
        let config = UpdateConfig::default();
        let _manager = UpdateManager::new(config);
    }
}
