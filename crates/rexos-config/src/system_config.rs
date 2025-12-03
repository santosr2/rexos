//! System-wide configuration

use serde::{Deserialize, Serialize};

/// Performance profile for power management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceProfile {
    /// Extended battery life, reduced performance
    Powersave,
    /// Balanced performance and battery
    #[default]
    Balanced,
    /// Maximum performance
    Performance,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable WiFi on boot
    #[serde(default)]
    pub wifi_enabled: bool,

    /// Enable SSH access
    #[serde(default)]
    pub ssh_enabled: bool,

    /// Enable Samba file sharing
    #[serde(default)]
    pub samba_enabled: bool,

    /// Enable web file browser
    #[serde(default)]
    pub filebrowser_enabled: bool,

    /// Hostname
    #[serde(default = "default_hostname")]
    pub hostname: String,
}

fn default_hostname() -> String {
    "rexos".to_string()
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            wifi_enabled: false,
            ssh_enabled: false, // Disabled by default like ArkOS
            samba_enabled: false,
            filebrowser_enabled: false,
            hostname: default_hostname(),
        }
    }
}

/// System-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Display brightness (0-255)
    #[serde(default = "default_brightness")]
    pub brightness: u8,

    /// Audio volume (0-100)
    #[serde(default = "default_volume")]
    pub volume: u8,

    /// Performance profile
    #[serde(default)]
    pub performance: PerformanceProfile,

    /// Auto-suspend timeout in minutes (0 = disabled)
    #[serde(default = "default_suspend_timeout")]
    pub suspend_timeout: u32,

    /// Low battery warning threshold (percentage)
    #[serde(default = "default_low_battery")]
    pub low_battery_threshold: u8,

    /// Enable low battery brightness flash (like ArkOS)
    #[serde(default = "default_true")]
    pub low_battery_warning: bool,

    /// Default EmulationStation frontend
    #[serde(default = "default_frontend")]
    pub frontend: String,

    /// Enable splash screen on boot
    #[serde(default = "default_true")]
    pub splash_screen: bool,

    /// Timezone
    #[serde(default = "default_timezone")]
    pub timezone: String,

    /// Locale
    #[serde(default = "default_locale")]
    pub locale: String,

    /// Network settings
    #[serde(default)]
    pub network: NetworkConfig,

    /// Enable auto-update checking
    #[serde(default)]
    pub auto_update_check: bool,

    /// Update channel (stable, beta, nightly)
    #[serde(default = "default_update_channel")]
    pub update_channel: String,
}

fn default_brightness() -> u8 {
    180
}

fn default_volume() -> u8 {
    70
}

fn default_suspend_timeout() -> u32 {
    5
}

fn default_low_battery() -> u8 {
    20
}

fn default_true() -> bool {
    true
}

fn default_frontend() -> String {
    "emulationstation".to_string()
}

fn default_timezone() -> String {
    "UTC".to_string()
}

fn default_locale() -> String {
    "en_US.UTF-8".to_string()
}

fn default_update_channel() -> String {
    "stable".to_string()
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            brightness: default_brightness(),
            volume: default_volume(),
            performance: PerformanceProfile::default(),
            suspend_timeout: default_suspend_timeout(),
            low_battery_threshold: default_low_battery(),
            low_battery_warning: true,
            frontend: default_frontend(),
            splash_screen: true,
            timezone: default_timezone(),
            locale: default_locale(),
            network: NetworkConfig::default(),
            auto_update_check: false,
            update_channel: default_update_channel(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = SystemConfig::default();
        assert_eq!(config.brightness, 180);
        assert_eq!(config.volume, 70);
        assert_eq!(config.performance, PerformanceProfile::Balanced);
    }

    #[test]
    fn test_performance_profile_serialize() {
        // Test serialization within a config struct
        let config = SystemConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("balanced")); // default is balanced
    }
}
