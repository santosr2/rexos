//! Main emulator launcher

use crate::{EmulatorError, GameSystem};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// Launch configuration
#[derive(Debug, Clone)]
pub struct LaunchConfig {
    /// ROM file path
    pub rom_path: PathBuf,

    /// Game system (auto-detected if None)
    pub system: Option<GameSystem>,

    /// Specific core to use (overrides default)
    pub core: Option<String>,

    /// Use 32-bit RetroArch
    pub use_32bit: bool,

    /// Custom RetroArch config
    pub config_path: Option<PathBuf>,

    /// State slot to load
    pub load_state: Option<u8>,

    /// Enable verbose logging
    pub verbose: bool,

    /// Additional arguments
    pub extra_args: Vec<String>,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            rom_path: PathBuf::new(),
            system: None,
            core: None,
            use_32bit: false,
            config_path: None,
            load_state: None,
            verbose: false,
            extra_args: Vec::new(),
        }
    }
}

impl LaunchConfig {
    /// Create config for a ROM path
    pub fn for_rom(rom_path: impl Into<PathBuf>) -> Self {
        let path = rom_path.into();

        // Auto-detect system from extension
        let system = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(GameSystem::from_extension);

        Self {
            rom_path: path,
            system,
            ..Default::default()
        }
    }

    /// Set the game system
    pub fn with_system(mut self, system: GameSystem) -> Self {
        self.system = Some(system);
        self
    }

    /// Set the core
    pub fn with_core(mut self, core: impl Into<String>) -> Self {
        self.core = Some(core.into());
        self
    }

    /// Use 32-bit RetroArch
    pub fn use_32bit(mut self) -> Self {
        self.use_32bit = true;
        self
    }

    /// Load a save state
    pub fn with_state(mut self, slot: u8) -> Self {
        self.load_state = Some(slot);
        self
    }
}

/// Launch result
#[derive(Debug)]
pub struct LaunchResult {
    /// Child process handle
    pub child: Child,

    /// PID of the launched process
    pub pid: u32,

    /// Core/emulator used
    pub emulator: String,
}

/// Main emulator launcher
pub struct EmulatorLauncher {
    /// RetroArch 64-bit path
    retroarch64: PathBuf,

    /// RetroArch 32-bit path
    retroarch32: PathBuf,

    /// Cores directory (64-bit)
    cores64_dir: PathBuf,

    /// Cores directory (32-bit)
    cores32_dir: PathBuf,

    /// Default RetroArch config
    config_path: PathBuf,
}

impl Default for EmulatorLauncher {
    fn default() -> Self {
        Self {
            retroarch64: PathBuf::from("/usr/bin/retroarch"),
            retroarch32: PathBuf::from("/usr/bin/retroarch32"),
            cores64_dir: PathBuf::from("/usr/lib/libretro"),
            cores32_dir: PathBuf::from("/usr/lib/libretro32"),
            config_path: PathBuf::from("/home/ark/.config/retroarch/retroarch.cfg"),
        }
    }
}

impl EmulatorLauncher {
    /// Create a new launcher with default paths
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom paths
    pub fn with_paths(
        retroarch64: impl Into<PathBuf>,
        retroarch32: impl Into<PathBuf>,
        cores64: impl Into<PathBuf>,
        cores32: impl Into<PathBuf>,
    ) -> Self {
        Self {
            retroarch64: retroarch64.into(),
            retroarch32: retroarch32.into(),
            cores64_dir: cores64.into(),
            cores32_dir: cores32.into(),
            config_path: PathBuf::from("/home/ark/.config/retroarch/retroarch.cfg"),
        }
    }

    /// Launch a game
    pub fn launch(&self, config: LaunchConfig) -> Result<LaunchResult, EmulatorError> {
        // Verify ROM exists
        if !config.rom_path.exists() {
            return Err(EmulatorError::RomNotFound(config.rom_path));
        }

        // Determine system
        let system = config
            .system
            .ok_or_else(|| EmulatorError::ConfigError("Could not determine game system".into()))?;

        // Determine core
        let core_name = config
            .core
            .unwrap_or_else(|| system.default_core().to_string());

        // Get paths based on 32/64 bit
        let (retroarch_path, cores_dir) = if config.use_32bit {
            (&self.retroarch32, &self.cores32_dir)
        } else {
            (&self.retroarch64, &self.cores64_dir)
        };

        // Build core path
        let core_path = cores_dir.join(format!("{}_libretro.so", core_name));

        // Verify core exists
        if !core_path.exists() {
            // Try alternative naming
            let alt_core_path = cores_dir.join(format!("libretro-{}.so", core_name));
            if !alt_core_path.exists() {
                return Err(EmulatorError::CoreNotFound(core_name));
            }
        }

        // Build command
        let mut cmd = Command::new(retroarch_path);

        // Core argument
        cmd.arg("-L").arg(&core_path);

        // Config argument
        let cfg = config.config_path.as_ref().unwrap_or(&self.config_path);
        if cfg.exists() {
            cmd.arg("--config").arg(cfg);
        }

        // Load state if requested
        if let Some(slot) = config.load_state {
            cmd.arg("-e").arg(slot.to_string());
        }

        // Verbose mode
        if config.verbose {
            cmd.arg("-v");
        }

        // Extra arguments
        for arg in &config.extra_args {
            cmd.arg(arg);
        }

        // ROM path (must be last)
        cmd.arg(&config.rom_path);

        // Set up stdio
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Launch
        tracing::info!(
            "Launching {} with core {}",
            config.rom_path.display(),
            core_name
        );

        let child = cmd
            .spawn()
            .map_err(|e| EmulatorError::LaunchFailed(format!("Failed to spawn process: {}", e)))?;

        let pid = child.id();

        Ok(LaunchResult {
            child,
            pid,
            emulator: core_name,
        })
    }

    /// Check if a core is available
    pub fn has_core(&self, core_name: &str, use_32bit: bool) -> bool {
        let cores_dir = if use_32bit {
            &self.cores32_dir
        } else {
            &self.cores64_dir
        };

        cores_dir
            .join(format!("{}_libretro.so", core_name))
            .exists()
    }

    /// List available cores
    pub fn list_cores(&self, use_32bit: bool) -> Vec<String> {
        let cores_dir = if use_32bit {
            &self.cores32_dir
        } else {
            &self.cores64_dir
        };

        let mut cores = Vec::new();

        if let Ok(entries) = std::fs::read_dir(cores_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("_libretro.so") {
                    let core_name = name.trim_end_matches("_libretro.so");
                    cores.push(core_name.to_string());
                }
            }
        }

        cores.sort();
        cores
    }

    /// Get RetroArch version
    pub fn retroarch_version(&self, use_32bit: bool) -> Option<String> {
        let path = if use_32bit {
            &self.retroarch32
        } else {
            &self.retroarch64
        };

        let output = Command::new(path).arg("--version").output().ok()?;

        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_config_builder() {
        let config = LaunchConfig::for_rom("/roms/gba/test.gba")
            .with_core("mgba")
            .with_state(1);

        assert!(config.rom_path.to_string_lossy().contains("test.gba"));
        assert_eq!(config.core, Some("mgba".to_string()));
        assert_eq!(config.load_state, Some(1));
    }

    #[test]
    fn test_system_detection() {
        let config = LaunchConfig::for_rom("/roms/gba/test.gba");
        assert_eq!(config.system, Some(GameSystem::GameBoyAdvance));
    }
}
