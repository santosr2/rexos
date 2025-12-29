//! Integration tests for the emulator launcher system

use rexos_emulator::{EmulatorLauncher, GameSystem, LaunchConfig};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test environment for emulator integration tests
struct EmulatorTestEnv {
    #[allow(dead_code)]
    temp_dir: TempDir,
    roms_dir: PathBuf,
    cores_dir: PathBuf,
}

impl EmulatorTestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let roms_dir = temp_dir.path().join("roms");
        let cores_dir = temp_dir.path().join("cores");

        fs::create_dir_all(&roms_dir).unwrap();
        fs::create_dir_all(&cores_dir).unwrap();

        Self {
            temp_dir,
            roms_dir,
            cores_dir,
        }
    }

    fn create_rom(&self, name: &str) -> PathBuf {
        let path = self.roms_dir.join(name);
        fs::write(&path, b"FAKE_ROM_DATA").unwrap();
        path
    }

    fn create_core(&self, name: &str) -> PathBuf {
        let path = self.cores_dir.join(format!("{}_libretro.so", name));
        fs::write(&path, b"FAKE_CORE").unwrap();
        path
    }
}

#[test]
fn test_launch_config_builder_integration() {
    let env = EmulatorTestEnv::new();
    let rom_path = env.create_rom("test_game.gba");

    let config = LaunchConfig::for_rom(&rom_path)
        .with_system(GameSystem::GameBoyAdvance)
        .with_core("mgba")
        .with_state(1);

    assert_eq!(config.rom_path, rom_path);
    assert_eq!(config.system, Some(GameSystem::GameBoyAdvance));
    assert_eq!(config.core, Some("mgba".to_string()));
    assert_eq!(config.load_state, Some(1));
}

#[test]
fn test_system_detection_from_extension_comprehensive() {
    let test_cases = [
        ("game.gba", Some(GameSystem::GameBoyAdvance)),
        ("game.gb", Some(GameSystem::GameBoy)),
        ("game.gbc", Some(GameSystem::GameBoyColor)),
        ("game.nes", Some(GameSystem::Nes)),
        ("game.sfc", Some(GameSystem::Snes)),
        ("game.smc", Some(GameSystem::Snes)),
        ("game.md", Some(GameSystem::Genesis)),
        ("game.bin", Some(GameSystem::Genesis)),
        ("game.gen", Some(GameSystem::Genesis)),
        ("game.n64", Some(GameSystem::N64)),
        ("game.z64", Some(GameSystem::N64)),
        ("game.v64", Some(GameSystem::N64)),
        ("game.nds", Some(GameSystem::Nds)),
        ("game.pce", Some(GameSystem::PcEngine)),
        ("game.xyz", None), // Unknown extension
    ];

    for (filename, expected_system) in test_cases {
        let config = LaunchConfig::for_rom(format!("/roms/{}", filename));
        assert_eq!(
            config.system, expected_system,
            "System detection failed for {}",
            filename
        );
    }
}

#[test]
fn test_emulator_launcher_core_management() {
    let env = EmulatorTestEnv::new();
    env.create_core("mgba");
    env.create_core("snes9x");

    let launcher = EmulatorLauncher::with_paths(
        "/usr/bin/retroarch",
        "/usr/bin/retroarch32",
        &env.cores_dir,
        &env.cores_dir,
    );

    // Test core existence check
    assert!(launcher.has_core("mgba", false));
    assert!(launcher.has_core("snes9x", false));
    assert!(!launcher.has_core("nonexistent", false));

    // Test core listing
    let cores = launcher.list_cores(false);
    assert_eq!(cores.len(), 2);
    assert!(cores.contains(&"mgba".to_string()));
    assert!(cores.contains(&"snes9x".to_string()));
}

#[test]
fn test_launch_config_32bit_mode() {
    let env = EmulatorTestEnv::new();
    let rom_path = env.create_rom("test.nes");

    let config = LaunchConfig::for_rom(&rom_path).use_32bit();
    assert!(config.use_32bit);

    let config_64bit = LaunchConfig::for_rom(&rom_path);
    assert!(!config_64bit.use_32bit);
}

#[test]
fn test_launch_validates_rom_path() {
    let launcher = EmulatorLauncher::new();
    let config = LaunchConfig::for_rom("/nonexistent/game.gba");

    let result = launcher.launch(config);
    assert!(result.is_err());
}

#[test]
fn test_game_system_properties() {
    // Test display names
    assert_eq!(
        GameSystem::GameBoyAdvance.display_name(),
        "Game Boy Advance"
    );
    assert_eq!(GameSystem::Snes.display_name(), "Super Nintendo");

    // Test default cores
    assert_eq!(GameSystem::GameBoyAdvance.default_core(), "mgba");
    assert_eq!(GameSystem::Snes.default_core(), "snes9x");
    assert_eq!(GameSystem::Nes.default_core(), "fceumm");
}
