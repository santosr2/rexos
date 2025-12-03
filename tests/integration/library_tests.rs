//! Integration tests for the game library service

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper to create a temporary test environment
struct TestEnvironment {
    #[allow(dead_code)]
    temp_dir: TempDir,
    roms_dir: PathBuf,
    db_path: PathBuf,
}

impl TestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let roms_dir = temp_dir.path().join("roms");
        let db_path = temp_dir.path().join("library.db");

        // Create ROM directories
        for system in &["gba", "snes", "nes", "psx"] {
            fs::create_dir_all(roms_dir.join(system)).expect("Failed to create ROM directory");
        }

        Self {
            temp_dir,
            roms_dir,
            db_path,
        }
    }

    fn create_test_rom(&self, system: &str, name: &str, size: usize) -> PathBuf {
        let path = self.roms_dir.join(system).join(name);
        let data = vec![0u8; size];
        fs::write(&path, data).expect("Failed to create test ROM");
        path
    }
}

#[test]
fn test_create_library_environment() {
    let env = TestEnvironment::new();

    assert!(env.roms_dir.exists());
    assert!(env.roms_dir.join("gba").exists());
    assert!(env.roms_dir.join("snes").exists());
    assert!(env.roms_dir.join("nes").exists());
    assert!(env.roms_dir.join("psx").exists());
}

#[test]
fn test_create_test_roms() {
    let env = TestEnvironment::new();

    let rom1 = env.create_test_rom("gba", "game1.gba", 1024);
    let rom2 = env.create_test_rom("snes", "game2.sfc", 2048);

    assert!(rom1.exists());
    assert!(rom2.exists());
    assert_eq!(fs::metadata(&rom1).unwrap().len(), 1024);
    assert_eq!(fs::metadata(&rom2).unwrap().len(), 2048);
}

#[test]
fn test_rom_extension_detection() {
    let gba_extensions = ["gba", "agb", "gb", "gbc", "sgb"];
    let snes_extensions = ["sfc", "smc", "fig", "swc"];
    let psx_extensions = ["bin", "cue", "iso", "img", "pbp", "chd"];

    for ext in gba_extensions {
        assert!(is_gba_extension(ext), "Expected {} to be GBA", ext);
    }

    for ext in snes_extensions {
        assert!(is_snes_extension(ext), "Expected {} to be SNES", ext);
    }

    for ext in psx_extensions {
        assert!(is_psx_extension(ext), "Expected {} to be PSX", ext);
    }
}

fn is_gba_extension(ext: &str) -> bool {
    matches!(ext.to_lowercase().as_str(), "gba" | "agb" | "gb" | "gbc" | "sgb")
}

fn is_snes_extension(ext: &str) -> bool {
    matches!(ext.to_lowercase().as_str(), "sfc" | "smc" | "fig" | "swc")
}

fn is_psx_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "bin" | "cue" | "iso" | "img" | "pbp" | "chd"
    )
}

#[test]
fn test_game_title_extraction() {
    let test_cases = [
        ("Super Mario World (USA).sfc", "Super Mario World"),
        ("Pokemon - Emerald Version (U).gba", "Pokemon - Emerald Version"),
        ("Final Fantasy VII (Disc 1).bin", "Final Fantasy VII"),
        ("game_name.gba", "game name"),
        ("NoRegion.sfc", "NoRegion"),
    ];

    for (filename, expected_title) in test_cases {
        let title = extract_game_title(filename);
        assert_eq!(title, expected_title, "Failed for: {}", filename);
    }
}

fn extract_game_title(filename: &str) -> String {
    // Remove extension
    let name = filename
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(filename);

    // Remove region codes in parentheses
    let name = name
        .split('(')
        .next()
        .unwrap_or(name)
        .trim();

    // Replace underscores with spaces
    name.replace('_', " ")
}

#[test]
fn test_system_detection_from_path() {
    let test_cases = [
        ("/rexos/roms/gba/game.gba", "gba"),
        ("/rexos/roms/snes/game.sfc", "snes"),
        ("/rexos/roms/psx/game.bin", "psx"),
        ("/rexos/roms/arcade/game.zip", "arcade"),
    ];

    for (path, expected_system) in test_cases {
        let system = detect_system_from_path(path);
        assert_eq!(
            system.as_deref(),
            Some(expected_system),
            "Failed for: {}",
            path
        );
    }
}

fn detect_system_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();

    // Find "roms" in path and get the next component
    for (i, part) in parts.iter().enumerate() {
        if *part == "roms" && i + 1 < parts.len() {
            return Some(parts[i + 1].to_string());
        }
    }

    None
}

#[test]
#[ignore] // Requires actual library crate
fn test_library_scan() {
    // This test would use the actual library crate
    // Marked as ignored since it requires the full crate
}

#[test]
#[ignore] // Requires actual library crate
fn test_library_search() {
    // This test would use the actual library crate
    // Marked as ignored since it requires the full crate
}
