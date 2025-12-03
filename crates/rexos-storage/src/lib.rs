//! Storage management for RexOS
//!
//! Handles SD cards, eMMC, and USB storage devices. Based on ArkOS storage patterns
//! with separate partitions for system and ROMs (EASYROMS pattern).
//!
//! # Storage Layout
//!
//! RexOS follows a similar layout to ArkOS:
//! - Partition 1: System (ext4) - OS files, emulators, configs
//! - Partition 2: ROMs (exFAT) - Games, BIOS files, saves
//! - Optional: Secondary SD card for additional storage

mod mount;
mod partition;
mod watcher;

pub use mount::{MountError, MountManager, MountPoint};
pub use partition::{Partition, PartitionInfo, StorageDevice};
pub use watcher::{StorageEvent, StorageWatcher};

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Mount failed: {0}")]
    MountFailed(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Partition error: {0}")]
    PartitionError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Standard RexOS paths
pub struct Paths {
    /// Root of the ROMs partition (like ArkOS EASYROMS)
    pub roms: PathBuf,
    /// BIOS files location
    pub bios: PathBuf,
    /// Save files location
    pub saves: PathBuf,
    /// Save states location
    pub states: PathBuf,
    /// Screenshots location
    pub screenshots: PathBuf,
    /// Configuration files
    pub config: PathBuf,
    /// Themes directory
    pub themes: PathBuf,
    /// RetroArch cores
    pub cores: PathBuf,
    /// Secondary SD card ROMs (if present)
    pub roms2: Option<PathBuf>,
}

impl Default for Paths {
    fn default() -> Self {
        Self {
            roms: PathBuf::from("/roms"),
            bios: PathBuf::from("/roms/bios"),
            saves: PathBuf::from("/roms/saves"),
            states: PathBuf::from("/roms/states"),
            screenshots: PathBuf::from("/roms/screenshots"),
            config: PathBuf::from("/etc/rexos"),
            themes: PathBuf::from("/usr/share/rexos/themes"),
            cores: PathBuf::from("/usr/lib/libretro"),
            roms2: None,
        }
    }
}

impl Paths {
    /// Detect and configure paths based on mounted storage
    pub fn detect() -> Result<Self, StorageError> {
        let mut paths = Self::default();

        // Check for secondary SD card
        let roms2_path = PathBuf::from("/roms2");
        if roms2_path.exists() && roms2_path.is_dir() {
            paths.roms2 = Some(roms2_path);
        }

        // Ensure critical directories exist
        std::fs::create_dir_all(&paths.bios).ok();
        std::fs::create_dir_all(&paths.saves).ok();
        std::fs::create_dir_all(&paths.states).ok();
        std::fs::create_dir_all(&paths.screenshots).ok();

        Ok(paths)
    }

    /// Get the ROM path for a specific system
    pub fn system_roms(&self, system: &str) -> PathBuf {
        self.roms.join(system)
    }

    /// Get the save path for a specific system
    pub fn system_saves(&self, system: &str) -> PathBuf {
        self.saves.join(system)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_paths() {
        let paths = Paths::default();
        assert_eq!(paths.roms, PathBuf::from("/roms"));
        assert_eq!(paths.bios, PathBuf::from("/roms/bios"));
        assert_eq!(paths.saves, PathBuf::from("/roms/saves"));
        assert_eq!(paths.states, PathBuf::from("/roms/states"));
        assert_eq!(paths.screenshots, PathBuf::from("/roms/screenshots"));
        assert_eq!(paths.config, PathBuf::from("/etc/rexos"));
        assert_eq!(paths.themes, PathBuf::from("/usr/share/rexos/themes"));
        assert_eq!(paths.cores, PathBuf::from("/usr/lib/libretro"));
        assert!(paths.roms2.is_none());
    }

    #[test]
    fn test_system_paths() {
        let paths = Paths::default();
        assert_eq!(paths.system_roms("gba"), PathBuf::from("/roms/gba"));
        assert_eq!(paths.system_roms("snes"), PathBuf::from("/roms/snes"));
        assert_eq!(paths.system_roms("psx"), PathBuf::from("/roms/psx"));
        assert_eq!(paths.system_saves("gba"), PathBuf::from("/roms/saves/gba"));
        assert_eq!(
            paths.system_saves("snes"),
            PathBuf::from("/roms/saves/snes")
        );
    }

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::MountFailed("test mount error".to_string());
        assert_eq!(format!("{}", err), "Mount failed: test mount error");

        let err = StorageError::DeviceNotFound("/dev/sda1".to_string());
        assert_eq!(format!("{}", err), "Device not found: /dev/sda1");

        let err = StorageError::PartitionError("invalid partition".to_string());
        assert_eq!(format!("{}", err), "Partition error: invalid partition");
    }

    #[test]
    fn test_paths_with_secondary_sd() {
        let paths = Paths {
            roms2: Some(PathBuf::from("/roms2")),
            ..Default::default()
        };

        assert!(paths.roms2.is_some());
        assert_eq!(paths.roms2.unwrap(), PathBuf::from("/roms2"));
    }

    #[test]
    fn test_all_gaming_systems() {
        let paths = Paths::default();
        let systems = [
            "gba",
            "gbc",
            "gb",
            "nes",
            "snes",
            "n64",
            "nds",
            "psx",
            "psp",
            "sega-md",
            "sega-cd",
            "dreamcast",
            "arcade",
            "mame",
            "neogeo",
            "pce",
        ];

        for system in systems {
            let rom_path = paths.system_roms(system);
            let save_path = paths.system_saves(system);

            assert!(rom_path.to_string_lossy().contains(system));
            assert!(save_path.to_string_lossy().contains(system));
        }
    }
}
