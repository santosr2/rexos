//! Bluetooth management using bluetoothctl

use crate::NetworkError;
use std::process::Command;

/// Bluetooth device information
#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    /// MAC address
    pub address: String,
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: BluetoothDeviceType,
    /// Whether device is paired
    pub paired: bool,
    /// Whether device is connected
    pub connected: bool,
    /// Whether device is trusted
    pub trusted: bool,
    /// Signal strength (RSSI)
    pub rssi: Option<i32>,
}

/// Bluetooth device types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothDeviceType {
    /// Generic/unknown device
    Unknown,
    /// Game controller
    Controller,
    /// Keyboard
    Keyboard,
    /// Mouse/pointer
    Mouse,
    /// Audio device (headphones/speakers)
    Audio,
    /// Phone/smartphone
    Phone,
    /// Computer
    Computer,
}

impl BluetoothDeviceType {
    /// Parse from device class or appearance
    pub fn from_class(class: u32) -> Self {
        // Major device class is bits 8-12
        let major = (class >> 8) & 0x1F;

        match major {
            0x01 => BluetoothDeviceType::Computer,
            0x02 => BluetoothDeviceType::Phone,
            0x04 => BluetoothDeviceType::Audio,
            0x05 => {
                // Peripheral - check minor class
                let minor = (class >> 2) & 0x3F;
                match minor {
                    0x01 => BluetoothDeviceType::Controller, // Joystick
                    0x02 => BluetoothDeviceType::Controller, // Gamepad
                    0x10 => BluetoothDeviceType::Keyboard,
                    0x20 => BluetoothDeviceType::Mouse,
                    0x30 => BluetoothDeviceType::Keyboard, // Combo keyboard/pointing
                    _ => BluetoothDeviceType::Unknown,
                }
            }
            _ => BluetoothDeviceType::Unknown,
        }
    }

    /// Get icon name
    pub fn icon(&self) -> &'static str {
        match self {
            BluetoothDeviceType::Controller => "input-gaming",
            BluetoothDeviceType::Keyboard => "input-keyboard",
            BluetoothDeviceType::Mouse => "input-mouse",
            BluetoothDeviceType::Audio => "audio-headphones",
            BluetoothDeviceType::Phone => "phone",
            BluetoothDeviceType::Computer => "computer",
            BluetoothDeviceType::Unknown => "bluetooth",
        }
    }
}

/// Bluetooth pairing state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingState {
    NotPaired,
    Pairing,
    Paired,
    Failed,
}

/// Manages Bluetooth connections
pub struct BluetoothManager {
    /// Bluetooth adapter interface name (e.g., "hci0")
    interface: String,
    available: bool,
}

impl BluetoothManager {
    /// Create a new Bluetooth manager
    pub fn new(interface: String) -> Result<Self, NetworkError> {
        let available = Self::check_available();

        Ok(Self {
            interface,
            available,
        })
    }

    /// Check if Bluetooth is available
    fn check_available() -> bool {
        Command::new("bluetoothctl")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    }

    /// Check if Bluetooth is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Enable Bluetooth
    pub fn enable(&self) -> Result<(), NetworkError> {
        self.bluetoothctl(&["power", "on"])?;
        tracing::info!("Bluetooth enabled");
        Ok(())
    }

    /// Disable Bluetooth
    pub fn disable(&self) -> Result<(), NetworkError> {
        self.bluetoothctl(&["power", "off"])?;
        tracing::info!("Bluetooth disabled");
        Ok(())
    }

    /// Check if Bluetooth is powered on
    pub fn is_powered(&self) -> bool {
        self.bluetoothctl(&["show"])
            .is_ok_and(|output| output.contains("Powered: yes"))
    }

    /// Start scanning for devices
    pub fn start_scan(&self) -> Result<(), NetworkError> {
        if !self.available {
            return Err(NetworkError::BluetoothNotAvailable);
        }

        self.bluetoothctl(&["scan", "on"])?;
        tracing::debug!("Bluetooth scan started");
        Ok(())
    }

    /// Stop scanning
    pub fn stop_scan(&self) -> Result<(), NetworkError> {
        self.bluetoothctl(&["scan", "off"])?;
        tracing::debug!("Bluetooth scan stopped");
        Ok(())
    }

    /// Get list of discovered devices
    pub fn list_devices(&self) -> Result<Vec<BluetoothDevice>, NetworkError> {
        if !self.available {
            return Err(NetworkError::BluetoothNotAvailable);
        }

        let output = self.bluetoothctl(&["devices"])?;
        let mut devices = Vec::new();

        for line in output.lines() {
            // Format: "Device XX:XX:XX:XX:XX:XX Name"
            if line.starts_with("Device ") {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() >= 3 {
                    let address = parts[1].to_string();
                    let name = parts[2].to_string();

                    // Get device info
                    if let Ok(info) = self.get_device_info(&address) {
                        devices.push(info);
                    } else {
                        devices.push(BluetoothDevice {
                            address,
                            name,
                            device_type: BluetoothDeviceType::Unknown,
                            paired: false,
                            connected: false,
                            trusted: false,
                            rssi: None,
                        });
                    }
                }
            }
        }

        Ok(devices)
    }

    /// Get paired devices
    pub fn list_paired_devices(&self) -> Result<Vec<BluetoothDevice>, NetworkError> {
        if !self.available {
            return Err(NetworkError::BluetoothNotAvailable);
        }

        let output = self.bluetoothctl(&["paired-devices"])?;
        let mut devices = Vec::new();

        for line in output.lines() {
            if line.starts_with("Device ") {
                let parts: Vec<&str> = line.splitn(3, ' ').collect();
                if parts.len() >= 3 {
                    let address = parts[1].to_string();

                    if let Ok(info) = self.get_device_info(&address) {
                        devices.push(info);
                    }
                }
            }
        }

        Ok(devices)
    }

    /// Get detailed device info
    fn get_device_info(&self, address: &str) -> Result<BluetoothDevice, NetworkError> {
        let output = self.bluetoothctl(&["info", address])?;

        let mut device = BluetoothDevice {
            address: address.to_string(),
            name: address.to_string(),
            device_type: BluetoothDeviceType::Unknown,
            paired: false,
            connected: false,
            trusted: false,
            rssi: None,
        };

        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("Name:") {
                device.name = line.trim_start_matches("Name:").trim().to_string();
            } else if line.starts_with("Paired:") {
                device.paired = line.contains("yes");
            } else if line.starts_with("Connected:") {
                device.connected = line.contains("yes");
            } else if line.starts_with("Trusted:") {
                device.trusted = line.contains("yes");
            } else if line.starts_with("Class:") {
                // Avoid if-let chains for MSRV 1.85 compatibility
                #[allow(clippy::collapsible_if)]
                if let Some(class_str) = line.split_whitespace().next_back() {
                    if let Ok(class) = u32::from_str_radix(class_str.trim_start_matches("0x"), 16) {
                        device.device_type = BluetoothDeviceType::from_class(class);
                    }
                }
            } else if line.starts_with("RSSI:") {
                #[allow(clippy::collapsible_if)]
                if let Some(rssi_str) = line.split_whitespace().next_back() {
                    device.rssi = rssi_str.parse().ok();
                }
            }
        }

        Ok(device)
    }

    /// Pair with a device
    pub fn pair(&self, address: &str) -> Result<(), NetworkError> {
        if !self.available {
            return Err(NetworkError::BluetoothNotAvailable);
        }

        tracing::info!("Pairing with device: {}", address);

        // Trust the device first (for auto-reconnect)
        self.bluetoothctl(&["trust", address])?;

        // Attempt pairing
        let output = self.bluetoothctl(&["pair", address])?;

        if output.contains("Pairing successful") || output.contains("already paired") {
            tracing::info!("Paired with {}", address);
            Ok(())
        } else {
            Err(NetworkError::PairingFailed(output))
        }
    }

    /// Connect to a paired device
    pub fn connect(&self, address: &str) -> Result<(), NetworkError> {
        if !self.available {
            return Err(NetworkError::BluetoothNotAvailable);
        }

        tracing::info!("Connecting to device: {}", address);

        let output = self.bluetoothctl(&["connect", address])?;

        if output.contains("Connection successful") || output.contains("Connected: yes") {
            tracing::info!("Connected to {}", address);
            Ok(())
        } else {
            Err(NetworkError::ConnectionFailed(output))
        }
    }

    /// Disconnect from a device
    pub fn disconnect(&self, address: &str) -> Result<(), NetworkError> {
        self.bluetoothctl(&["disconnect", address])?;
        tracing::info!("Disconnected from {}", address);
        Ok(())
    }

    /// Remove/unpair a device
    pub fn remove(&self, address: &str) -> Result<(), NetworkError> {
        self.bluetoothctl(&["remove", address])?;
        tracing::info!("Removed device {}", address);
        Ok(())
    }

    /// Set device as trusted (for auto-connect)
    pub fn trust(&self, address: &str) -> Result<(), NetworkError> {
        self.bluetoothctl(&["trust", address])?;
        Ok(())
    }

    /// Remove trust from device
    pub fn untrust(&self, address: &str) -> Result<(), NetworkError> {
        self.bluetoothctl(&["untrust", address])?;
        Ok(())
    }

    /// Set discoverable mode
    pub fn set_discoverable(&self, enabled: bool) -> Result<(), NetworkError> {
        let arg = if enabled { "on" } else { "off" };
        self.bluetoothctl(&["discoverable", arg])?;
        tracing::debug!("Bluetooth discoverable: {}", arg);
        Ok(())
    }

    /// Set pairable mode
    pub fn set_pairable(&self, enabled: bool) -> Result<(), NetworkError> {
        let arg = if enabled { "on" } else { "off" };
        self.bluetoothctl(&["pairable", arg])?;
        Ok(())
    }

    /// Get connected controllers (game pads)
    pub fn get_connected_controllers(&self) -> Result<Vec<BluetoothDevice>, NetworkError> {
        let devices = self.list_paired_devices()?;

        Ok(devices
            .into_iter()
            .filter(|d| d.connected && d.device_type == BluetoothDeviceType::Controller)
            .collect())
    }

    /// Run bluetoothctl command
    fn bluetoothctl(&self, args: &[&str]) -> Result<String, NetworkError> {
        let output = Command::new("bluetoothctl").args(args).output()?;

        // bluetoothctl often returns success even on failure, check output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !stderr.is_empty() && stderr.contains("Failed") {
            return Err(NetworkError::CommandFailed(stderr));
        }

        Ok(stdout)
    }

    /// Set Bluetooth adapter alias (name)
    pub fn set_alias(&self, name: &str) -> Result<(), NetworkError> {
        self.bluetoothctl(&["system-alias", name])?;
        Ok(())
    }

    /// Get the Bluetooth interface name
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Get adapter info
    pub fn get_adapter_info(&self) -> Result<AdapterInfo, NetworkError> {
        let output = self.bluetoothctl(&["show"])?;

        let mut info = AdapterInfo {
            address: String::new(),
            name: String::new(),
            powered: false,
            discoverable: false,
            pairable: false,
        };

        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("Controller") {
                info.address = line.split_whitespace().nth(1).unwrap_or("").to_string();
            } else if line.starts_with("Name:") {
                info.name = line.trim_start_matches("Name:").trim().to_string();
            } else if line.starts_with("Powered:") {
                info.powered = line.contains("yes");
            } else if line.starts_with("Discoverable:") {
                info.discoverable = line.contains("yes");
            } else if line.starts_with("Pairable:") {
                info.pairable = line.contains("yes");
            }
        }

        Ok(info)
    }
}

/// Bluetooth adapter information
#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub address: String,
    pub name: String,
    pub powered: bool,
    pub discoverable: bool,
    pub pairable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_from_class() {
        // Gamepad class
        assert_eq!(
            BluetoothDeviceType::from_class(0x002508),
            BluetoothDeviceType::Controller
        );

        // Keyboard class
        assert_eq!(
            BluetoothDeviceType::from_class(0x002540),
            BluetoothDeviceType::Keyboard
        );

        // Audio class
        assert_eq!(
            BluetoothDeviceType::from_class(0x240404),
            BluetoothDeviceType::Audio
        );
    }

    #[test]
    fn test_device_type_icon() {
        assert_eq!(BluetoothDeviceType::Controller.icon(), "input-gaming");
        assert_eq!(BluetoothDeviceType::Audio.icon(), "audio-headphones");
    }
}
