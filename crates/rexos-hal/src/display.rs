//! Display management
//!
//! Handles display brightness, rotation, and HDMI output via sysfs.

use crate::DeviceError;
use std::fs;
use std::path::{Path, PathBuf};

/// Display configuration
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub brightness: u8,
    pub rotation: Rotation,
    pub backlight_path: PathBuf,
    pub max_brightness: u32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            brightness: 180,
            rotation: Rotation::Normal,
            backlight_path: PathBuf::from("/sys/class/backlight/backlight"),
            max_brightness: 255,
        }
    }
}

/// Display rotation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Rotation {
    /// Get rotation angle in degrees
    pub fn degrees(&self) -> u32 {
        match self {
            Rotation::Normal => 0,
            Rotation::Rotate90 => 90,
            Rotation::Rotate180 => 180,
            Rotation::Rotate270 => 270,
        }
    }

    /// Get fbcon rotate value
    pub fn fbcon_value(&self) -> u8 {
        match self {
            Rotation::Normal => 0,
            Rotation::Rotate90 => 1,
            Rotation::Rotate180 => 2,
            Rotation::Rotate270 => 3,
        }
    }
}

/// Backlight controller information
#[derive(Debug, Clone)]
pub struct BacklightInfo {
    pub name: String,
    pub path: PathBuf,
    pub max_brightness: u32,
    pub current_brightness: u32,
}

/// Display manager
pub struct Display {
    config: DisplayConfig,
    backlight_path: PathBuf,
    max_brightness: u32,
}

impl Display {
    /// Create a new display manager
    pub fn new(config: DisplayConfig) -> Result<Self, DeviceError> {
        let backlight_path = config.backlight_path.clone();
        let max_brightness = config.max_brightness;

        let mut display = Self {
            config,
            backlight_path,
            max_brightness,
        };

        // Try to detect actual max brightness from sysfs
        display.detect_backlight()?;

        // Apply initial brightness
        let brightness = display.config.brightness;
        display.set_brightness(brightness)?;

        Ok(display)
    }

    /// Detect backlight device from sysfs
    #[allow(clippy::collapsible_if)] // Avoid if-let chains for MSRV 1.85 compatibility
    fn detect_backlight(&mut self) -> Result<(), DeviceError> {
        // If specific path doesn't exist, scan for backlight devices
        if !self.backlight_path.exists() {
            let backlight_dir = Path::new("/sys/class/backlight");
            if backlight_dir.exists() {
                for entry in fs::read_dir(backlight_dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    // Check if it has brightness control
                    if path.join("brightness").exists() {
                        self.backlight_path = path;
                        tracing::info!("Found backlight at {}", self.backlight_path.display());
                        break;
                    }
                }
            }
        }

        // Read max brightness
        let max_brightness_path = self.backlight_path.join("max_brightness");
        if max_brightness_path.exists() {
            if let Ok(contents) = fs::read_to_string(&max_brightness_path) {
                if let Ok(max) = contents.trim().parse::<u32>() {
                    self.max_brightness = max;
                    tracing::debug!("Max brightness: {}", max);
                }
            }
        }

        Ok(())
    }

    /// Set display brightness (0-255, scaled to device max)
    pub fn set_brightness(&mut self, level: u8) -> Result<(), DeviceError> {
        self.config.brightness = level;

        // Scale to device's actual max brightness
        let scaled = (level as u32 * self.max_brightness) / 255;

        let brightness_path = self.backlight_path.join("brightness");
        if brightness_path.exists() {
            fs::write(&brightness_path, scaled.to_string()).map_err(|e| {
                DeviceError::InitializationFailed(format!("Failed to set brightness: {}", e))
            })?;

            tracing::debug!("Brightness set to {} (raw: {})", level, scaled);
        } else {
            tracing::warn!("Backlight sysfs not available");
        }

        Ok(())
    }

    /// Get current brightness (0-255)
    pub fn get_brightness(&self) -> u8 {
        self.config.brightness
    }

    /// Read actual brightness from hardware
    pub fn read_brightness(&self) -> Result<u8, DeviceError> {
        let brightness_path = self.backlight_path.join("brightness");
        if brightness_path.exists() {
            let contents = fs::read_to_string(&brightness_path)?;
            let raw: u32 = contents.trim().parse().unwrap_or(0);

            // Scale back to 0-255
            let scaled = (raw * 255) / self.max_brightness.max(1);
            Ok(scaled.min(255) as u8)
        } else {
            Ok(self.config.brightness)
        }
    }

    /// Increase brightness by step
    pub fn brightness_up(&mut self, step: u8) -> Result<(), DeviceError> {
        let new_level = self.config.brightness.saturating_add(step);
        self.set_brightness(new_level)
    }

    /// Decrease brightness by step
    pub fn brightness_down(&mut self, step: u8) -> Result<(), DeviceError> {
        let new_level = self.config.brightness.saturating_sub(step);
        self.set_brightness(new_level)
    }

    /// Get display resolution
    pub fn resolution(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Get display rotation
    pub fn rotation(&self) -> Rotation {
        self.config.rotation
    }

    /// Set display rotation (requires framebuffer support)
    pub fn set_rotation(&mut self, rotation: Rotation) -> Result<(), DeviceError> {
        self.config.rotation = rotation;

        // Try to set via fbcon (framebuffer console)
        let fbcon_rotate = Path::new("/sys/class/graphics/fb0/rotate");
        if fbcon_rotate.exists() {
            fs::write(fbcon_rotate, rotation.fbcon_value().to_string()).map_err(|e| {
                DeviceError::InitializationFailed(format!("Failed to set rotation: {}", e))
            })?;

            tracing::info!("Display rotation set to {} degrees", rotation.degrees());
        }

        Ok(())
    }

    /// Turn display on
    pub fn power_on(&self) -> Result<(), DeviceError> {
        let bl_power = self.backlight_path.join("bl_power");
        if bl_power.exists() {
            fs::write(&bl_power, "0")?; // 0 = FB_BLANK_UNBLANK
            tracing::info!("Display powered on");
        }
        Ok(())
    }

    /// Turn display off (for power saving)
    pub fn power_off(&self) -> Result<(), DeviceError> {
        let bl_power = self.backlight_path.join("bl_power");
        if bl_power.exists() {
            fs::write(&bl_power, "4")?; // 4 = FB_BLANK_POWERDOWN
            tracing::info!("Display powered off");
        }
        Ok(())
    }

    /// Get display configuration
    pub fn config(&self) -> &DisplayConfig {
        &self.config
    }

    /// Get list of all backlight devices
    pub fn list_backlights() -> Result<Vec<BacklightInfo>, DeviceError> {
        let mut backlights = Vec::new();

        let backlight_dir = Path::new("/sys/class/backlight");
        if !backlight_dir.exists() {
            return Ok(backlights);
        }

        for entry in fs::read_dir(backlight_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            let max_brightness = fs::read_to_string(path.join("max_brightness"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(255);

            let current_brightness = fs::read_to_string(path.join("brightness"))
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);

            backlights.push(BacklightInfo {
                name,
                path,
                max_brightness,
                current_brightness,
            });
        }

        Ok(backlights)
    }

    /// Check if HDMI is connected
    #[allow(clippy::collapsible_if)] // Avoid if-let chains for MSRV 1.85 compatibility
    pub fn is_hdmi_connected() -> bool {
        // Check common HDMI sysfs paths
        let paths = [
            "/sys/class/drm/card0-HDMI-A-1/status",
            "/sys/class/drm/card1-HDMI-A-1/status",
            "/sys/devices/platform/display-subsystem/drm/card0/card0-HDMI-A-1/status",
        ];

        for path in &paths {
            if let Ok(contents) = fs::read_to_string(path) {
                if contents.trim() == "connected" {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_config_default() {
        let config = DisplayConfig::default();
        assert_eq!(config.width, 640);
        assert_eq!(config.height, 480);
        assert_eq!(config.brightness, 180);
    }

    #[test]
    fn test_rotation_degrees() {
        assert_eq!(Rotation::Normal.degrees(), 0);
        assert_eq!(Rotation::Rotate90.degrees(), 90);
        assert_eq!(Rotation::Rotate180.degrees(), 180);
        assert_eq!(Rotation::Rotate270.degrees(), 270);
    }

    #[test]
    fn test_rotation_fbcon() {
        assert_eq!(Rotation::Normal.fbcon_value(), 0);
        assert_eq!(Rotation::Rotate90.fbcon_value(), 1);
    }
}
