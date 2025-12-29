//! WiFi management using wpa_supplicant

use crate::NetworkError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// WiFi network information
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    /// Network SSID
    pub ssid: String,
    /// BSSID (MAC address)
    pub bssid: String,
    /// Signal strength (0-100)
    pub signal: i32,
    /// Frequency in MHz
    pub frequency: u32,
    /// Security type
    pub security: WifiSecurity,
    /// Whether this is a saved network
    pub saved: bool,
    /// Whether currently connected to this network
    pub connected: bool,
}

/// WiFi security types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    WPA2Enterprise,
}

impl WifiSecurity {
    /// Parse from wpa_supplicant flags
    pub fn from_flags(flags: &str) -> Self {
        if flags.contains("WPA3") {
            WifiSecurity::WPA3
        } else if flags.contains("WPA2-EAP") {
            WifiSecurity::WPA2Enterprise
        } else if flags.contains("WPA2") {
            WifiSecurity::WPA2
        } else if flags.contains("WPA") {
            WifiSecurity::WPA
        } else if flags.contains("WEP") {
            WifiSecurity::WEP
        } else {
            WifiSecurity::Open
        }
    }

    /// Get display name
    pub fn as_str(&self) -> &'static str {
        match self {
            WifiSecurity::Open => "Open",
            WifiSecurity::WEP => "WEP",
            WifiSecurity::WPA => "WPA",
            WifiSecurity::WPA2 => "WPA2",
            WifiSecurity::WPA3 => "WPA3",
            WifiSecurity::WPA2Enterprise => "WPA2-Enterprise",
        }
    }
}

/// WiFi connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Scanning,
    Connecting,
    Connected,
    Failed,
}

/// WiFi status information
#[derive(Debug, Clone)]
pub struct WifiStatus {
    pub state: ConnectionState,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub ip_address: Option<String>,
    pub signal: Option<i32>,
    pub frequency: Option<u32>,
}

/// Manages WiFi connections
pub struct WifiManager {
    interface: String,
    /// Path to wpa_supplicant control socket
    wpa_socket: PathBuf,
    /// Path to wpa_supplicant configuration file
    wpa_config: PathBuf,
    available: bool,
}

impl WifiManager {
    /// Create a new WiFi manager
    pub fn new(
        interface: String,
        wpa_socket: PathBuf,
        wpa_config: PathBuf,
    ) -> Result<Self, NetworkError> {
        let available = Self::check_available(&interface);

        Ok(Self {
            interface,
            wpa_socket,
            wpa_config,
            available,
        })
    }

    /// Check if WiFi interface is available
    fn check_available(interface: &str) -> bool {
        let path = format!("/sys/class/net/{}", interface);
        std::path::Path::new(&path).exists()
    }

    /// Check if WiFi is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Enable WiFi interface
    pub fn enable(&self) -> Result<(), NetworkError> {
        self.run_command("ip", &["link", "set", &self.interface, "up"])?;
        tracing::info!("WiFi interface {} enabled", self.interface);
        Ok(())
    }

    /// Disable WiFi interface
    pub fn disable(&self) -> Result<(), NetworkError> {
        self.run_command("ip", &["link", "set", &self.interface, "down"])?;
        tracing::info!("WiFi interface {} disabled", self.interface);
        Ok(())
    }

    /// Scan for available networks
    pub fn scan(&self) -> Result<Vec<WifiNetwork>, NetworkError> {
        if !self.available {
            return Err(NetworkError::WifiNotAvailable);
        }

        // Trigger scan
        self.wpa_cli(&["scan"])?;

        // Wait a bit for scan to complete
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Get results
        let output = self.wpa_cli(&["scan_results"])?;
        let networks = self.parse_scan_results(&output)?;

        tracing::debug!("Found {} networks", networks.len());
        Ok(networks)
    }

    /// Parse scan results
    fn parse_scan_results(&self, output: &str) -> Result<Vec<WifiNetwork>, NetworkError> {
        let mut networks = Vec::new();
        let current_ssid = self.get_current_ssid();

        // Skip header line
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 5 {
                let bssid = parts[0].to_string();
                let frequency: u32 = parts[1].parse().unwrap_or(0);
                let signal: i32 = parts[2].parse().unwrap_or(-100);
                let flags = parts[3];
                let ssid = parts[4..].join("\t");

                if ssid.is_empty() {
                    continue; // Skip hidden networks without SSID
                }

                // Convert dBm to percentage (rough approximation)
                let signal_percent = ((signal + 100) * 2).clamp(0, 100);

                let network = WifiNetwork {
                    ssid: ssid.clone(),
                    bssid,
                    signal: signal_percent,
                    frequency,
                    security: WifiSecurity::from_flags(flags),
                    saved: self.is_network_saved(&ssid),
                    connected: current_ssid.as_ref() == Some(&ssid),
                };

                networks.push(network);
            }
        }

        // Sort by signal strength
        networks.sort_by(|a, b| b.signal.cmp(&a.signal));

        // Remove duplicates (same SSID, keep strongest signal)
        let mut seen = std::collections::HashSet::new();
        networks.retain(|n| seen.insert(n.ssid.clone()));

        Ok(networks)
    }

    /// Connect to a network
    pub fn connect(&self, ssid: &str, password: Option<&str>) -> Result<(), NetworkError> {
        if !self.available {
            return Err(NetworkError::WifiNotAvailable);
        }

        tracing::info!("Connecting to network: {}", ssid);

        // Check if network already configured
        if let Some(network_id) = self.find_network_id(ssid)? {
            // Use existing configuration
            self.wpa_cli(&["select_network", &network_id])?;
        } else {
            // Add new network
            let output = self.wpa_cli(&["add_network"])?;
            let network_id = output.trim();

            // Set SSID
            self.wpa_cli(&["set_network", network_id, "ssid", &format!("\"{}\"", ssid)])?;

            // Set password if provided
            if let Some(pass) = password {
                self.wpa_cli(&["set_network", network_id, "psk", &format!("\"{}\"", pass)])?;
            } else {
                self.wpa_cli(&["set_network", network_id, "key_mgmt", "NONE"])?;
            }

            // Enable and select network
            self.wpa_cli(&["enable_network", network_id])?;
            self.wpa_cli(&["select_network", network_id])?;
        }

        // Wait for connection
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_secs(1));

            if let Ok(status) = self.status() {
                match status.state {
                    ConnectionState::Connected => {
                        tracing::info!("Connected to {}", ssid);
                        self.wpa_cli(&["save_config"])?;
                        return Ok(());
                    }
                    ConnectionState::Failed => {
                        return Err(NetworkError::ConnectionFailed("Connection failed".into()));
                    }
                    _ => continue,
                }
            }
        }

        Err(NetworkError::Timeout)
    }

    /// Disconnect from current network
    pub fn disconnect(&self) -> Result<(), NetworkError> {
        self.wpa_cli(&["disconnect"])?;
        tracing::info!("Disconnected from WiFi");
        Ok(())
    }

    /// Get current status
    pub fn status(&self) -> Result<WifiStatus, NetworkError> {
        let output = self.wpa_cli(&["status"])?;
        let mut status = WifiStatus {
            state: ConnectionState::Disconnected,
            ssid: None,
            bssid: None,
            ip_address: None,
            signal: None,
            frequency: None,
        };

        for line in output.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "wpa_state" => {
                        status.state = match value {
                            "COMPLETED" => ConnectionState::Connected,
                            "SCANNING" => ConnectionState::Scanning,
                            "ASSOCIATING" | "ASSOCIATED" | "4WAY_HANDSHAKE" | "GROUP_HANDSHAKE" => {
                                ConnectionState::Connecting
                            }
                            "DISCONNECTED" | "INACTIVE" => ConnectionState::Disconnected,
                            _ => ConnectionState::Disconnected,
                        };
                    }
                    "ssid" => status.ssid = Some(value.to_string()),
                    "bssid" => status.bssid = Some(value.to_string()),
                    "ip_address" => status.ip_address = Some(value.to_string()),
                    "freq" => status.frequency = value.parse().ok(),
                    _ => {}
                }
            }
        }

        Ok(status)
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.status()
            .is_ok_and(|s| s.state == ConnectionState::Connected)
    }

    /// Get current IP address
    pub fn get_ip_address(&self) -> Option<String> {
        self.status().ok().and_then(|s| s.ip_address)
    }

    /// Get current SSID
    pub fn get_current_ssid(&self) -> Option<String> {
        self.status().ok().and_then(|s| s.ssid)
    }

    /// Get saved networks
    pub fn list_saved_networks(&self) -> Result<Vec<String>, NetworkError> {
        let output = self.wpa_cli(&["list_networks"])?;
        let mut networks = Vec::new();

        // Skip header line
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                networks.push(parts[1].to_string());
            }
        }

        Ok(networks)
    }

    /// Remove a saved network
    pub fn forget_network(&self, ssid: &str) -> Result<(), NetworkError> {
        if let Some(network_id) = self.find_network_id(ssid)? {
            self.wpa_cli(&["remove_network", &network_id])?;
            self.wpa_cli(&["save_config"])?;
            tracing::info!("Forgot network: {}", ssid);
        }
        Ok(())
    }

    /// Check if a network is saved
    fn is_network_saved(&self, ssid: &str) -> bool {
        self.find_network_id(ssid).ok().flatten().is_some()
    }

    /// Find network ID by SSID
    fn find_network_id(&self, ssid: &str) -> Result<Option<String>, NetworkError> {
        let output = self.wpa_cli(&["list_networks"])?;

        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 && parts[1] == ssid {
                return Ok(Some(parts[0].to_string()));
            }
        }

        Ok(None)
    }

    /// Run wpa_cli command
    fn wpa_cli(&self, args: &[&str]) -> Result<String, NetworkError> {
        let mut cmd = Command::new("wpa_cli");
        cmd.arg("-i").arg(&self.interface);

        // Use custom socket path if it exists
        if self.wpa_socket.exists() {
            cmd.arg("-p")
                .arg(self.wpa_socket.to_string_lossy().as_ref());
        }

        cmd.args(args);

        let output = cmd.output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(NetworkError::CommandFailed(stderr.to_string()))
        }
    }

    /// Get the path to the wpa_supplicant configuration file
    pub fn config_path(&self) -> &Path {
        &self.wpa_config
    }

    /// Get the path to the wpa_supplicant control socket
    pub fn socket_path(&self) -> &Path {
        &self.wpa_socket
    }

    /// Run generic command
    fn run_command(&self, cmd: &str, args: &[&str]) -> Result<String, NetworkError> {
        let output = Command::new(cmd).args(args).output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(NetworkError::CommandFailed(stderr.to_string()))
        }
    }

    /// Set WiFi power save mode
    pub fn set_power_save(&self, enabled: bool) -> Result<(), NetworkError> {
        let value = if enabled { "on" } else { "off" };
        self.run_command("iw", &["dev", &self.interface, "set", "power_save", value])?;
        tracing::debug!("WiFi power save: {}", value);
        Ok(())
    }

    /// Get signal strength of current connection
    pub fn get_signal_strength(&self) -> Option<i32> {
        let output = self
            .run_command("iw", &["dev", &self.interface, "link"])
            .ok()?;

        for line in output.lines() {
            if line.contains("signal:")
                && let Some(signal_str) = line.split_whitespace().nth(1)
                && let Ok(signal) = signal_str.parse::<i32>()
            {
                // Convert dBm to percentage
                return Some(((signal + 100) * 2).clamp(0, 100));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_from_flags() {
        assert_eq!(
            WifiSecurity::from_flags("[WPA2-PSK-CCMP]"),
            WifiSecurity::WPA2
        );
        assert_eq!(WifiSecurity::from_flags("[WPA-PSK]"), WifiSecurity::WPA);
        assert_eq!(WifiSecurity::from_flags("[ESS]"), WifiSecurity::Open);
    }

    #[test]
    fn test_security_display() {
        assert_eq!(WifiSecurity::WPA2.as_str(), "WPA2");
        assert_eq!(WifiSecurity::Open.as_str(), "Open");
    }
}
