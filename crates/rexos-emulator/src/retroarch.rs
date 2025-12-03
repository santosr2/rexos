//! RetroArch-specific functionality

use crate::EmulatorError;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Information about a RetroArch core
#[derive(Debug, Clone)]
pub struct CoreInfo {
    pub name: String,
    pub display_name: String,
    pub path: PathBuf,
    pub supported_extensions: Vec<String>,
    pub needs_bios: bool,
    pub required_bios: Vec<String>,
    pub version: Option<String>,
}

/// RetroArch launcher with configuration management
pub struct RetroArchLauncher {
    /// Path to RetroArch executable
    pub path: PathBuf,

    /// Path to cores directory
    pub cores_dir: PathBuf,

    /// Path to config directory
    pub config_dir: PathBuf,

    /// Core info cache
    core_info: HashMap<String, CoreInfo>,
}

impl RetroArchLauncher {
    /// Create a new RetroArch launcher
    pub fn new(path: impl Into<PathBuf>, cores_dir: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            cores_dir: cores_dir.into(),
            config_dir: PathBuf::from("/home/ark/.config/retroarch"),
            core_info: HashMap::new(),
        }
    }

    /// Scan and cache core information
    pub fn scan_cores(&mut self) -> Result<(), EmulatorError> {
        self.core_info.clear();

        if !self.cores_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.cores_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with("_libretro.so") {
                    let core_name = name.trim_end_matches("_libretro.so").to_string();

                    // Try to load .info file
                    let info = self.load_core_info(&core_name, &path);
                    self.core_info.insert(core_name, info);
                }
            }
        }

        tracing::info!("Found {} RetroArch cores", self.core_info.len());
        Ok(())
    }

    /// Load core info from .info file or create default
    fn load_core_info(&self, name: &str, lib_path: &Path) -> CoreInfo {
        // Check for .info file in info directory
        let info_path = self.config_dir
            .join("cores")
            .join(format!("{}_libretro.info", name));

        let mut info = CoreInfo {
            name: name.to_string(),
            display_name: name.to_string(),
            path: lib_path.to_path_buf(),
            supported_extensions: Vec::new(),
            needs_bios: false,
            required_bios: Vec::new(),
            version: None,
        };

        if let Ok(contents) = fs::read_to_string(&info_path) {
            for line in contents.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');

                    match key {
                        "display_name" => info.display_name = value.to_string(),
                        "supported_extensions" => {
                            info.supported_extensions = value
                                .split('|')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                        "firmware_count" => {
                            if let Ok(count) = value.parse::<u32>() {
                                info.needs_bios = count > 0;
                            }
                        }
                        "display_version" => info.version = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        }

        info
    }

    /// Get core info by name
    pub fn get_core(&self, name: &str) -> Option<&CoreInfo> {
        self.core_info.get(name)
    }

    /// Get all cores
    pub fn cores(&self) -> &HashMap<String, CoreInfo> {
        &self.core_info
    }

    /// Find cores supporting an extension
    pub fn find_cores_for_extension(&self, ext: &str) -> Vec<&CoreInfo> {
        let ext_lower = ext.to_lowercase();
        self.core_info
            .values()
            .filter(|core| {
                core.supported_extensions
                    .iter()
                    .any(|e| e.to_lowercase() == ext_lower)
            })
            .collect()
    }

    /// Get path to core-specific config override
    pub fn core_config_path(&self, core_name: &str) -> PathBuf {
        self.config_dir.join("config").join(format!("{}.cfg", core_name))
    }

    /// Get path to game-specific config override
    pub fn game_config_path(&self, rom_path: &Path) -> PathBuf {
        let game_name = rom_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        self.config_dir.join("config").join(format!("{}.cfg", game_name))
    }

    /// Read a RetroArch config value
    pub fn read_config(&self, key: &str) -> Option<String> {
        let config_path = self.config_dir.join("retroarch.cfg");
        let contents = fs::read_to_string(&config_path).ok()?;

        for line in contents.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k.trim() == key {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }

        None
    }

    /// Write a RetroArch config value
    pub fn write_config(&self, key: &str, value: &str) -> Result<(), EmulatorError> {
        let config_path = self.config_dir.join("retroarch.cfg");

        let contents = fs::read_to_string(&config_path).unwrap_or_default();
        let mut lines: Vec<String> = contents.lines().map(String::from).collect();

        // Find and update existing key, or append
        let mut found = false;
        for line in &mut lines {
            if line.starts_with(key) && line.contains('=') {
                *line = format!("{} = \"{}\"", key, value);
                found = true;
                break;
            }
        }

        if !found {
            lines.push(format!("{} = \"{}\"", key, value));
        }

        fs::write(&config_path, lines.join("\n"))?;
        Ok(())
    }

    /// Get save state path for a game
    pub fn save_state_path(&self, rom_path: &Path, slot: u8) -> PathBuf {
        let game_name = rom_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let states_dir = self.read_config("savestate_directory")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config_dir.join("states"));

        states_dir.join(format!("{}.state{}", game_name, if slot == 0 { String::new() } else { slot.to_string() }))
    }

    /// Get SRAM save path for a game
    pub fn save_path(&self, rom_path: &Path) -> PathBuf {
        let game_name = rom_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let saves_dir = self.read_config("savefile_directory")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.config_dir.join("saves"));

        saves_dir.join(format!("{}.srm", game_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_info_default() {
        let info = CoreInfo {
            name: "test".to_string(),
            display_name: "Test Core".to_string(),
            path: PathBuf::from("/cores/test_libretro.so"),
            supported_extensions: vec!["bin".to_string()],
            needs_bios: false,
            required_bios: vec![],
            version: None,
        };

        assert_eq!(info.name, "test");
        assert!(info.supported_extensions.contains(&"bin".to_string()));
    }
}
