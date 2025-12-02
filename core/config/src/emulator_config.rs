//! Emulator and core configuration
//!
//! Manages RetroArch cores and standalone emulator settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for a RetroArch core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Core library name (e.g., "mgba_libretro.so")
    pub library: String,

    /// Display name
    #[serde(default)]
    pub name: String,

    /// Supported file extensions
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Whether this core needs BIOS files
    #[serde(default)]
    pub needs_bios: bool,

    /// Required BIOS files
    #[serde(default)]
    pub bios_files: Vec<String>,

    /// Core-specific options
    #[serde(default)]
    pub options: HashMap<String, String>,
}

/// Configuration for a game system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// System name (e.g., "Game Boy Advance")
    pub name: String,

    /// Short name for directories (e.g., "gba")
    pub short_name: String,

    /// Default core/emulator
    pub default_core: String,

    /// Alternative cores
    #[serde(default)]
    pub alternative_cores: Vec<String>,

    /// File extensions for this system
    #[serde(default)]
    pub extensions: Vec<String>,

    /// ROM directory path (relative to /roms)
    #[serde(default)]
    pub rom_path: Option<String>,

    /// System-specific settings
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

/// Global emulator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    /// RetroArch executable path (32-bit)
    #[serde(default = "default_retroarch32")]
    pub retroarch32_path: PathBuf,

    /// RetroArch executable path (64-bit)
    #[serde(default = "default_retroarch64")]
    pub retroarch64_path: PathBuf,

    /// Default RetroArch to use
    #[serde(default = "default_retroarch_default")]
    pub default_retroarch: String,

    /// Cores directory (64-bit)
    #[serde(default = "default_cores64")]
    pub cores64_path: PathBuf,

    /// Cores directory (32-bit)
    #[serde(default = "default_cores32")]
    pub cores32_path: PathBuf,

    /// RetroArch config directory
    #[serde(default = "default_retroarch_config")]
    pub config_path: PathBuf,

    /// System configurations
    #[serde(default = "default_systems")]
    pub systems: HashMap<String, SystemConfig>,

    /// Core configurations
    #[serde(default)]
    pub cores: HashMap<String, CoreConfig>,

    /// Standalone emulators
    #[serde(default = "default_standalone")]
    pub standalone: HashMap<String, StandaloneEmulator>,

    /// Auto-save state on exit
    #[serde(default = "default_true")]
    pub auto_save: bool,

    /// Auto-load state on start
    #[serde(default)]
    pub auto_load: bool,

    /// Show FPS by default
    #[serde(default)]
    pub show_fps: bool,

    /// Enable shaders
    #[serde(default = "default_true")]
    pub shaders_enabled: bool,

    /// Default shader preset
    #[serde(default)]
    pub default_shader: Option<String>,
}

/// Configuration for a standalone emulator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneEmulator {
    /// Executable path
    pub path: PathBuf,

    /// Display name
    pub name: String,

    /// Supported systems
    pub systems: Vec<String>,

    /// Command line arguments template
    #[serde(default)]
    pub args: Vec<String>,

    /// Configuration directory
    #[serde(default)]
    pub config_dir: Option<PathBuf>,
}

fn default_retroarch32() -> PathBuf {
    PathBuf::from("/usr/bin/retroarch32")
}

fn default_retroarch64() -> PathBuf {
    PathBuf::from("/usr/bin/retroarch")
}

fn default_retroarch_default() -> String {
    "64".to_string()
}

fn default_cores64() -> PathBuf {
    PathBuf::from("/usr/lib/libretro")
}

fn default_cores32() -> PathBuf {
    PathBuf::from("/usr/lib/libretro32")
}

fn default_retroarch_config() -> PathBuf {
    PathBuf::from("/home/ark/.config/retroarch")
}

fn default_true() -> bool {
    true
}

fn default_systems() -> HashMap<String, SystemConfig> {
    let mut systems = HashMap::new();

    // Game Boy
    systems.insert(
        "gb".to_string(),
        SystemConfig {
            name: "Game Boy".to_string(),
            short_name: "gb".to_string(),
            default_core: "gambatte".to_string(),
            alternative_cores: vec!["sameboy".to_string(), "gearboy".to_string()],
            extensions: vec!["gb".to_string(), "gbc".to_string()],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    // Game Boy Advance
    systems.insert(
        "gba".to_string(),
        SystemConfig {
            name: "Game Boy Advance".to_string(),
            short_name: "gba".to_string(),
            default_core: "mgba".to_string(),
            alternative_cores: vec!["vba_next".to_string(), "gpsp".to_string()],
            extensions: vec!["gba".to_string()],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    // NES
    systems.insert(
        "nes".to_string(),
        SystemConfig {
            name: "Nintendo Entertainment System".to_string(),
            short_name: "nes".to_string(),
            default_core: "fceumm".to_string(),
            alternative_cores: vec!["nestopia".to_string(), "quicknes".to_string()],
            extensions: vec!["nes".to_string(), "fds".to_string()],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    // SNES
    systems.insert(
        "snes".to_string(),
        SystemConfig {
            name: "Super Nintendo".to_string(),
            short_name: "snes".to_string(),
            default_core: "snes9x".to_string(),
            alternative_cores: vec!["snes9x2010".to_string(), "bsnes".to_string()],
            extensions: vec!["smc".to_string(), "sfc".to_string()],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    // PlayStation
    systems.insert(
        "psx".to_string(),
        SystemConfig {
            name: "Sony PlayStation".to_string(),
            short_name: "psx".to_string(),
            default_core: "pcsx_rearmed".to_string(),
            alternative_cores: vec!["duckstation".to_string()],
            extensions: vec![
                "bin".to_string(),
                "cue".to_string(),
                "chd".to_string(),
                "pbp".to_string(),
            ],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    // N64
    systems.insert(
        "n64".to_string(),
        SystemConfig {
            name: "Nintendo 64".to_string(),
            short_name: "n64".to_string(),
            default_core: "mupen64plus_next".to_string(),
            alternative_cores: vec!["parallel_n64".to_string()],
            extensions: vec!["n64".to_string(), "z64".to_string(), "v64".to_string()],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    // Sega Genesis
    systems.insert(
        "genesis".to_string(),
        SystemConfig {
            name: "Sega Genesis".to_string(),
            short_name: "genesis".to_string(),
            default_core: "genesis_plus_gx".to_string(),
            alternative_cores: vec!["picodrive".to_string()],
            extensions: vec!["md".to_string(), "bin".to_string(), "gen".to_string()],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    // PSP
    systems.insert(
        "psp".to_string(),
        SystemConfig {
            name: "Sony PSP".to_string(),
            short_name: "psp".to_string(),
            default_core: "ppsspp".to_string(),
            alternative_cores: vec![],
            extensions: vec!["iso".to_string(), "cso".to_string(), "pbp".to_string()],
            rom_path: None,
            settings: HashMap::new(),
        },
    );

    systems
}

fn default_standalone() -> HashMap<String, StandaloneEmulator> {
    let mut standalone = HashMap::new();

    // PPSSPP (standalone is often better for PSP)
    standalone.insert(
        "ppsspp".to_string(),
        StandaloneEmulator {
            path: PathBuf::from("/usr/bin/PPSSPPSDL"),
            name: "PPSSPP".to_string(),
            systems: vec!["psp".to_string()],
            args: vec!["--fullscreen".to_string()],
            config_dir: Some(PathBuf::from("/home/ark/.config/ppsspp")),
        },
    );

    // DraStic for DS (if available)
    standalone.insert(
        "drastic".to_string(),
        StandaloneEmulator {
            path: PathBuf::from("/opt/drastic/drastic"),
            name: "DraStic".to_string(),
            systems: vec!["nds".to_string()],
            args: vec![],
            config_dir: Some(PathBuf::from("/opt/drastic")),
        },
    );

    standalone
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            retroarch32_path: default_retroarch32(),
            retroarch64_path: default_retroarch64(),
            default_retroarch: default_retroarch_default(),
            cores64_path: default_cores64(),
            cores32_path: default_cores32(),
            config_path: default_retroarch_config(),
            systems: default_systems(),
            cores: HashMap::new(),
            standalone: default_standalone(),
            auto_save: true,
            auto_load: false,
            show_fps: false,
            shaders_enabled: true,
            default_shader: None,
        }
    }
}

impl EmulatorConfig {
    /// Get the system config for a system short name
    pub fn get_system(&self, short_name: &str) -> Option<&SystemConfig> {
        self.systems.get(short_name)
    }

    /// Get the default core library path for a system
    pub fn get_core_path(&self, system: &str) -> Option<PathBuf> {
        let sys_config = self.systems.get(system)?;
        let core_name = &sys_config.default_core;

        // Determine if 32-bit or 64-bit based on system
        let use_32bit = matches!(system, "psx" | "dreamcast");

        let cores_dir = if use_32bit {
            &self.cores32_path
        } else {
            &self.cores64_path
        };

        Some(cores_dir.join(format!("{}_libretro.so", core_name)))
    }

    /// Find the system for a file extension
    pub fn find_system_for_extension(&self, ext: &str) -> Option<&SystemConfig> {
        let ext_lower = ext.to_lowercase();
        self.systems
            .values()
            .find(|sys| sys.extensions.iter().any(|e| e == &ext_lower))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_systems() {
        let config = EmulatorConfig::default();
        assert!(config.systems.contains_key("gba"));
        assert!(config.systems.contains_key("nes"));
    }

    #[test]
    fn test_get_core_path() {
        let config = EmulatorConfig::default();
        let path = config.get_core_path("gba");
        assert!(path.is_some());
        assert!(path.unwrap().to_string_lossy().contains("mgba"));
    }

    #[test]
    fn test_find_system_for_extension() {
        let config = EmulatorConfig::default();
        let system = config.find_system_for_extension("gba");
        assert!(system.is_some());
        assert_eq!(system.unwrap().short_name, "gba");
    }
}
