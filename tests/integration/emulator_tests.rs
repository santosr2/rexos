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
fn test_launch_config_builder() {
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
fn test_system_detection_from_extension() {
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
        ("game.nds", Some(GameSystem::NintendoDs)),
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
fn test_default_cores() {
    let systems = [
        (GameSystem::GameBoyAdvance, "mgba"),
        (GameSystem::GameBoy, "gambatte"),
        (GameSystem::GameBoyColor, "gambatte"),
        (GameSystem::Nes, "nestopia"),
        (GameSystem::Snes, "snes9x"),
        (GameSystem::Genesis, "genesis_plus_gx"),
        (GameSystem::N64, "mupen64plus_next"),
        (GameSystem::PcEngine, "mednafen_pce"),
    ];

    for (system, expected_core) in systems {
        assert_eq!(
            system.default_core(),
            expected_core,
            "Wrong default core for {:?}",
            system
        );
    }
}

#[test]
fn test_emulator_launcher_core_check() {
    let env = EmulatorTestEnv::new();
    env.create_core("mgba");
    env.create_core("snes9x");

    let launcher = EmulatorLauncher::with_paths(
        "/usr/bin/retroarch",     // Doesn't need to exist for this test
        "/usr/bin/retroarch32",   // Doesn't need to exist for this test
        &env.cores_dir,
        &env.cores_dir,
    );

    assert!(launcher.has_core("mgba", false));
    assert!(launcher.has_core("snes9x", false));
    assert!(!launcher.has_core("nonexistent", false));
}

#[test]
fn test_emulator_launcher_list_cores() {
    let env = EmulatorTestEnv::new();
    env.create_core("mgba");
    env.create_core("snes9x");
    env.create_core("nestopia");

    let launcher = EmulatorLauncher::with_paths(
        "/usr/bin/retroarch",
        "/usr/bin/retroarch32",
        &env.cores_dir,
        &env.cores_dir,
    );

    let cores = launcher.list_cores(false);
    assert_eq!(cores.len(), 3);
    assert!(cores.contains(&"mgba".to_string()));
    assert!(cores.contains(&"snes9x".to_string()));
    assert!(cores.contains(&"nestopia".to_string()));
}

#[test]
fn test_launch_config_32bit() {
    let env = EmulatorTestEnv::new();
    let rom_path = env.create_rom("test.nes");

    let config = LaunchConfig::for_rom(&rom_path).use_32bit();

    assert!(config.use_32bit);
}

#[test]
fn test_launch_validates_rom_exists() {
    let launcher = EmulatorLauncher::new();
    let config = LaunchConfig::for_rom("/nonexistent/game.gba");

    let result = launcher.launch(config);
    assert!(result.is_err());
}

#[test]
fn test_game_system_names() {
    let systems = [
        (GameSystem::GameBoyAdvance, "Game Boy Advance"),
        (GameSystem::GameBoy, "Game Boy"),
        (GameSystem::Nes, "Nintendo Entertainment System"),
        (GameSystem::Snes, "Super Nintendo"),
        (GameSystem::Genesis, "Sega Genesis"),
        (GameSystem::N64, "Nintendo 64"),
    ];

    for (system, expected_name) in systems {
        assert_eq!(
            system.display_name(),
            expected_name,
            "Wrong display name for {:?}",
            system
        );
    }
}
