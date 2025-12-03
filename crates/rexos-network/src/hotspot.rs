//! WiFi hotspot/AP mode for file transfer

use crate::NetworkError;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Hotspot configuration
#[derive(Debug, Clone)]
pub struct HotspotConfig {
    /// Network name (SSID)
    pub ssid: String,
    /// Password (minimum 8 characters)
    pub password: String,
    /// Channel (1-11 for 2.4GHz)
    pub channel: u8,
    /// Hide SSID
    pub hidden: bool,
    /// IP address for AP
    pub ip_address: String,
    /// DHCP range start
    pub dhcp_start: String,
    /// DHCP range end
    pub dhcp_end: String,
}

impl Default for HotspotConfig {
    fn default() -> Self {
        Self {
            ssid: "RexOS".to_string(),
            password: "rexos123".to_string(),
            channel: 6,
            hidden: false,
            ip_address: "192.168.4.1".to_string(),
            dhcp_start: "192.168.4.2".to_string(),
            dhcp_end: "192.168.4.20".to_string(),
        }
    }
}

/// Manages WiFi hotspot mode
pub struct HotspotManager {
    interface: String,
    config: HotspotConfig,
    running: bool,
}

impl HotspotManager {
    /// Create a new hotspot manager
    pub fn new(interface: String) -> Self {
        Self {
            interface,
            config: HotspotConfig::default(),
            running: false,
        }
    }

    /// Configure hotspot settings
    pub fn configure(&mut self, config: HotspotConfig) {
        self.config = config;
    }

    /// Start the hotspot
    pub fn start(&mut self) -> Result<(), NetworkError> {
        if self.running {
            return Ok(());
        }

        tracing::info!("Starting WiFi hotspot: {}", self.config.ssid);

        // Check if hostapd and dnsmasq are available
        if !Self::is_hostapd_available() {
            return Err(NetworkError::CommandFailed("hostapd not found".into()));
        }

        // Stop any existing hostapd/dnsmasq
        self.stop_services();

        // Configure interface
        self.configure_interface()?;

        // Write hostapd config
        self.write_hostapd_config()?;

        // Write dnsmasq config
        self.write_dnsmasq_config()?;

        // Start services
        self.start_hostapd()?;
        self.start_dnsmasq()?;

        self.running = true;
        tracing::info!("Hotspot started on {}", self.config.ip_address);

        Ok(())
    }

    /// Stop the hotspot
    pub fn stop(&mut self) -> Result<(), NetworkError> {
        if !self.running {
            return Ok(());
        }

        tracing::info!("Stopping WiFi hotspot");

        self.stop_services();
        self.running = false;

        // Return interface to managed mode
        let _ = Command::new("ip")
            .args(["addr", "flush", "dev", &self.interface])
            .output();

        Ok(())
    }

    /// Check if hotspot is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get connected clients
    pub fn get_clients(&self) -> Result<Vec<HotspotClient>, NetworkError> {
        if !self.running {
            return Ok(Vec::new());
        }

        // Read DHCP leases
        let leases_path = "/tmp/dnsmasq.leases";
        if !std::path::Path::new(leases_path).exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(leases_path)?;
        let mut clients = Vec::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                clients.push(HotspotClient {
                    mac_address: parts[1].to_string(),
                    ip_address: parts[2].to_string(),
                    hostname: parts[3].to_string(),
                });
            }
        }

        Ok(clients)
    }

    /// Check if hostapd is available
    fn is_hostapd_available() -> bool {
        Command::new("which")
            .arg("hostapd")
            .output()
            .map_or(false, |o| o.status.success())
    }

    /// Configure network interface
    fn configure_interface(&self) -> Result<(), NetworkError> {
        // Set IP address
        let _ = Command::new("ip")
            .args(["addr", "flush", "dev", &self.interface])
            .output();

        let output = Command::new("ip")
            .args([
                "addr", "add",
                &format!("{}/24", self.config.ip_address),
                "dev", &self.interface
            ])
            .output()?;

        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Bring interface up
        Command::new("ip")
            .args(["link", "set", &self.interface, "up"])
            .output()?;

        Ok(())
    }

    /// Write hostapd configuration
    fn write_hostapd_config(&self) -> Result<(), NetworkError> {
        let config = format!(
            r#"interface={}
driver=nl80211
ssid={}
hw_mode=g
channel={}
wmm_enabled=0
macaddr_acl=0
auth_algs=1
ignore_broadcast_ssid={}
wpa=2
wpa_passphrase={}
wpa_key_mgmt=WPA-PSK
wpa_pairwise=TKIP
rsn_pairwise=CCMP
"#,
            self.interface,
            self.config.ssid,
            self.config.channel,
            if self.config.hidden { 1 } else { 0 },
            self.config.password,
        );

        fs::write("/tmp/hostapd.conf", config)?;
        Ok(())
    }

    /// Write dnsmasq configuration
    fn write_dnsmasq_config(&self) -> Result<(), NetworkError> {
        let config = format!(
            r#"interface={}
dhcp-range={},{},12h
dhcp-leasefile=/tmp/dnsmasq.leases
bind-interfaces
server=8.8.8.8
server=8.8.4.4
domain-needed
bogus-priv
"#,
            self.interface,
            self.config.dhcp_start,
            self.config.dhcp_end,
        );

        fs::write("/tmp/dnsmasq.conf", config)?;
        Ok(())
    }

    /// Start hostapd
    fn start_hostapd(&self) -> Result<(), NetworkError> {
        let output = Command::new("hostapd")
            .args(["-B", "-P", "/tmp/hostapd.pid", "/tmp/hostapd.conf"])
            .output()?;

        if !output.status.success() {
            return Err(NetworkError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }

    /// Start dnsmasq
    fn start_dnsmasq(&self) -> Result<(), NetworkError> {
        let output = Command::new("dnsmasq")
            .args(["-C", "/tmp/dnsmasq.conf", "--pid-file=/tmp/dnsmasq.pid"])
            .output()?;

        if !output.status.success() {
            // dnsmasq might already be running
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("already in use") {
                return Err(NetworkError::CommandFailed(stderr.to_string()));
            }
        }

        Ok(())
    }

    /// Stop hostapd and dnsmasq
    fn stop_services(&self) {
        // Kill hostapd
        let _ = Command::new("pkill").arg("-F").arg("/tmp/hostapd.pid").output();
        let _ = Command::new("pkill").arg("hostapd").output();

        // Kill dnsmasq
        let _ = Command::new("pkill").arg("-F").arg("/tmp/dnsmasq.pid").output();
        let _ = Command::new("pkill").arg("-f").arg("dnsmasq.*rexos").output();

        // Clean up files
        let _ = fs::remove_file("/tmp/hostapd.conf");
        let _ = fs::remove_file("/tmp/hostapd.pid");
        let _ = fs::remove_file("/tmp/dnsmasq.conf");
        let _ = fs::remove_file("/tmp/dnsmasq.pid");
    }

    /// Get hotspot status
    pub fn status(&self) -> HotspotStatus {
        HotspotStatus {
            running: self.running,
            ssid: self.config.ssid.clone(),
            ip_address: self.config.ip_address.clone(),
            clients: self.get_clients().unwrap_or_default(),
        }
    }
}

/// Connected client information
#[derive(Debug, Clone)]
pub struct HotspotClient {
    pub mac_address: String,
    pub ip_address: String,
    pub hostname: String,
}

/// Hotspot status
#[derive(Debug, Clone)]
pub struct HotspotStatus {
    pub running: bool,
    pub ssid: String,
    pub ip_address: String,
    pub clients: Vec<HotspotClient>,
}

impl Drop for HotspotManager {
    fn drop(&mut self) {
        if self.running {
            let _ = self.stop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotspot_config_default() {
        let config = HotspotConfig::default();
        assert_eq!(config.ssid, "RexOS");
        assert_eq!(config.channel, 6);
        assert!(!config.hidden);
    }

    #[test]
    fn test_hotspot_manager_creation() {
        let manager = HotspotManager::new("wlan0".to_string());
        assert!(!manager.is_running());
    }
}
