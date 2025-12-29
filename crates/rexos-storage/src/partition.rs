//! Partition and storage device detection

use crate::StorageError;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a partition
#[derive(Debug, Clone)]
pub struct PartitionInfo {
    pub device: String,
    pub size_bytes: u64,
    pub filesystem: Option<String>,
    pub label: Option<String>,
    pub uuid: Option<String>,
}

/// A partition on a storage device
#[derive(Debug, Clone)]
pub struct Partition {
    pub path: PathBuf,
    pub info: PartitionInfo,
}

/// A storage device (SD card, eMMC, USB drive)
#[derive(Debug, Clone)]
pub struct StorageDevice {
    pub path: PathBuf,
    pub model: Option<String>,
    pub size_bytes: u64,
    pub removable: bool,
    pub partitions: Vec<Partition>,
}

impl StorageDevice {
    /// Detect all storage devices in the system
    pub fn detect_all() -> Result<Vec<Self>, StorageError> {
        let mut devices = Vec::new();

        // Read block devices from /sys/block
        let block_dir = Path::new("/sys/block");
        if !block_dir.exists() {
            return Ok(devices);
        }

        for entry in fs::read_dir(block_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip loop devices and ram devices
            if name.starts_with("loop") || name.starts_with("ram") {
                continue;
            }

            // Only include mmcblk (SD/eMMC) and sd (USB/SATA) devices
            if !name.starts_with("mmcblk") && !name.starts_with("sd") {
                continue;
            }

            if let Ok(device) = Self::from_sysfs(&entry.path()) {
                devices.push(device);
            }
        }

        Ok(devices)
    }

    /// Create a StorageDevice from sysfs path
    fn from_sysfs(sysfs_path: &Path) -> Result<Self, StorageError> {
        // Use let-else for early return (Rust 1.65+)
        let Some(file_name) = sysfs_path.file_name() else {
            return Err(StorageError::DeviceNotFound("Invalid path".into()));
        };
        let name = file_name.to_string_lossy().into_owned();

        let device_path = PathBuf::from(format!("/dev/{name}"));

        // Read device size (in 512-byte sectors)
        let size_sectors: u64 = fs::read_to_string(sysfs_path.join("size"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        let size_bytes = size_sectors * 512;

        // Check if removable
        let removable = fs::read_to_string(sysfs_path.join("removable"))
            .unwrap_or_default()
            .trim()
            == "1";

        // Read model if available
        let model = fs::read_to_string(sysfs_path.join("device/model"))
            .ok()
            .map(|s| s.trim().to_string());

        // Find partitions
        let partitions = Self::find_partitions(sysfs_path, &name)?;

        Ok(Self {
            path: device_path,
            model,
            size_bytes,
            removable,
            partitions,
        })
    }

    /// Find partitions for a device
    fn find_partitions(
        sysfs_path: &Path,
        device_name: &str,
    ) -> Result<Vec<Partition>, StorageError> {
        let mut partitions = Vec::new();

        for entry in fs::read_dir(sysfs_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            // Partition names are like mmcblk0p1 or sda1
            if !name.starts_with(device_name) || name == device_name {
                continue;
            }

            // Check if it's a partition (has a 'partition' file)
            if !entry.path().join("partition").exists() {
                continue;
            }

            let partition_path = PathBuf::from(format!("/dev/{}", name));

            // Read partition size
            let size_sectors: u64 = fs::read_to_string(entry.path().join("size"))
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0);

            let info = PartitionInfo {
                device: name.clone(),
                size_bytes: size_sectors * 512,
                filesystem: Self::get_partition_fs(&partition_path),
                label: Self::get_partition_label(&partition_path),
                uuid: Self::get_partition_uuid(&partition_path),
            };

            partitions.push(Partition {
                path: partition_path,
                info,
            });
        }

        // Sort partitions by name
        partitions.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(partitions)
    }

    /// Get filesystem type using blkid-style lookup
    fn get_partition_fs(path: &Path) -> Option<String> {
        // Try reading from /sys/class/block/*/device/uevent or use blkid
        let device_name = path.file_name()?.to_string_lossy();
        let uevent_path = format!("/sys/class/block/{}/uevent", device_name);

        if let Ok(contents) = fs::read_to_string(&uevent_path) {
            for line in contents.lines() {
                if line.starts_with("DEVTYPE=") {
                    // This is just device type, not filesystem
                    // In real implementation, use blkid or libblkid
                }
            }
        }

        // Fallback: would normally call blkid here
        None
    }

    /// Get partition label
    fn get_partition_label(path: &Path) -> Option<String> {
        // Check /dev/disk/by-label symlinks
        let by_label = Path::new("/dev/disk/by-label");
        if !by_label.exists() {
            return None;
        }

        for entry in fs::read_dir(by_label).ok()? {
            let entry = entry.ok()?;
            if let Ok(target) = fs::read_link(entry.path()) {
                let target_name = target.file_name()?;
                let path_name = path.file_name()?;
                if target_name == path_name {
                    return Some(entry.file_name().to_string_lossy().to_string());
                }
            }
        }

        None
    }

    /// Get partition UUID
    fn get_partition_uuid(path: &Path) -> Option<String> {
        // Check /dev/disk/by-uuid symlinks
        let by_uuid = Path::new("/dev/disk/by-uuid");
        if !by_uuid.exists() {
            return None;
        }

        for entry in fs::read_dir(by_uuid).ok()? {
            let entry = entry.ok()?;
            if let Ok(target) = fs::read_link(entry.path()) {
                let target_name = target.file_name()?;
                let path_name = path.file_name()?;
                if target_name == path_name {
                    return Some(entry.file_name().to_string_lossy().to_string());
                }
            }
        }

        None
    }

    /// Check if this is likely the boot device
    pub fn is_boot_device(&self) -> bool {
        // On most Anbernic devices, mmcblk0 is the boot device
        self.path.to_string_lossy().contains("mmcblk0")
    }

    /// Check if this is likely a secondary SD card
    pub fn is_secondary_sd(&self) -> bool {
        self.path.to_string_lossy().contains("mmcblk1")
    }

    /// Get total size in human-readable format
    pub fn size_human(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size_bytes >= GB {
            format!("{:.1} GB", self.size_bytes as f64 / GB as f64)
        } else if self.size_bytes >= MB {
            format!("{:.1} MB", self.size_bytes as f64 / MB as f64)
        } else if self.size_bytes >= KB {
            format!("{:.1} KB", self.size_bytes as f64 / KB as f64)
        } else {
            format!("{} B", self.size_bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_human() {
        let device = StorageDevice {
            path: PathBuf::from("/dev/test"),
            model: None,
            size_bytes: 32 * 1024 * 1024 * 1024, // 32GB
            removable: true,
            partitions: vec![],
        };
        assert!(device.size_human().contains("GB"));
    }

    #[test]
    fn test_size_human_mb() {
        let device = StorageDevice {
            path: PathBuf::from("/dev/test"),
            model: None,
            size_bytes: 512 * 1024 * 1024, // 512MB
            removable: false,
            partitions: vec![],
        };
        assert!(device.size_human().contains("MB"));
    }

    #[test]
    fn test_size_human_kb() {
        let device = StorageDevice {
            path: PathBuf::from("/dev/test"),
            model: None,
            size_bytes: 512 * 1024, // 512KB
            removable: false,
            partitions: vec![],
        };
        assert!(device.size_human().contains("KB"));
    }

    #[test]
    fn test_size_human_bytes() {
        let device = StorageDevice {
            path: PathBuf::from("/dev/test"),
            model: None,
            size_bytes: 512, // 512B
            removable: false,
            partitions: vec![],
        };
        assert!(device.size_human().contains("B"));
        assert!(!device.size_human().contains("KB"));
    }

    #[test]
    fn test_is_boot_device() {
        let device = StorageDevice {
            path: PathBuf::from("/dev/mmcblk0"),
            model: None,
            size_bytes: 0,
            removable: false,
            partitions: vec![],
        };
        assert!(device.is_boot_device());

        let device2 = StorageDevice {
            path: PathBuf::from("/dev/mmcblk1"),
            model: None,
            size_bytes: 0,
            removable: true,
            partitions: vec![],
        };
        assert!(!device2.is_boot_device());
    }

    #[test]
    fn test_is_secondary_sd() {
        let device = StorageDevice {
            path: PathBuf::from("/dev/mmcblk1"),
            model: None,
            size_bytes: 0,
            removable: true,
            partitions: vec![],
        };
        assert!(device.is_secondary_sd());

        let device2 = StorageDevice {
            path: PathBuf::from("/dev/mmcblk0"),
            model: None,
            size_bytes: 0,
            removable: false,
            partitions: vec![],
        };
        assert!(!device2.is_secondary_sd());
    }

    #[test]
    fn test_partition_info() {
        let info = PartitionInfo {
            device: "mmcblk0p1".to_string(),
            size_bytes: 1024 * 1024 * 1024, // 1GB
            filesystem: Some("ext4".to_string()),
            label: Some("BOOT".to_string()),
            uuid: Some("1234-5678".to_string()),
        };

        assert_eq!(info.device, "mmcblk0p1");
        assert_eq!(info.filesystem, Some("ext4".to_string()));
        assert_eq!(info.label, Some("BOOT".to_string()));
    }

    #[test]
    fn test_partition_struct() {
        let partition = Partition {
            path: PathBuf::from("/dev/mmcblk0p1"),
            info: PartitionInfo {
                device: "mmcblk0p1".to_string(),
                size_bytes: 1024 * 1024 * 1024,
                filesystem: None,
                label: None,
                uuid: None,
            },
        };

        assert_eq!(partition.path, PathBuf::from("/dev/mmcblk0p1"));
        assert_eq!(partition.info.size_bytes, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_storage_device_with_partitions() {
        let partition = Partition {
            path: PathBuf::from("/dev/sda1"),
            info: PartitionInfo {
                device: "sda1".to_string(),
                size_bytes: 10 * 1024 * 1024 * 1024,
                filesystem: Some("ext4".to_string()),
                label: Some("DATA".to_string()),
                uuid: None,
            },
        };

        let device = StorageDevice {
            path: PathBuf::from("/dev/sda"),
            model: Some("USB Flash Drive".to_string()),
            size_bytes: 16 * 1024 * 1024 * 1024,
            removable: true,
            partitions: vec![partition],
        };

        assert_eq!(device.partitions.len(), 1);
        assert_eq!(device.model, Some("USB Flash Drive".to_string()));
        assert!(device.removable);
    }
}
