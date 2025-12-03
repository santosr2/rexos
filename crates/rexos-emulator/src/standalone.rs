//! Standalone emulator support

use crate::EmulatorError;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Information about a standalone emulator
#[derive(Debug, Clone)]
pub struct EmulatorInfo {
    /// Emulator name
    pub name: String,

    /// Display name
    pub display_name: String,

    /// Path to executable
    pub path: PathBuf,

    /// Supported systems
    pub systems: Vec<String>,

    /// Default command line arguments
    pub default_args: Vec<String>,

    /// Config directory
    pub config_dir: Option<PathBuf>,
}

impl EmulatorInfo {
    /// Create new emulator info
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            path: path.into(),
            systems: Vec::new(),
            default_args: Vec::new(),
            config_dir: None,
        }
    }

    /// Set display name
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    /// Add supported system
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.systems.push(system.into());
        self
    }

    /// Set default arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.default_args = args;
        self
    }

    /// Set config directory
    pub fn with_config_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config_dir = Some(dir.into());
        self
    }
}

/// Launcher for standalone emulators
pub struct StandaloneLauncher {
    emulators: Vec<EmulatorInfo>,
}

impl Default for StandaloneLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl StandaloneLauncher {
    /// Create a new launcher with default emulators
    pub fn new() -> Self {
        let mut launcher = Self {
            emulators: Vec::new(),
        };

        // Register default standalone emulators
        launcher.register_defaults();
        launcher
    }

    /// Register default standalone emulators found on ArkOS-style systems
    fn register_defaults(&mut self) {
        // PPSSPP for PSP
        if Path::new("/usr/bin/PPSSPPSDL").exists() {
            self.register(
                EmulatorInfo::new("ppsspp", "/usr/bin/PPSSPPSDL")
                    .with_display_name("PPSSPP")
                    .with_system("psp")
                    .with_args(vec!["--fullscreen".to_string()])
                    .with_config_dir("/home/ark/.config/ppsspp"),
            );
        }

        // DraStic for NDS
        if Path::new("/opt/drastic/drastic").exists() {
            self.register(
                EmulatorInfo::new("drastic", "/opt/drastic/drastic")
                    .with_display_name("DraStic")
                    .with_system("nds")
                    .with_config_dir("/opt/drastic"),
            );
        }

        // Amiberry for Amiga
        if Path::new("/usr/bin/amiberry").exists() {
            self.register(
                EmulatorInfo::new("amiberry", "/usr/bin/amiberry")
                    .with_display_name("Amiberry")
                    .with_system("amiga")
                    .with_config_dir("/home/ark/.config/amiberry"),
            );
        }

        // ScummVM
        if Path::new("/usr/bin/scummvm").exists() {
            self.register(
                EmulatorInfo::new("scummvm", "/usr/bin/scummvm")
                    .with_display_name("ScummVM")
                    .with_system("scummvm")
                    .with_args(vec!["--fullscreen".to_string()]),
            );
        }

        // DOSBox
        if Path::new("/usr/bin/dosbox").exists() {
            self.register(
                EmulatorInfo::new("dosbox", "/usr/bin/dosbox")
                    .with_display_name("DOSBox")
                    .with_system("dos")
                    .with_args(vec!["-fullscreen".to_string()]),
            );
        }

        // OpenBOR
        if Path::new("/usr/bin/OpenBOR").exists() {
            self.register(
                EmulatorInfo::new("openbor", "/usr/bin/OpenBOR")
                    .with_display_name("OpenBOR")
                    .with_system("openbor"),
            );
        }

        // Pico-8 / Fake08
        if Path::new("/usr/bin/fake08").exists() {
            self.register(
                EmulatorInfo::new("fake08", "/usr/bin/fake08")
                    .with_display_name("Fake08 (Pico-8)")
                    .with_system("pico8"),
            );
        }
    }

    /// Register an emulator
    pub fn register(&mut self, info: EmulatorInfo) {
        tracing::debug!("Registered standalone emulator: {}", info.name);
        self.emulators.push(info);
    }

    /// Get emulator by name
    pub fn get(&self, name: &str) -> Option<&EmulatorInfo> {
        self.emulators.iter().find(|e| e.name == name)
    }

    /// Get emulators for a system
    pub fn get_for_system(&self, system: &str) -> Vec<&EmulatorInfo> {
        self.emulators
            .iter()
            .filter(|e| e.systems.iter().any(|s| s == system))
            .collect()
    }

    /// List all registered emulators
    pub fn list(&self) -> &[EmulatorInfo] {
        &self.emulators
    }

    /// Check if emulator exists
    pub fn exists(&self, name: &str) -> bool {
        self.emulators
            .iter()
            .any(|e| e.name == name && e.path.exists())
    }

    /// Launch a standalone emulator
    pub fn launch(
        &self,
        emulator: &str,
        rom_path: &Path,
        extra_args: &[String],
    ) -> Result<Child, EmulatorError> {
        let info = self
            .get(emulator)
            .ok_or_else(|| EmulatorError::CoreNotFound(emulator.to_string()))?;

        if !info.path.exists() {
            return Err(EmulatorError::CoreNotFound(emulator.to_string()));
        }

        if !rom_path.exists() {
            return Err(EmulatorError::RomNotFound(rom_path.to_path_buf()));
        }

        let mut cmd = Command::new(&info.path);

        // Add default args
        for arg in &info.default_args {
            cmd.arg(arg);
        }

        // Add extra args
        for arg in extra_args {
            cmd.arg(arg);
        }

        // Add ROM path
        cmd.arg(rom_path);

        // Configure stdio
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        tracing::info!(
            "Launching {} with {}",
            info.display_name,
            rom_path.display()
        );

        cmd.spawn().map_err(|e| {
            EmulatorError::LaunchFailed(format!("Failed to spawn {}: {}", emulator, e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_info_builder() {
        let info = EmulatorInfo::new("test", "/usr/bin/test")
            .with_display_name("Test Emulator")
            .with_system("test_system")
            .with_args(vec!["--fullscreen".to_string()]);

        assert_eq!(info.name, "test");
        assert_eq!(info.display_name, "Test Emulator");
        assert!(info.systems.contains(&"test_system".to_string()));
    }

    #[test]
    fn test_standalone_launcher() {
        let launcher = StandaloneLauncher::new();
        // Should have registered defaults (may or may not exist on test system)
        // The list() method always returns a valid Vec, even if empty
        let _emulators = launcher.list();
    }
}
