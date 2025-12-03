//! Power management
//!
//! Handles battery monitoring, charging detection, and CPU governor control via sysfs.
//! Based on ArkOS power management patterns including low battery warning.

use crate::DeviceError;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Battery information
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percentage: u8,
    pub voltage: f32,
    pub current: f32,
    pub is_charging: bool,
    pub status: BatteryStatus,
    pub health: BatteryHealth,
    pub temperature: f32,
}

/// Battery charging status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    NotCharging,
    Unknown,
}

/// Battery health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryHealth {
    Good,
    Overheat,
    Dead,
    Cold,
    Unknown,
}

/// CPU governor (performance profile)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuGovernor {
    Performance,
    Powersave,
    Ondemand,
    Schedutil,
    Conservative,
}

impl CpuGovernor {
    /// Get sysfs name
    pub fn as_str(&self) -> &'static str {
        match self {
            CpuGovernor::Performance => "performance",
            CpuGovernor::Powersave => "powersave",
            CpuGovernor::Ondemand => "ondemand",
            CpuGovernor::Schedutil => "schedutil",
            CpuGovernor::Conservative => "conservative",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "performance" => Some(CpuGovernor::Performance),
            "powersave" => Some(CpuGovernor::Powersave),
            "ondemand" => Some(CpuGovernor::Ondemand),
            "schedutil" => Some(CpuGovernor::Schedutil),
            "conservative" => Some(CpuGovernor::Conservative),
            _ => None,
        }
    }
}

/// Power manager configuration
#[derive(Debug, Clone)]
pub struct PowerConfig {
    pub battery_path: PathBuf,
    pub charger_path: PathBuf,
    pub low_battery_threshold: u8,
    pub critical_battery_threshold: u8,
    pub suspend_timeout: u32,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            battery_path: PathBuf::from("/sys/class/power_supply/battery"),
            charger_path: PathBuf::from("/sys/class/power_supply/usb"),
            low_battery_threshold: 20,
            critical_battery_threshold: 5,
            suspend_timeout: 300,
        }
    }
}

/// Power manager
pub struct PowerManager {
    config: PowerConfig,
    battery_path: PathBuf,
    charger_path: PathBuf,
}

impl PowerManager {
    /// Create a new power manager
    pub fn new() -> Result<Self, DeviceError> {
        Self::with_config(PowerConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: PowerConfig) -> Result<Self, DeviceError> {
        let mut manager = Self {
            battery_path: config.battery_path.clone(),
            charger_path: config.charger_path.clone(),
            config,
        };

        // Auto-detect battery and charger paths
        manager.detect_power_supplies()?;

        Ok(manager)
    }

    /// Detect power supply sysfs paths
    fn detect_power_supplies(&mut self) -> Result<(), DeviceError> {
        let power_supply_dir = Path::new("/sys/class/power_supply");
        if !power_supply_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(power_supply_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();

            // Read type to determine if it's battery or charger
            let type_path = path.join("type");
            if let Ok(psu_type) = fs::read_to_string(&type_path) {
                let psu_type = psu_type.trim().to_lowercase();

                if psu_type == "battery" {
                    self.battery_path = path.clone();
                    tracing::info!("Found battery at {}", path.display());
                } else if psu_type == "usb" || psu_type == "mains" || name.contains("charger") {
                    self.charger_path = path.clone();
                    tracing::info!("Found charger at {}", path.display());
                }
            }
        }

        Ok(())
    }

    /// Get battery information
    pub fn get_battery_info(&self) -> Result<BatteryInfo, DeviceError> {
        // Read capacity (percentage)
        let percentage = self
            .read_sysfs_int(&self.battery_path.join("capacity"))
            .unwrap_or(50) as u8;

        // Read voltage (in microvolts, convert to volts)
        let voltage_uv = self
            .read_sysfs_int(&self.battery_path.join("voltage_now"))
            .unwrap_or(3700000);
        let voltage = voltage_uv as f32 / 1_000_000.0;

        // Read current (in microamps, convert to amps)
        let current_ua = self
            .read_sysfs_int(&self.battery_path.join("current_now"))
            .unwrap_or(0);
        let current = current_ua as f32 / 1_000_000.0;

        // Read status
        let status_str = fs::read_to_string(self.battery_path.join("status"))
            .unwrap_or_else(|_| "Unknown".to_string());
        let status = match status_str.trim() {
            "Charging" => BatteryStatus::Charging,
            "Discharging" => BatteryStatus::Discharging,
            "Full" => BatteryStatus::Full,
            "Not charging" => BatteryStatus::NotCharging,
            _ => BatteryStatus::Unknown,
        };

        // Read health
        let health_str = fs::read_to_string(self.battery_path.join("health"))
            .unwrap_or_else(|_| "Unknown".to_string());
        let health = match health_str.trim() {
            "Good" => BatteryHealth::Good,
            "Overheat" => BatteryHealth::Overheat,
            "Dead" => BatteryHealth::Dead,
            "Cold" => BatteryHealth::Cold,
            _ => BatteryHealth::Unknown,
        };

        // Read temperature (in tenths of degree celsius)
        let temp_raw = self
            .read_sysfs_int(&self.battery_path.join("temp"))
            .unwrap_or(250);
        let temperature = temp_raw as f32 / 10.0;

        // Determine charging state
        let is_charging = status == BatteryStatus::Charging || self.is_charger_connected();

        Ok(BatteryInfo {
            percentage,
            voltage,
            current,
            is_charging,
            status,
            health,
            temperature,
        })
    }

    /// Read integer from sysfs file
    fn read_sysfs_int(&self, path: &Path) -> Option<i64> {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    /// Check if charger is connected
    pub fn is_charger_connected(&self) -> bool {
        // Check charger online status
        let online_path = self.charger_path.join("online");
        if let Ok(contents) = fs::read_to_string(&online_path) {
            return contents.trim() == "1";
        }

        // Fallback: check battery status
        if let Ok(status) = fs::read_to_string(self.battery_path.join("status")) {
            return status.trim() == "Charging" || status.trim() == "Full";
        }

        false
    }

    /// Check if battery is low (below threshold)
    pub fn is_battery_low(&self) -> bool {
        if let Ok(info) = self.get_battery_info() {
            return info.percentage <= self.config.low_battery_threshold && !info.is_charging;
        }
        false
    }

    /// Check if battery is critical
    pub fn is_battery_critical(&self) -> bool {
        if let Ok(info) = self.get_battery_info() {
            return info.percentage <= self.config.critical_battery_threshold && !info.is_charging;
        }
        false
    }

    /// Get current CPU governor
    pub fn get_governor(&self) -> Option<CpuGovernor> {
        let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
        fs::read_to_string(path)
            .ok()
            .and_then(|s| CpuGovernor::parse(&s))
    }

    /// Set CPU governor for all CPUs
    pub fn set_governor(&self, governor: CpuGovernor) -> Result<(), DeviceError> {
        let cpu_dir = Path::new("/sys/devices/system/cpu");

        for entry in fs::read_dir(cpu_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with("cpu") && name.chars().nth(3).is_some_and(|c| c.is_ascii_digit()) {
                let governor_path = entry.path().join("cpufreq/scaling_governor");
                if governor_path.exists() {
                    fs::write(&governor_path, governor.as_str()).map_err(|e| {
                        DeviceError::InitializationFailed(format!("Failed to set governor: {}", e))
                    })?;
                }
            }
        }

        tracing::info!("CPU governor set to {}", governor.as_str());
        Ok(())
    }

    /// Get available governors
    pub fn available_governors(&self) -> Vec<CpuGovernor> {
        let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
        if let Ok(contents) = fs::read_to_string(path) {
            return contents
                .split_whitespace()
                .filter_map(CpuGovernor::parse)
                .collect();
        }
        vec![]
    }

    /// Get current CPU frequency (Hz)
    pub fn get_cpu_frequency(&self) -> Option<u64> {
        let path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq";
        fs::read_to_string(path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|khz| khz * 1000) // Convert to Hz
    }

    /// Suspend the system
    pub fn suspend(&self) -> Result<(), DeviceError> {
        tracing::info!("Suspending system...");

        // Write to /sys/power/state
        let result = fs::write("/sys/power/state", "mem");

        if result.is_err() {
            // Fallback to systemctl
            let _ = Command::new("systemctl").arg("suspend").output();
        }

        Ok(())
    }

    /// Shutdown the system
    pub fn shutdown(&self) -> Result<(), DeviceError> {
        tracing::info!("Shutting down system...");

        let _ = Command::new("shutdown").args(["-h", "now"]).output();

        Ok(())
    }

    /// Reboot the system
    pub fn reboot(&self) -> Result<(), DeviceError> {
        tracing::info!("Rebooting system...");

        let _ = Command::new("reboot").output();

        Ok(())
    }

    /// Get configuration
    pub fn config(&self) -> &PowerConfig {
        &self.config
    }

    /// Set low battery threshold
    pub fn set_low_battery_threshold(&mut self, threshold: u8) {
        self.config.low_battery_threshold = threshold.min(100);
    }
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            config: PowerConfig::default(),
            battery_path: PathBuf::from("/sys/class/power_supply/battery"),
            charger_path: PathBuf::from("/sys/class/power_supply/usb"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_config_default() {
        let config = PowerConfig::default();
        assert_eq!(config.low_battery_threshold, 20);
        assert_eq!(config.critical_battery_threshold, 5);
    }

    #[test]
    fn test_cpu_governor_str() {
        assert_eq!(CpuGovernor::Performance.as_str(), "performance");
        assert_eq!(CpuGovernor::Powersave.as_str(), "powersave");
    }

    #[test]
    fn test_cpu_governor_from_str() {
        assert_eq!(
            CpuGovernor::parse("performance"),
            Some(CpuGovernor::Performance)
        );
        assert_eq!(CpuGovernor::parse("invalid"), None);
    }
}
