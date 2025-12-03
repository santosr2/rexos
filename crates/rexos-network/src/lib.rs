//! Network management for RexOS
//!
//! Provides WiFi and Bluetooth management using wpa_supplicant and bluetoothctl.
//! Compatible with typical embedded Linux networking setups.
//!
//! # Features
//!
//! - WiFi network scanning and connection
//! - WPA/WPA2/WPA3 support
//! - Hidden network support
//! - Saved network management
//! - Bluetooth device discovery and pairing
//! - Bluetooth audio (A2DP) for wireless controllers

mod bluetooth;
mod hotspot;
mod wifi;

pub use bluetooth::{BluetoothDevice, BluetoothDeviceType, BluetoothManager, PairingState};
pub use hotspot::{HotspotConfig, HotspotManager};
pub use wifi::{ConnectionState, WifiManager, WifiNetwork, WifiSecurity, WifiStatus};

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("WiFi not available")]
    WifiNotAvailable,

    #[error("Bluetooth not available")]
    BluetoothNotAvailable,

    #[error("Network not found: {0}")]
    NetworkNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Pairing failed: {0}")]
    PairingFailed(String),

    #[error("Timeout")]
    Timeout,

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Path to wpa_supplicant socket
    pub wpa_socket: PathBuf,

    /// Path to wpa_supplicant config
    pub wpa_config: PathBuf,

    /// WiFi interface name
    pub wifi_interface: String,

    /// Bluetooth interface (hci0, etc.)
    pub bt_interface: String,

    /// Enable WiFi power saving
    pub wifi_power_save: bool,

    /// Auto-reconnect to known networks
    pub auto_reconnect: bool,

    /// Scan interval in seconds
    pub scan_interval: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            wpa_socket: PathBuf::from("/var/run/wpa_supplicant"),
            wpa_config: PathBuf::from("/etc/wpa_supplicant/wpa_supplicant.conf"),
            wifi_interface: "wlan0".to_string(),
            bt_interface: "hci0".to_string(),
            wifi_power_save: false, // Keep responsive for gaming
            auto_reconnect: true,
            scan_interval: 30,
        }
    }
}

/// Main network manager
pub struct NetworkManager {
    wifi: WifiManager,
    bluetooth: BluetoothManager,
    hotspot: HotspotManager,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        let wifi = WifiManager::new(
            config.wifi_interface.clone(),
            config.wpa_socket.clone(),
            config.wpa_config.clone(),
        )?;

        let bluetooth = BluetoothManager::new(config.bt_interface.clone())?;

        let hotspot = HotspotManager::new(config.wifi_interface.clone());

        Ok(Self {
            wifi,
            bluetooth,
            hotspot,
        })
    }

    /// Get WiFi manager
    pub fn wifi(&mut self) -> &mut WifiManager {
        &mut self.wifi
    }

    /// Get Bluetooth manager
    pub fn bluetooth(&mut self) -> &mut BluetoothManager {
        &mut self.bluetooth
    }

    /// Get hotspot manager
    pub fn hotspot(&mut self) -> &mut HotspotManager {
        &mut self.hotspot
    }

    /// Check if WiFi is available
    pub fn wifi_available(&self) -> bool {
        self.wifi.is_available()
    }

    /// Check if Bluetooth is available
    pub fn bluetooth_available(&self) -> bool {
        self.bluetooth.is_available()
    }

    /// Check if connected to any network
    pub fn is_connected(&self) -> bool {
        self.wifi.is_connected()
    }

    /// Get current IP address
    pub fn get_ip_address(&self) -> Option<String> {
        self.wifi.get_ip_address()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.wifi_interface, "wlan0");
        assert!(!config.wifi_power_save);
    }
}
