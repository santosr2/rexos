//! Device detection and configuration
//!
//! Handles hardware detection using Linux sysfs and device tree interfaces.
//! Based on ArkOS device detection patterns for Rockchip-based handhelds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Failed to detect device")]
    DetectionFailed,

    #[error("Unsupported device: {0}")]
    UnsupportedDevice(String),

    #[error("Hardware initialization failed: {0}")]
    InitializationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Device profile containing hardware specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub id: String,
    pub name: String,
    pub chipset: String,
    pub architecture: String,
    pub display: DisplaySpec,
    pub buttons: Vec<String>,
    pub analog_sticks: u8,
    pub battery_capacity: u32,
    #[serde(default)]
    pub quirks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySpec {
    pub width: u32,
    pub height: u32,
    pub format: String,
    #[serde(default = "default_refresh")]
    pub refresh_rate: u32,
}

fn default_refresh() -> u32 {
    60
}

/// System information gathered from sysfs
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub model: String,
    pub compatible: Vec<String>,
    pub serial: Option<String>,
    pub cpu_model: Option<String>,
    pub cpu_count: u32,
    pub total_memory_kb: u64,
}

/// Main device structure
pub struct Device {
    profile: DeviceProfile,
    system_info: SystemInfo,
}

impl Device {
    /// Auto-detect the device based on hardware identifiers
    pub fn detect() -> Result<Self, DeviceError> {
        tracing::info!("Attempting to detect device...");

        // Gather system information from sysfs
        let system_info = Self::gather_system_info()?;
        tracing::info!("Detected model: {}", system_info.model);
        tracing::debug!("Compatible: {:?}", system_info.compatible);

        // Match to a known profile
        let profile = Self::match_profile(&system_info)?;
        tracing::info!("Matched device profile: {}", profile.name);

        Ok(Self {
            profile,
            system_info,
        })
    }

    /// Create device from a specific profile file
    pub fn from_profile_file(path: &Path) -> Result<Self, DeviceError> {
        let contents = fs::read_to_string(path)?;
        let profile: DeviceProfile = toml::from_str(&contents)
            .map_err(|e| DeviceError::InitializationFailed(e.to_string()))?;

        let system_info = Self::gather_system_info().unwrap_or_else(|_| SystemInfo {
            model: profile.name.clone(),
            compatible: vec![],
            serial: None,
            cpu_model: None,
            cpu_count: 4,
            total_memory_kb: 1024 * 1024,
        });

        Ok(Self {
            profile,
            system_info,
        })
    }

    /// Get device profile
    pub fn profile(&self) -> &DeviceProfile {
        &self.profile
    }

    /// Get system information
    pub fn system_info(&self) -> &SystemInfo {
        &self.system_info
    }

    /// Check if this is an RK3566-based device
    pub fn is_rk3566(&self) -> bool {
        self.profile.chipset == "RK3566"
    }

    /// Check if this is an RK3326-based device
    pub fn is_rk3326(&self) -> bool {
        self.profile.chipset == "RK3326"
    }

    /// Check if device has a specific quirk
    pub fn has_quirk(&self, quirk: &str) -> bool {
        self.profile.quirks.iter().any(|q| q == quirk)
    }

    /// Gather system information from sysfs and device tree
    fn gather_system_info() -> Result<SystemInfo, DeviceError> {
        // Read device model from device tree
        let model = Self::read_device_tree_string("/sys/firmware/devicetree/base/model")
            .or_else(|_| Self::read_device_tree_string("/proc/device-tree/model"))
            .or_else(|_| Self::read_file_trimmed("/etc/device-model"))
            .unwrap_or_else(|_| "Unknown Device".to_string());

        // Read compatible strings
        let compatible = Self::read_device_tree_strings("/sys/firmware/devicetree/base/compatible")
            .unwrap_or_default();

        // Read serial number
        let serial = Self::read_device_tree_string("/sys/firmware/devicetree/base/serial-number").ok();

        // Read CPU info
        let cpu_info = Self::parse_cpuinfo();
        let cpu_model = cpu_info.get("model name").cloned()
            .or_else(|| cpu_info.get("Hardware").cloned());

        // Count CPUs
        let cpu_count = Self::count_cpus();

        // Read memory info
        let total_memory_kb = Self::read_meminfo_total().unwrap_or(1024 * 1024);

        Ok(SystemInfo {
            model,
            compatible,
            serial,
            cpu_model,
            cpu_count,
            total_memory_kb,
        })
    }

    /// Read a null-terminated string from device tree
    fn read_device_tree_string(path: &str) -> Result<String, DeviceError> {
        let bytes = fs::read(path)?;
        // Device tree strings are null-terminated
        let s = String::from_utf8_lossy(&bytes);
        Ok(s.trim_matches('\0').trim().to_string())
    }

    /// Read multiple null-separated strings from device tree
    fn read_device_tree_strings(path: &str) -> Result<Vec<String>, DeviceError> {
        let bytes = fs::read(path)?;
        let strings: Vec<String> = bytes
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect();
        Ok(strings)
    }

    /// Read and trim a text file
    fn read_file_trimmed(path: &str) -> Result<String, DeviceError> {
        Ok(fs::read_to_string(path)?.trim().to_string())
    }

    /// Parse /proc/cpuinfo
    fn parse_cpuinfo() -> HashMap<String, String> {
        let mut info = HashMap::new();

        if let Ok(contents) = fs::read_to_string("/proc/cpuinfo") {
            for line in contents.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    info.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        info
    }

    /// Count online CPUs
    fn count_cpus() -> u32 {
        // Try to read from sysfs
        let online_path = "/sys/devices/system/cpu/online";
        if let Ok(contents) = fs::read_to_string(online_path) {
            // Format is like "0-3" for 4 cores
            if let Some(range) = contents.trim().split('-').last() {
                if let Ok(max) = range.parse::<u32>() {
                    return max + 1;
                }
            }
        }

        // Fallback: count cpu directories
        let cpu_dir = Path::new("/sys/devices/system/cpu");
        if let Ok(entries) = fs::read_dir(cpu_dir) {
            return entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with("cpu")
                        && e.file_name()
                            .to_string_lossy()
                            .chars()
                            .nth(3)
                            .map_or(false, |c| c.is_ascii_digit())
                })
                .count() as u32;
        }

        4 // Default assumption
    }

    /// Read total memory from /proc/meminfo
    fn read_meminfo_total() -> Result<u64, DeviceError> {
        let contents = fs::read_to_string("/proc/meminfo")?;
        for line in contents.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return Ok(kb);
                    }
                }
            }
        }
        Err(DeviceError::DetectionFailed)
    }

    /// Match system info to a known device profile
    fn match_profile(info: &SystemInfo) -> Result<DeviceProfile, DeviceError> {
        let model_lower = info.model.to_lowercase();
        let compatible_str = info.compatible.join(" ").to_lowercase();

        // RG353M/V/VS detection (RK3566)
        if model_lower.contains("rg353") || compatible_str.contains("rg353") {
            let variant = if model_lower.contains("353m") {
                "RG353M"
            } else if model_lower.contains("353vs") {
                "RG353VS"
            } else {
                "RG353V"
            };

            return Ok(Self::profile_rg353(variant));
        }

        // RGB30 detection (RK3566)
        if model_lower.contains("rgb30") || compatible_str.contains("rgb30") {
            return Ok(Self::profile_rgb30());
        }

        // RG503 detection (RK3566)
        if model_lower.contains("rg503") || compatible_str.contains("rg503") {
            return Ok(Self::profile_rg503());
        }

        // RG351 series detection (RK3326)
        if model_lower.contains("rg351") || compatible_str.contains("rg351") {
            let variant = if model_lower.contains("351mp") {
                "RG351MP"
            } else if model_lower.contains("351v") {
                "RG351V"
            } else {
                "RG351P"
            };

            return Ok(Self::profile_rg351(variant));
        }

        // RG35XX detection (Allwinner H700)
        if model_lower.contains("rg35xx") || model_lower.contains("h700") {
            return Ok(Self::profile_rg35xx());
        }

        // Generic RK3566 fallback
        if compatible_str.contains("rk3566") {
            tracing::warn!("Unknown RK3566 device, using generic profile");
            return Ok(Self::profile_generic_rk3566());
        }

        // Generic RK3326 fallback
        if compatible_str.contains("rk3326") {
            tracing::warn!("Unknown RK3326 device, using generic profile");
            return Ok(Self::profile_generic_rk3326());
        }

        Err(DeviceError::UnsupportedDevice(info.model.clone()))
    }

    /// RG353 series profile (RK3566)
    fn profile_rg353(variant: &str) -> DeviceProfile {
        DeviceProfile {
            id: variant.to_lowercase(),
            name: format!("Anbernic {}", variant),
            chipset: "RK3566".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width: 640,
                height: 480,
                format: "RGB565".to_string(),
                refresh_rate: 60,
            },
            buttons: vec![
                "up", "down", "left", "right",
                "a", "b", "x", "y",
                "l1", "l2", "r1", "r2",
                "start", "select",
            ].into_iter().map(String::from).collect(),
            analog_sticks: 2,
            battery_capacity: 3500,
            quirks: vec![],
        }
    }

    /// RGB30 profile (RK3566, square display)
    fn profile_rgb30() -> DeviceProfile {
        DeviceProfile {
            id: "rgb30".to_string(),
            name: "Powkiddy RGB30".to_string(),
            chipset: "RK3566".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width: 720,
                height: 720,
                format: "RGB565".to_string(),
                refresh_rate: 60,
            },
            buttons: vec![
                "up", "down", "left", "right",
                "a", "b", "x", "y",
                "l1", "l2", "r1", "r2",
                "start", "select",
            ].into_iter().map(String::from).collect(),
            analog_sticks: 2,
            battery_capacity: 4100,
            quirks: vec!["square_display".to_string()],
        }
    }

    /// RG503 profile (RK3566, OLED display)
    fn profile_rg503() -> DeviceProfile {
        DeviceProfile {
            id: "rg503".to_string(),
            name: "Anbernic RG503".to_string(),
            chipset: "RK3566".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width: 960,
                height: 544,
                format: "RGB888".to_string(),
                refresh_rate: 60,
            },
            buttons: vec![
                "up", "down", "left", "right",
                "a", "b", "x", "y",
                "l1", "l2", "r1", "r2",
                "start", "select",
            ].into_iter().map(String::from).collect(),
            analog_sticks: 2,
            battery_capacity: 3500,
            quirks: vec!["oled_display".to_string()],
        }
    }

    /// RG351 series profile (RK3326)
    fn profile_rg351(variant: &str) -> DeviceProfile {
        let (width, height) = match variant {
            "RG351V" => (640, 480),
            "RG351MP" => (640, 480),
            _ => (480, 320),
        };

        DeviceProfile {
            id: variant.to_lowercase(),
            name: format!("Anbernic {}", variant),
            chipset: "RK3326".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width,
                height,
                format: "RGB565".to_string(),
                refresh_rate: 60,
            },
            buttons: vec![
                "up", "down", "left", "right",
                "a", "b", "x", "y",
                "l1", "l2", "r1", "r2",
                "start", "select",
            ].into_iter().map(String::from).collect(),
            analog_sticks: if variant == "RG351V" { 1 } else { 2 },
            battery_capacity: 3500,
            quirks: vec![],
        }
    }

    /// RG35XX profile (Allwinner H700)
    fn profile_rg35xx() -> DeviceProfile {
        DeviceProfile {
            id: "rg35xx".to_string(),
            name: "Anbernic RG35XX".to_string(),
            chipset: "H700".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width: 640,
                height: 480,
                format: "RGB565".to_string(),
                refresh_rate: 60,
            },
            buttons: vec![
                "up", "down", "left", "right",
                "a", "b", "x", "y",
                "l1", "r1",
                "start", "select",
            ].into_iter().map(String::from).collect(),
            analog_sticks: 0,
            battery_capacity: 2600,
            quirks: vec!["no_analog".to_string(), "no_l2r2".to_string()],
        }
    }

    /// Generic RK3566 profile
    fn profile_generic_rk3566() -> DeviceProfile {
        DeviceProfile {
            id: "generic_rk3566".to_string(),
            name: "Generic RK3566 Device".to_string(),
            chipset: "RK3566".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width: 640,
                height: 480,
                format: "RGB565".to_string(),
                refresh_rate: 60,
            },
            buttons: vec![
                "up", "down", "left", "right",
                "a", "b", "x", "y",
                "l1", "l2", "r1", "r2",
                "start", "select",
            ].into_iter().map(String::from).collect(),
            analog_sticks: 2,
            battery_capacity: 3500,
            quirks: vec!["generic".to_string()],
        }
    }

    /// Generic RK3326 profile
    fn profile_generic_rk3326() -> DeviceProfile {
        DeviceProfile {
            id: "generic_rk3326".to_string(),
            name: "Generic RK3326 Device".to_string(),
            chipset: "RK3326".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width: 480,
                height: 320,
                format: "RGB565".to_string(),
                refresh_rate: 60,
            },
            buttons: vec![
                "up", "down", "left", "right",
                "a", "b", "x", "y",
                "l1", "l2", "r1", "r2",
                "start", "select",
            ].into_iter().map(String::from).collect(),
            analog_sticks: 2,
            battery_capacity: 3500,
            quirks: vec!["generic".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_profile_creation() {
        let profile = DeviceProfile {
            id: "test".to_string(),
            name: "Test Device".to_string(),
            chipset: "Test Chip".to_string(),
            architecture: "aarch64".to_string(),
            display: DisplaySpec {
                width: 640,
                height: 480,
                format: "RGB565".to_string(),
                refresh_rate: 60,
            },
            buttons: vec!["a".to_string(), "b".to_string()],
            analog_sticks: 2,
            battery_capacity: 3500,
            quirks: vec![],
        };

        assert_eq!(profile.name, "Test Device");
        assert_eq!(profile.display.width, 640);
    }
}
