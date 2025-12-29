//! Mount point management

use crate::StorageError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MountError {
    #[error("Failed to mount {device} at {mount_point}: {reason}")]
    MountFailed {
        device: String,
        mount_point: String,
        reason: String,
    },

    #[error("Failed to unmount {mount_point}: {reason}")]
    UnmountFailed { mount_point: String, reason: String },

    #[error("Mount point busy: {0}")]
    Busy(String),
}

/// Information about a mount point
#[derive(Debug, Clone)]
pub struct MountPoint {
    pub device: String,
    pub mount_point: PathBuf,
    pub filesystem: String,
    pub options: Vec<String>,
}

/// Manages mount operations
pub struct MountManager {
    mounts: HashMap<PathBuf, MountPoint>,
}

impl MountManager {
    pub fn new() -> Self {
        Self {
            mounts: HashMap::new(),
        }
    }

    /// Read current mount points from /proc/mounts
    pub fn refresh(&mut self) -> Result<(), StorageError> {
        self.mounts.clear();

        let contents = fs::read_to_string("/proc/mounts")?;

        for line in contents.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let mount_point = PathBuf::from(parts[1]);
                let options: Vec<String> = parts[3].split(',').map(String::from).collect();

                self.mounts.insert(
                    mount_point.clone(),
                    MountPoint {
                        device: parts[0].to_string(),
                        mount_point,
                        filesystem: parts[2].to_string(),
                        options,
                    },
                );
            }
        }

        Ok(())
    }

    /// Get all current mount points
    pub fn mounts(&self) -> &HashMap<PathBuf, MountPoint> {
        &self.mounts
    }

    /// Check if a path is mounted
    pub fn is_mounted(&self, path: &Path) -> bool {
        self.mounts.contains_key(path)
    }

    /// Get mount point for a path
    pub fn get_mount(&self, path: &Path) -> Option<&MountPoint> {
        self.mounts.get(path)
    }

    /// Mount a device at a mount point
    pub fn mount(
        &mut self,
        device: &str,
        mount_point: &Path,
        filesystem: Option<&str>,
        options: &[&str],
    ) -> Result<(), MountError> {
        // Create mount point if it doesn't exist
        if !mount_point.exists() {
            fs::create_dir_all(mount_point).map_err(|e| MountError::MountFailed {
                device: device.to_string(),
                mount_point: mount_point.display().to_string(),
                reason: e.to_string(),
            })?;
        }

        let mut cmd = Command::new("mount");

        if let Some(fs) = filesystem {
            cmd.args(["-t", fs]);
        }

        if !options.is_empty() {
            cmd.args(["-o", &options.join(",")]);
        }

        cmd.arg(device);
        cmd.arg(mount_point);

        let output = cmd.output().map_err(|e| MountError::MountFailed {
            device: device.to_string(),
            mount_point: mount_point.display().to_string(),
            reason: e.to_string(),
        })?;

        if !output.status.success() {
            return Err(MountError::MountFailed {
                device: device.to_string(),
                mount_point: mount_point.display().to_string(),
                reason: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // Refresh to get updated mount info
        self.refresh().ok();

        tracing::info!("Mounted {} at {}", device, mount_point.display());
        Ok(())
    }

    /// Unmount a mount point
    pub fn unmount(&mut self, mount_point: &Path) -> Result<(), MountError> {
        let output = Command::new("umount")
            .arg(mount_point)
            .output()
            .map_err(|e| MountError::UnmountFailed {
                mount_point: mount_point.display().to_string(),
                reason: e.to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("busy") {
                return Err(MountError::Busy(mount_point.display().to_string()));
            }
            return Err(MountError::UnmountFailed {
                mount_point: mount_point.display().to_string(),
                reason: stderr.to_string(),
            });
        }

        self.mounts.remove(mount_point);
        tracing::info!("Unmounted {}", mount_point.display());
        Ok(())
    }

    /// Find mount points for removable storage (SD cards, USB)
    pub fn find_removable(&self) -> Vec<&MountPoint> {
        self.mounts
            .values()
            .filter(|m| {
                m.device.starts_with("/dev/mmcblk")
                    || m.device.starts_with("/dev/sd")
                    || m.mount_point.starts_with("/media")
                    || m.mount_point.starts_with("/mnt")
            })
            .collect()
    }
}

impl Default for MountManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.refresh().ok();
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mount_manager_creation() {
        let manager = MountManager::new();
        assert!(manager.mounts.is_empty());
    }

    #[test]
    fn test_mount_point_struct() {
        let mount = MountPoint {
            device: "/dev/sda1".to_string(),
            mount_point: PathBuf::from("/mnt/usb"),
            filesystem: "ext4".to_string(),
            options: vec!["rw".to_string(), "noatime".to_string()],
        };

        assert_eq!(mount.device, "/dev/sda1");
        assert_eq!(mount.filesystem, "ext4");
        assert_eq!(mount.options.len(), 2);
    }

    #[test]
    fn test_is_mounted_empty() {
        let manager = MountManager::new();
        assert!(!manager.is_mounted(Path::new("/mnt/test")));
    }

    #[test]
    fn test_get_mount_not_found() {
        let manager = MountManager::new();
        assert!(manager.get_mount(Path::new("/mnt/nonexistent")).is_none());
    }

    #[test]
    fn test_find_removable_empty() {
        let manager = MountManager::new();
        let removable = manager.find_removable();
        assert!(removable.is_empty());
    }

    #[test]
    fn test_mount_error_display() {
        let err = MountError::MountFailed {
            device: "/dev/sda1".to_string(),
            mount_point: "/mnt/usb".to_string(),
            reason: "Permission denied".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("sda1"));
        assert!(msg.contains("Permission denied"));

        let err = MountError::UnmountFailed {
            mount_point: "/mnt/usb".to_string(),
            reason: "Device busy".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("unmount"));
        assert!(msg.contains("busy"));

        let err = MountError::Busy("/mnt/usb".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("busy"));
    }

    #[test]
    fn test_default_manager() {
        // Default manager tries to read /proc/mounts which may work on Linux
        let _manager = MountManager::default();
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_mounts_accessor() {
        let manager = MountManager::new();
        let mounts = manager.mounts();
        assert!(mounts.is_empty());
    }
}
