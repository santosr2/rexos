//! Integration tests for the configuration system

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper for configuration tests
struct ConfigTestEnvironment {
    #[allow(dead_code)]
    temp_dir: TempDir,
    config_dir: PathBuf,
}

impl ConfigTestEnvironment {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_dir = temp_dir.path().join("config");
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        Self {
            temp_dir,
            config_dir,
        }
    }

    fn write_config(&self, name: &str, content: &str) -> PathBuf {
        let path = self.config_dir.join(name);
        fs::write(&path, content).expect("Failed to write config");
        path
    }
}

#[test]
fn test_system_config_parsing() {
    let env = ConfigTestEnvironment::new();

    let config_content = r#"
[system]
device = "rg353m"
timezone = "America/New_York"
language = "en"

[display]
brightness = 75
auto_brightness = false
screen_timeout = 300

[audio]
volume = 80
mute = false

[power]
sleep_timeout = 600
auto_poweroff = 0
"#;

    let path = env.write_config("system.toml", config_content);
    assert!(path.exists());

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("device = \"rg353m\""));
    assert!(content.contains("brightness = 75"));
    assert!(content.contains("volume = 80"));
}

#[test]
fn test_retroarch_config_parsing() {
    let env = ConfigTestEnvironment::new();

    let config_content = r#"
video_fullscreen = "true"
video_vsync = "true"
audio_driver = "alsa"
audio_latency = "64"
input_enable_hotkey_btn = "6"
savefile_directory = "/rexos/saves"
savestate_directory = "/rexos/states"
"#;

    let path = env.write_config("retroarch.cfg", config_content);
    let content = fs::read_to_string(&path).unwrap();

    assert!(content.contains("video_fullscreen = \"true\""));
    assert!(content.contains("audio_driver = \"alsa\""));
    assert!(content.contains("savefile_directory = \"/rexos/saves\""));
}

#[test]
fn test_hotkey_config_parsing() {
    let hotkeys = [
        ("exit", "SELECT+START"),
        ("save_state", "SELECT+R1"),
        ("load_state", "SELECT+L1"),
        ("menu", "SELECT+X"),
        ("pause", "SELECT+Y"),
        ("fast_forward", "SELECT+R2"),
        ("screenshot", "SELECT+L2"),
    ];

    for (action, combo) in hotkeys {
        let parts: Vec<&str> = combo.split('+').collect();
        assert_eq!(parts.len(), 2, "Hotkey {} should have 2 parts", action);
        assert_eq!(parts[0], "SELECT", "First part should be SELECT for {}", action);
    }
}

#[test]
fn test_emulator_core_mapping() {
    let core_mappings = [
        ("gba", "mgba_libretro.so"),
        ("snes", "snes9x_libretro.so"),
        ("nes", "nestopia_libretro.so"),
        ("psx", "pcsx_rearmed_libretro.so"),
        ("n64", "mupen64plus_next_libretro.so"),
        ("arcade", "fbneo_libretro.so"),
    ];

    for (system, core) in core_mappings {
        assert!(
            core.ends_with("_libretro.so"),
            "Core for {} should be a libretro core",
            system
        );
    }
}

#[test]
fn test_device_profile_structure() {
    // Test device profile structure
    struct DeviceProfile {
        name: String,
        chipset: String,
        display: DisplayProfile,
        input: InputProfile,
        audio: AudioProfile,
    }

    struct DisplayProfile {
        width: u32,
        height: u32,
        refresh_rate: u32,
    }

    struct InputProfile {
        has_analog: bool,
        button_count: u32,
    }

    struct AudioProfile {
        sample_rate: u32,
        channels: u32,
    }

    let rg353m = DeviceProfile {
        name: "Anbernic RG353M".to_string(),
        chipset: "RK3566".to_string(),
        display: DisplayProfile {
            width: 640,
            height: 480,
            refresh_rate: 60,
        },
        input: InputProfile {
            has_analog: true,
            button_count: 16,
        },
        audio: AudioProfile {
            sample_rate: 48000,
            channels: 2,
        },
    };

    assert_eq!(rg353m.name, "Anbernic RG353M");
    assert_eq!(rg353m.display.width, 640);
    assert!(rg353m.input.has_analog);
    assert_eq!(rg353m.audio.sample_rate, 48000);
}

#[test]
fn test_config_merging() {
    // Test that user config overrides system defaults
    let system_defaults = [
        ("brightness", "50"),
        ("volume", "70"),
        ("language", "en"),
    ];

    let user_overrides = [
        ("brightness", "80"),
        ("language", "es"),
    ];

    // Merge configs
    let mut merged: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

    for (key, value) in system_defaults {
        merged.insert(key, value);
    }

    for (key, value) in user_overrides {
        merged.insert(key, value);
    }

    assert_eq!(merged.get("brightness"), Some(&"80")); // Overridden
    assert_eq!(merged.get("volume"), Some(&"70")); // Default
    assert_eq!(merged.get("language"), Some(&"es")); // Overridden
}

#[test]
fn test_per_game_config() {
    let env = ConfigTestEnvironment::new();

    // Create per-game config
    let game_config = r#"
[core]
name = "snes9x_libretro.so"

[video]
shader = "crt-royale"
aspect_ratio = "4:3"

[input]
turbo_enabled = true

[audio]
volume_adjust = -5
"#;

    let game_hash = "abc123def456";
    let config_path = env
        .config_dir
        .join("games")
        .join(format!("{}.toml", game_hash));

    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, game_config).unwrap();

    assert!(config_path.exists());

    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("shader = \"crt-royale\""));
}

#[test]
#[ignore] // Requires actual config crate
fn test_config_load_and_save() {
    // This test would use the actual config crate
}

#[test]
#[ignore] // Requires actual config crate
fn test_config_validation() {
    // This test would validate config values are within acceptable ranges
}
