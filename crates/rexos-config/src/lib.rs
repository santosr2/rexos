//! Configuration management for RexOS
//!
//! Handles system configuration, device profiles, emulator settings, and user preferences.
//! Based on ArkOS configuration patterns with TOML-based config files.

mod device_profiles;
mod emulator_config;
mod hotkeys;
mod system_config;

pub use device_profiles::{DeviceProfileConfig, load_device_profiles};
pub use emulator_config::{CoreConfig, EmulatorConfig, SystemConfig as EmulatorSystemConfig};
pub use hotkeys::{Hotkey, HotkeyAction, HotkeyConfig};
pub use system_config::{NetworkConfig, PerformanceProfile, SystemConfig};

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

/// Standard configuration paths
pub const CONFIG_DIR: &str = "/etc/rexos";
pub const USER_CONFIG_DIR: &str = "/roms/.rexos";

/// Main RexOS configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RexOSConfig {
    #[serde(default)]
    pub system: SystemConfig,

    #[serde(default)]
    pub hotkeys: HotkeyConfig,

    #[serde(default)]
    pub emulators: EmulatorConfig,
}

impl RexOSConfig {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration from default locations
    pub fn load_default() -> Result<Self, ConfigError> {
        // Try user config first, then system config
        let user_config = Path::new(USER_CONFIG_DIR).join("config.toml");
        if user_config.exists() {
            return Self::load(&user_config);
        }

        let system_config = Path::new(CONFIG_DIR).join("config.toml");
        if system_config.exists() {
            return Self::load(&system_config);
        }

        // Return default config if no file exists
        tracing::warn!("No configuration file found, using defaults");
        Ok(Self::default())
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(self)?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, contents)?;
        tracing::info!("Configuration saved to {}", path.display());
        Ok(())
    }

    /// Save to default user configuration location
    pub fn save_default(&self) -> Result<(), ConfigError> {
        let user_config = Path::new(USER_CONFIG_DIR).join("config.toml");
        self.save(&user_config)
    }
}

/// Helper function to merge TOML values
pub fn merge_toml(base: &mut toml::Value, overlay: toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base_table), toml::Value::Table(overlay_table)) => {
            for (key, value) in overlay_table {
                if let Some(base_value) = base_table.get_mut(&key) {
                    merge_toml(base_value, value);
                } else {
                    base_table.insert(key, value);
                }
            }
        }
        (base, overlay) => *base = overlay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = RexOSConfig::default();
        assert!(config.system.brightness > 0);
        // Volume is u8, so it's always >= 0
        assert!(config.system.volume <= 100);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = RexOSConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: RexOSConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.system.brightness, parsed.system.brightness);
        assert_eq!(config.system.volume, parsed.system.volume);
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::NotFound(PathBuf::from("/etc/rexos/config.toml"));
        assert!(format!("{}", err).contains("not found"));

        let err = ConfigError::Invalid("test error".to_string());
        assert!(format!("{}", err).contains("Invalid"));

        let err = ConfigError::ParseError("parse failed".to_string());
        assert!(format!("{}", err).contains("parse"));
    }

    #[test]
    fn test_load_config_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[system]
brightness = 80
volume = 70

[hotkeys]

[emulators]
"#;
        write!(temp_file, "{}", config_content).unwrap();

        let config = RexOSConfig::load(temp_file.path()).unwrap();
        assert_eq!(config.system.brightness, 80);
        assert_eq!(config.system.volume, 70);
    }

    #[test]
    fn test_save_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = RexOSConfig::default();

        config.save(temp_file.path()).unwrap();

        // Verify the file was written and can be read back
        let loaded = RexOSConfig::load(temp_file.path()).unwrap();
        assert_eq!(config.system.brightness, loaded.system.brightness);
    }

    #[test]
    fn test_merge_toml_tables() {
        let base_table = toml::toml! {
            [system]
            brightness = 50
            volume = 60
        };
        let mut base = toml::Value::Table(base_table);

        let overlay_table = toml::toml! {
            [system]
            brightness = 80
        };
        let overlay = toml::Value::Table(overlay_table);

        merge_toml(&mut base, overlay);

        let table = base.as_table().unwrap();
        let system = table.get("system").unwrap().as_table().unwrap();
        assert_eq!(system.get("brightness").unwrap().as_integer(), Some(80));
        assert_eq!(system.get("volume").unwrap().as_integer(), Some(60));
    }

    #[test]
    fn test_merge_toml_replaces_non_tables() {
        let mut base = toml::Value::Integer(42);
        let overlay = toml::Value::Integer(100);

        merge_toml(&mut base, overlay);

        assert_eq!(base.as_integer(), Some(100));
    }

    #[test]
    fn test_config_pretty_print() {
        let config = RexOSConfig::default();
        let pretty = toml::to_string_pretty(&config).unwrap();

        // Pretty printed TOML should have sections
        assert!(pretty.contains("[system]") || pretty.contains("system"));
    }

    #[test]
    fn test_constants() {
        assert_eq!(CONFIG_DIR, "/etc/rexos");
        assert_eq!(USER_CONFIG_DIR, "/roms/.rexos");
    }
}
