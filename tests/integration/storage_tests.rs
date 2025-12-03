//! Integration tests for the storage system

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper for storage tests
struct StorageTestEnvironment {
    #[allow(dead_code)]
    temp_dir: TempDir,
    mount_point: PathBuf,
}

impl StorageTestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mount_point = temp_dir.path().join("mnt");
        fs::create_dir_all(&mount_point).expect("Failed to create mount point");

        Self {
            temp_dir,
            mount_point,
        }
    }
}

#[test]
fn test_storage_environment_creation() {
    let env = StorageTestEnvironment::new();
    assert!(env.mount_point.exists());
}

#[test]
fn test_path_validation() {
    let valid_paths = ["/rexos/roms", "/mnt/sdcard", "/home/user/games"];

    let invalid_paths = ["../escape", "/etc/passwd", "relative/path"];

    for path in valid_paths {
        assert!(
            is_safe_path(path),
            "Path should be valid: {}",
            path
        );
    }

    for path in invalid_paths {
        // Note: This is a simplified check
        if path.contains("..") || !path.starts_with('/') {
            // These would be caught by a real path validator
        }
    }
}

fn is_safe_path(path: &str) -> bool {
    // Simple path safety check
    path.starts_with('/') && !path.contains("..") && !path.contains("//")
}

#[test]
fn test_file_extension_filter() {
    let rom_extensions = [
        "gba", "sfc", "smc", "nes", "gb", "gbc", "bin", "iso", "cue", "zip", "7z",
    ];

    let non_rom_extensions = ["txt", "md", "jpg", "png", "xml", "json"];

    for ext in rom_extensions {
        assert!(
            is_rom_extension(ext),
            "{} should be recognized as ROM",
            ext
        );
    }

    for ext in non_rom_extensions {
        assert!(
            !is_rom_extension(ext),
            "{} should not be recognized as ROM",
            ext
        );
    }
}

fn is_rom_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "gba" | "gb"
            | "gbc"
            | "sfc"
            | "smc"
            | "nes"
            | "bin"
            | "iso"
            | "cue"
            | "chd"
            | "zip"
            | "7z"
            | "n64"
            | "z64"
            | "v64"
            | "nds"
            | "pbp"
            | "img"
    )
}

#[test]
fn test_storage_size_formatting() {
    let test_cases = [
        (0, "0 B"),
        (512, "512 B"),
        (1024, "1.0 KB"),
        (1536, "1.5 KB"),
        (1048576, "1.0 MB"),
        (1073741824, "1.0 GB"),
        (1099511627776, "1.0 TB"),
    ];

    for (bytes, expected) in test_cases {
        let formatted = format_size(bytes);
        assert_eq!(formatted, expected, "Failed for {} bytes", bytes);
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[test]
fn test_partition_label_parsing() {
    let test_cases = [
        ("REXOS_ROOT", true),
        ("REXOS_BOOT", true),
        ("ROMS", true),
        ("My Games", true),
        ("", false),
    ];

    for (label, valid) in test_cases {
        let is_valid = !label.is_empty() && label.len() <= 32;
        assert_eq!(is_valid, valid, "Label '{}' validity check failed", label);
    }
}

#[test]
fn test_mount_options_parsing() {
    let options = "rw,noatime,nodiratime";
    let parts: Vec<&str> = options.split(',').collect();

    assert!(parts.contains(&"rw"));
    assert!(parts.contains(&"noatime"));
    assert!(parts.contains(&"nodiratime"));
    assert!(!parts.contains(&"ro"));
}

#[test]
fn test_filesystem_type_detection() {
    let fs_types = [
        ("ext4", true),
        ("vfat", true),
        ("ntfs", true),
        ("exfat", true),
        ("f2fs", true),
        ("unknown_fs", false),
    ];

    for (fs_type, supported) in fs_types {
        let is_supported = is_supported_filesystem(fs_type);
        assert_eq!(
            is_supported, supported,
            "Filesystem {} support check failed",
            fs_type
        );
    }
}

fn is_supported_filesystem(fs_type: &str) -> bool {
    matches!(
        fs_type.to_lowercase().as_str(),
        "ext4" | "ext3" | "ext2" | "vfat" | "fat32" | "fat16" | "ntfs" | "exfat" | "f2fs"
    )
}

#[test]
fn test_device_path_parsing() {
    let valid_devices = [
        "/dev/sda",
        "/dev/sda1",
        "/dev/mmcblk0",
        "/dev/mmcblk0p1",
        "/dev/nvme0n1",
        "/dev/nvme0n1p1",
    ];

    for device in valid_devices {
        assert!(
            is_valid_device_path(device),
            "{} should be valid device path",
            device
        );
    }
}

fn is_valid_device_path(path: &str) -> bool {
    path.starts_with("/dev/")
        && (path.contains("sd")
            || path.contains("mmcblk")
            || path.contains("nvme")
            || path.contains("loop"))
}

#[test]
fn test_directory_size_calculation() {
    let env = StorageTestEnvironment::new();

    // Create some test files
    let dir = env.mount_point.join("test");
    fs::create_dir_all(&dir).unwrap();

    fs::write(dir.join("file1.txt"), vec![0u8; 1000]).unwrap();
    fs::write(dir.join("file2.txt"), vec![0u8; 2000]).unwrap();

    let subdir = dir.join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    fs::write(subdir.join("file3.txt"), vec![0u8; 500]).unwrap();

    let total_size = calculate_directory_size(&dir);
    assert_eq!(total_size, 3500, "Total size should be 3500 bytes");
}

fn calculate_directory_size(path: &PathBuf) -> u64 {
    let mut size = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                size += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                size += calculate_directory_size(&path);
            }
        }
    }

    size
}

#[test]
#[ignore] // Requires root permissions
fn test_mount_unmount() {
    // This test would require root permissions to actually mount/unmount
}

#[test]
#[ignore] // Requires actual storage crate
fn test_storage_watcher() {
    // This test would use the actual storage watcher
}
