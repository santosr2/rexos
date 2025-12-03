//! Update availability checking

use crate::{UpdateError, UpdateManifest};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Update channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    /// Stable releases only
    #[default]
    Stable,
    /// Beta releases
    Beta,
    /// Nightly builds
    Nightly,
}

impl UpdateChannel {
    /// Get channel as URL path component
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Beta => "beta",
            UpdateChannel::Nightly => "nightly",
        }
    }
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Version string
    pub version: String,

    /// Channel this update is from
    pub channel: UpdateChannel,

    /// Download URL
    pub download_url: String,

    /// File size in bytes
    pub size: u64,

    /// SHA256 hash of the update file
    pub sha256: String,

    /// Ed25519 signature (hex-encoded)
    pub signature: String,

    /// Release notes
    pub release_notes: Option<String>,

    /// Release date (ISO 8601)
    pub release_date: String,

    /// Whether this is a critical security update
    pub critical: bool,

    /// Minimum version required (for delta updates)
    pub min_version: Option<String>,

    /// Full manifest URL
    pub manifest_url: Option<String>,
}

/// Checks for available updates
pub struct UpdateChecker {
    server_url: String,
    channel: UpdateChannel,
    client: reqwest::Client,
}

impl UpdateChecker {
    /// Create a new update checker
    pub fn new(server_url: String, channel: UpdateChannel) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(format!("RexOS/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            server_url,
            channel,
            client,
        }
    }

    /// Check for available updates
    pub async fn check(&self, current_version: &str) -> Result<Option<UpdateInfo>, UpdateError> {
        let url = format!(
            "{}/api/v1/updates/{}/latest",
            self.server_url,
            self.channel.as_str()
        );

        tracing::debug!("Checking for updates at {}", url);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("current_version", current_version),
                ("arch", std::env::consts::ARCH),
            ])
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(UpdateError::CheckFailed(format!(
                "Server returned {}",
                response.status()
            )));
        }

        let update: UpdateInfo = response.json().await?;

        // Compare versions
        if Self::is_newer(&update.version, current_version) {
            Ok(Some(update))
        } else {
            Ok(None)
        }
    }

    /// Check all channels for updates
    pub async fn check_all_channels(
        &self,
        current_version: &str,
    ) -> Result<Vec<UpdateInfo>, UpdateError> {
        let channels = [
            UpdateChannel::Stable,
            UpdateChannel::Beta,
            UpdateChannel::Nightly,
        ];
        let mut updates = Vec::new();

        for channel in channels {
            let url = format!(
                "{}/api/v1/updates/{}/latest",
                self.server_url,
                channel.as_str()
            );

            let response = self
                .client
                .get(&url)
                .query(&[("current_version", current_version)])
                .send()
                .await;

            if let Ok(resp) = response
                && resp.status().is_success()
                && let Ok(update) = resp.json::<UpdateInfo>().await
                && Self::is_newer(&update.version, current_version)
            {
                updates.push(update);
            }
        }

        Ok(updates)
    }

    /// Get full manifest for an update
    pub async fn get_manifest(&self, update: &UpdateInfo) -> Result<UpdateManifest, UpdateError> {
        let url = update
            .manifest_url
            .clone()
            .ok_or_else(|| UpdateError::InvalidManifest("No manifest URL".into()))?;

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(UpdateError::CheckFailed(format!(
                "Failed to fetch manifest: {}",
                response.status()
            )));
        }

        let manifest: UpdateManifest = response
            .json()
            .await
            .map_err(|e| UpdateError::InvalidManifest(e.to_string()))?;

        Ok(manifest)
    }

    /// Compare version strings (semver-aware)
    pub fn is_newer(new_version: &str, current_version: &str) -> bool {
        match (
            semver::Version::parse(new_version.trim_start_matches('v')),
            semver::Version::parse(current_version.trim_start_matches('v')),
        ) {
            (Ok(new), Ok(current)) => new > current,
            _ => {
                // Fallback to string comparison
                new_version.cmp(current_version) == Ordering::Greater
            }
        }
    }

    /// Get release history
    pub async fn get_releases(&self, limit: usize) -> Result<Vec<UpdateInfo>, UpdateError> {
        let url = format!(
            "{}/api/v1/updates/{}/history?limit={}",
            self.server_url,
            self.channel.as_str(),
            limit
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(UpdateError::CheckFailed(format!(
                "Failed to fetch release history: {}",
                response.status()
            )));
        }

        let releases: Vec<UpdateInfo> = response.json().await?;
        Ok(releases)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(UpdateChecker::is_newer("1.0.1", "1.0.0"));
        assert!(UpdateChecker::is_newer("1.1.0", "1.0.5"));
        assert!(UpdateChecker::is_newer("2.0.0", "1.9.9"));
        assert!(!UpdateChecker::is_newer("1.0.0", "1.0.0"));
        assert!(!UpdateChecker::is_newer("1.0.0", "1.0.1"));
    }

    #[test]
    fn test_version_with_v_prefix() {
        assert!(UpdateChecker::is_newer("v1.0.1", "v1.0.0"));
        assert!(UpdateChecker::is_newer("v1.0.1", "1.0.0"));
    }

    #[test]
    fn test_channel_str() {
        assert_eq!(UpdateChannel::Stable.as_str(), "stable");
        assert_eq!(UpdateChannel::Beta.as_str(), "beta");
        assert_eq!(UpdateChannel::Nightly.as_str(), "nightly");
    }
}
