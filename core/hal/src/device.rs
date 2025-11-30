//! Device detection and configuration

use serde::{Deserialize, Serialize};
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
    pub name: String,
    pub chipset: String,
    pub display: DisplaySpec,
    pub buttons: Vec<String>,
    pub analog_sticks: u8,
    pub battery_capacity: u32, // mAh
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySpec {
    pub width: u32,
    pub height: u32,
    pub format: String, // RGB565, RGB888, etc.
}

/// Main device structure
pub struct Device {
    profile: DeviceProfile,
}

impl Device {
    /// Auto-detect the device based on hardware identifiers
    pub fn detect() -> Result<Self, DeviceError> {
        tracing::info!("Attempting to detect device...");
        
        // Try to read device tree or other identifiers
        let device_name = Self::read_device_identifier()
            .unwrap_or_else(|_| "unknown".to_string());
        
        tracing::info!("Device identifier: {}", device_name);
        
        // Load profile based on detection
        let profile = Self::load_profile(&device_name)?;
        
        Ok(Self { profile })
    }
    
    /// Create device from a specific profile file
    pub fn from_profile_file(path: &Path) -> Result<Self, DeviceError> {
        let contents = fs::read_to_string(path)?;
        let profile: DeviceProfile = toml::from_str(&contents)
            .map_err(|e| DeviceError::InitializationFailed(e.to_string()))?;
        
        Ok(Self { profile })
    }
    
    /// Get device profile
    pub fn profile(&self) -> &DeviceProfile {
        &self.profile
    }
    
    /// Read device identifier from device tree or system files
    fn read_device_identifier() -> Result<String, DeviceError> {
        // Try common locations for device identification
        let paths = [
            "/sys/firmware/devicetree/base/model",
            "/proc/device-tree/model",
            "/etc/device-model",
        ];
        
        for path in &paths {
            if let Ok(contents) = fs::read_to_string(path) {
                return Ok(contents.trim().to_string());
            }
        }
        
        Err(DeviceError::DetectionFailed)
    }
    
    /// Load device profile based on identifier
    fn load_profile(identifier: &str) -> Result<DeviceProfile, DeviceError> {
        // For now, return a default RG353M profile
        // In production, this would load from /etc/rexos/devices/
        
        if identifier.contains("rg353") || identifier.contains("RG353") {
            Ok(DeviceProfile {
                name: "Anbernic RG353M".to_string(),
                chipset: "RK3566".to_string(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".to_string(),
                },
                buttons: vec![
                    "up".to_string(),
                    "down".to_string(),
                    "left".to_string(),
                    "right".to_string(),
                    "a".to_string(),
                    "b".to_string(),
                    "x".to_string(),
                    "y".to_string(),
                    "l1".to_string(),
                    "l2".to_string(),
                    "r1".to_string(),
                    "r2".to_string(),
                    "start".to_string(),
                    "select".to_string(),
                ],
                analog_sticks: 2,
                battery_capacity: 3500,
            })
        } else {
            Err(DeviceError::UnsupportedDevice(identifier.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_profile_creation() {
        let profile = DeviceProfile {
            name: "Test Device".to_string(),
            chipset: "Test Chip".to_string(),
            display: DisplaySpec {
                width: 640,
                height: 480,
                format: "RGB565".to_string(),
            },
            buttons: vec!["a".to_string(), "b".to_string()],
            analog_sticks: 2,
            battery_capacity: 3500,
        };
        
        assert_eq!(profile.name, "Test Device");
        assert_eq!(profile.display.width, 640);
    }
}
