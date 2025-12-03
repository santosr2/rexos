//! Device profile configuration
//!
//! Stores hardware-specific configurations for different Anbernic devices

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::ConfigError;

/// Display configuration for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayProfile {
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Pixel format (RGB565, RGB888)
    pub format: String,
    /// Default refresh rate
    #[serde(default = "default_refresh")]
    pub refresh_rate: u32,
    /// Backlight sysfs path
    #[serde(default = "default_backlight_path")]
    pub backlight_path: String,
    /// Maximum brightness value
    #[serde(default = "default_max_brightness")]
    pub max_brightness: u32,
    /// Supports HDMI output
    #[serde(default)]
    pub hdmi_support: bool,
}

fn default_refresh() -> u32 {
    60
}

fn default_backlight_path() -> String {
    "/sys/class/backlight/backlight/brightness".to_string()
}

fn default_max_brightness() -> u32 {
    255
}

/// Input configuration for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputProfile {
    /// Input event device path
    #[serde(default = "default_input_path")]
    pub event_device: String,
    /// GPIO chip for buttons (if applicable)
    pub gpio_chip: Option<String>,
    /// Number of analog sticks
    #[serde(default)]
    pub analog_sticks: u8,
    /// Button mapping (physical to logical)
    #[serde(default)]
    pub button_map: HashMap<String, String>,
    /// Analog stick deadzone (0-32767)
    #[serde(default = "default_deadzone")]
    pub deadzone: u16,
}

fn default_input_path() -> String {
    "/dev/input/event0".to_string()
}

fn default_deadzone() -> u16 {
    4096
}

/// Audio configuration for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProfile {
    /// ALSA card name
    #[serde(default = "default_alsa_card")]
    pub alsa_card: String,
    /// ALSA mixer control name
    #[serde(default = "default_mixer")]
    pub mixer_control: String,
    /// Default sample rate
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    /// Supports headphone detection
    #[serde(default)]
    pub headphone_detect: bool,
    /// Headphone detection GPIO or sysfs path
    pub headphone_detect_path: Option<String>,
}

fn default_alsa_card() -> String {
    "default".to_string()
}

fn default_mixer() -> String {
    "Playback".to_string()
}

fn default_sample_rate() -> u32 {
    48000
}

/// Power configuration for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerProfile {
    /// Battery capacity in mAh
    pub battery_capacity: u32,
    /// Battery sysfs path
    #[serde(default = "default_battery_path")]
    pub battery_path: String,
    /// Charger sysfs path
    #[serde(default = "default_charger_path")]
    pub charger_path: String,
    /// CPU governor sysfs path
    #[serde(default = "default_governor_path")]
    pub governor_path: String,
    /// Available CPU frequencies
    #[serde(default)]
    pub cpu_frequencies: Vec<u32>,
    /// Supports sleep/suspend
    #[serde(default = "default_true")]
    pub sleep_support: bool,
}

fn default_battery_path() -> String {
    "/sys/class/power_supply/battery".to_string()
}

fn default_charger_path() -> String {
    "/sys/class/power_supply/usb".to_string()
}

fn default_governor_path() -> String {
    "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string()
}

fn default_true() -> bool {
    true
}

/// Complete device profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfileConfig {
    /// Device identifier
    pub id: String,
    /// Device display name
    pub name: String,
    /// Chipset (RK3566, RK3326, etc.)
    pub chipset: String,
    /// Architecture (aarch64, armv7)
    pub architecture: String,
    /// Display settings
    pub display: DisplayProfile,
    /// Input settings
    pub input: InputProfile,
    /// Audio settings
    pub audio: AudioProfile,
    /// Power settings
    pub power: PowerProfile,
    /// Device-specific quirks
    #[serde(default)]
    pub quirks: Vec<String>,
}

impl Default for DeviceProfileConfig {
    fn default() -> Self {
        // Default to RG353M profile
        Self::rg353m()
    }
}

impl DeviceProfileConfig {
    /// Create RG353M profile
    pub fn rg353m() -> Self {
        Self {
            id: "rg353m".to_string(),
            name: "Anbernic RG353M".to_string(),
            chipset: "RK3566".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplayProfile {
                width: 640,
                height: 480,
                format: "RGB565".to_string(),
                refresh_rate: 60,
                backlight_path: "/sys/class/backlight/backlight/brightness".to_string(),
                max_brightness: 255,
                hdmi_support: true,
            },
            input: InputProfile {
                event_device: "/dev/input/event3".to_string(),
                gpio_chip: Some("gpio0".to_string()),
                analog_sticks: 2,
                button_map: HashMap::new(),
                deadzone: 4096,
            },
            audio: AudioProfile {
                alsa_card: "rockchiphdmi".to_string(),
                mixer_control: "Playback".to_string(),
                sample_rate: 48000,
                headphone_detect: true,
                headphone_detect_path: Some("/sys/class/switch/h2w/state".to_string()),
            },
            power: PowerProfile {
                battery_capacity: 3500,
                battery_path: "/sys/class/power_supply/battery".to_string(),
                charger_path: "/sys/class/power_supply/ac".to_string(),
                governor_path: "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string(),
                cpu_frequencies: vec![
                    408000, 600000, 816000, 1008000, 1200000, 1416000, 1608000, 1800000,
                ],
                sleep_support: true,
            },
            quirks: vec![],
        }
    }

    /// Create RG353V profile
    pub fn rg353v() -> Self {
        let mut profile = Self::rg353m();
        profile.id = "rg353v".to_string();
        profile.name = "Anbernic RG353V".to_string();
        profile.display.width = 640;
        profile.display.height = 480;
        profile
    }

    /// Create RG35XX profile
    pub fn rg35xx() -> Self {
        Self {
            id: "rg35xx".to_string(),
            name: "Anbernic RG35XX".to_string(),
            chipset: "H700".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplayProfile {
                width: 640,
                height: 480,
                format: "RGB565".to_string(),
                refresh_rate: 60,
                backlight_path: "/sys/class/backlight/backlight/brightness".to_string(),
                max_brightness: 255,
                hdmi_support: false,
            },
            input: InputProfile {
                event_device: "/dev/input/event0".to_string(),
                gpio_chip: None,
                analog_sticks: 0,
                button_map: HashMap::new(),
                deadzone: 0,
            },
            audio: AudioProfile {
                alsa_card: "default".to_string(),
                mixer_control: "Playback".to_string(),
                sample_rate: 48000,
                headphone_detect: true,
                headphone_detect_path: None,
            },
            power: PowerProfile {
                battery_capacity: 2600,
                battery_path: "/sys/class/power_supply/battery".to_string(),
                charger_path: "/sys/class/power_supply/usb".to_string(),
                governor_path: "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string(),
                cpu_frequencies: vec![480000, 600000, 720000, 816000, 1008000, 1200000],
                sleep_support: true,
            },
            quirks: vec!["no_analog".to_string()],
        }
    }

    /// Create RGB30 profile
    pub fn rgb30() -> Self {
        Self {
            id: "rgb30".to_string(),
            name: "Powkiddy RGB30".to_string(),
            chipset: "RK3566".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplayProfile {
                width: 720,
                height: 720,
                format: "RGB565".to_string(),
                refresh_rate: 60,
                backlight_path: "/sys/class/backlight/backlight/brightness".to_string(),
                max_brightness: 255,
                hdmi_support: true,
            },
            input: InputProfile {
                event_device: "/dev/input/event3".to_string(),
                gpio_chip: Some("gpio0".to_string()),
                analog_sticks: 2,
                button_map: HashMap::new(),
                deadzone: 4096,
            },
            audio: AudioProfile {
                alsa_card: "default".to_string(),
                mixer_control: "Playback".to_string(),
                sample_rate: 48000,
                headphone_detect: true,
                headphone_detect_path: None,
            },
            power: PowerProfile {
                battery_capacity: 4100,
                battery_path: "/sys/class/power_supply/battery".to_string(),
                charger_path: "/sys/class/power_supply/usb".to_string(),
                governor_path: "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor".to_string(),
                cpu_frequencies: vec![
                    408000, 600000, 816000, 1008000, 1200000, 1416000, 1608000, 1800000,
                ],
                sleep_support: true,
            },
            quirks: vec!["square_display".to_string()],
        }
    }
}

/// Load device profiles from configuration directory
pub fn load_device_profiles(
    config_dir: &Path,
) -> Result<HashMap<String, DeviceProfileConfig>, ConfigError> {
    let profiles_dir = config_dir.join("devices");
    let mut profiles = HashMap::new();

    // Add built-in profiles
    profiles.insert("rg353m".to_string(), DeviceProfileConfig::rg353m());
    profiles.insert("rg353v".to_string(), DeviceProfileConfig::rg353v());
    profiles.insert("rg35xx".to_string(), DeviceProfileConfig::rg35xx());
    profiles.insert("rgb30".to_string(), DeviceProfileConfig::rgb30());

    // Load custom profiles from directory
    if profiles_dir.exists() {
        for entry in std::fs::read_dir(&profiles_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "toml") {
                let contents = std::fs::read_to_string(&path)?;
                let profile: DeviceProfileConfig = toml::from_str(&contents)?;
                profiles.insert(profile.id.clone(), profile);
            }
        }
    }

    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rg353m_profile() {
        let profile = DeviceProfileConfig::rg353m();
        assert_eq!(profile.chipset, "RK3566");
        assert_eq!(profile.display.width, 640);
        assert_eq!(profile.input.analog_sticks, 2);
    }

    #[test]
    fn test_profile_serialize() {
        let profile = DeviceProfileConfig::rg353m();
        let toml_str = toml::to_string_pretty(&profile).unwrap();
        assert!(toml_str.contains("RK3566"));
    }
}
