//! Mock implementations for testing without real hardware
//!
//! This module provides mock backends for all HAL components, allowing
//! development and testing on desktop systems without Anbernic hardware.
//!
//! # Usage
//!
//! ```no_run
//! use rexos_hal::mock::{MockDevice, MockProfile};
//! use std::path::Path;
//!
//! // Create a mock RG353M device
//! let device = MockDevice::new(MockProfile::Rg353m);
//!
//! // Or use a custom profile
//! let custom = MockDevice::from_profile_file(Path::new("profiles/custom.toml"));
//! ```

use crate::power::CpuGovernor;
use crate::{
    AudioConfig, BatteryHealth, BatteryStatus, Button, DeviceError, DeviceProfile, DisplaySpec,
    HeadphoneState, InputEvent, InputState, Rotation,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Pre-defined mock device profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockProfile {
    /// Anbernic RG353M (RK3566, 640x480, dual analog)
    Rg353m,
    /// Anbernic RG353V (RK3566, 640x480, dual analog, touch)
    Rg353v,
    /// Anbernic RG353VS (RK3566, 640x480, single analog)
    Rg353vs,
    /// Anbernic RG353P (RK3566, 640x480, dual analog, eMMC)
    Rg353p,
    /// Anbernic RG353PS (RK3566, 640x480, single analog, eMMC)
    Rg353ps,
    /// Anbernic RG35XX (H700, 640x480, no analog)
    Rg35xx,
    /// Powkiddy RGB30 (RK3566, 720x720 square display)
    Rgb30,
    /// Anbernic RG503 (RK3566, 960x544 OLED)
    Rg503,
    /// Anbernic RG351P (RK3326, 480x320)
    Rg351p,
    /// Generic QEMU virt machine
    QemuVirt,
    /// Desktop development (no hardware emulation)
    Desktop,
}

impl MockProfile {
    /// Get the device profile for this mock
    pub fn to_device_profile(self) -> DeviceProfile {
        match self {
            MockProfile::Rg353m => DeviceProfile {
                id: "rg353m".into(),
                name: "Mock Anbernic RG353M".into(),
                chipset: "RK3566".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 3500,
                quirks: vec!["mock".into()],
            },
            MockProfile::Rg353v => DeviceProfile {
                id: "rg353v".into(),
                name: "Mock Anbernic RG353V".into(),
                chipset: "RK3566".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 3500,
                quirks: vec!["mock".into(), "touchscreen".into()],
            },
            MockProfile::Rg353vs => DeviceProfile {
                id: "rg353vs".into(),
                name: "Mock Anbernic RG353VS".into(),
                chipset: "RK3566".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 1,
                battery_capacity: 3500,
                quirks: vec!["mock".into()],
            },
            MockProfile::Rg353p => DeviceProfile {
                id: "rg353p".into(),
                name: "Mock Anbernic RG353P".into(),
                chipset: "RK3566".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 3500,
                quirks: vec!["mock".into(), "emmc_storage".into()],
            },
            MockProfile::Rg353ps => DeviceProfile {
                id: "rg353ps".into(),
                name: "Mock Anbernic RG353PS".into(),
                chipset: "RK3566".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 1,
                battery_capacity: 3500,
                quirks: vec!["mock".into(), "emmc_storage".into()],
            },
            MockProfile::Rg35xx => DeviceProfile {
                id: "rg35xx".into(),
                name: "Mock Anbernic RG35XX".into(),
                chipset: "H700".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: vec![
                    "up", "down", "left", "right", "a", "b", "x", "y", "l1", "r1", "start",
                    "select",
                ]
                .into_iter()
                .map(Into::into)
                .collect(),
                analog_sticks: 0,
                battery_capacity: 2600,
                quirks: vec!["mock".into(), "no_analog".into(), "no_l2r2".into()],
            },
            MockProfile::Rgb30 => DeviceProfile {
                id: "rgb30".into(),
                name: "Mock Powkiddy RGB30".into(),
                chipset: "RK3566".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 720,
                    height: 720,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 4100,
                quirks: vec!["mock".into(), "square_display".into()],
            },
            MockProfile::Rg503 => DeviceProfile {
                id: "rg503".into(),
                name: "Mock Anbernic RG503".into(),
                chipset: "RK3566".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 960,
                    height: 544,
                    format: "RGB888".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 3500,
                quirks: vec!["mock".into(), "oled_display".into()],
            },
            MockProfile::Rg351p => DeviceProfile {
                id: "rg351p".into(),
                name: "Mock Anbernic RG351P".into(),
                chipset: "RK3326".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 480,
                    height: 320,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 3500,
                quirks: vec!["mock".into()],
            },
            MockProfile::QemuVirt => DeviceProfile {
                id: "qemu_virt".into(),
                name: "QEMU Virtual Machine".into(),
                chipset: "virt".into(),
                architecture: "aarch64".into(),
                display: DisplaySpec {
                    width: 640,
                    height: 480,
                    format: "RGB565".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 5000,
                quirks: vec!["mock".into(), "qemu".into(), "virtio".into()],
            },
            MockProfile::Desktop => DeviceProfile {
                id: "desktop".into(),
                name: "Desktop Development".into(),
                chipset: "x86_64".into(),
                architecture: std::env::consts::ARCH.into(),
                display: DisplaySpec {
                    width: 800,
                    height: 600,
                    format: "RGB888".into(),
                    refresh_rate: 60,
                },
                buttons: standard_buttons(),
                analog_sticks: 2,
                battery_capacity: 10000,
                quirks: vec!["mock".into(), "desktop".into()],
            },
        }
    }

    /// Get profile from string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "rg353m" => Some(MockProfile::Rg353m),
            "rg353v" => Some(MockProfile::Rg353v),
            "rg353vs" => Some(MockProfile::Rg353vs),
            "rg353p" => Some(MockProfile::Rg353p),
            "rg353ps" => Some(MockProfile::Rg353ps),
            "rg35xx" => Some(MockProfile::Rg35xx),
            "rgb30" => Some(MockProfile::Rgb30),
            "rg503" => Some(MockProfile::Rg503),
            "rg351p" => Some(MockProfile::Rg351p),
            "qemu" | "qemu_virt" => Some(MockProfile::QemuVirt),
            "desktop" => Some(MockProfile::Desktop),
            _ => None,
        }
    }

    /// List all available mock profiles
    pub fn all() -> &'static [MockProfile] {
        &[
            MockProfile::Rg353m,
            MockProfile::Rg353v,
            MockProfile::Rg353vs,
            MockProfile::Rg353p,
            MockProfile::Rg353ps,
            MockProfile::Rg35xx,
            MockProfile::Rgb30,
            MockProfile::Rg503,
            MockProfile::Rg351p,
            MockProfile::QemuVirt,
            MockProfile::Desktop,
        ]
    }
}

fn standard_buttons() -> Vec<String> {
    vec![
        "up", "down", "left", "right", "a", "b", "x", "y", "l1", "l2", "r1", "r2", "start",
        "select",
    ]
    .into_iter()
    .map(Into::into)
    .collect()
}

/// Mock battery info (simplified for testing)
#[derive(Debug, Clone)]
pub struct MockBatteryInfo {
    pub status: BatteryStatus,
    pub capacity: u8,
    pub voltage_now: u32,
    pub current_now: Option<i32>,
    pub temperature: Option<i32>,
    pub health: BatteryHealth,
}

impl Default for MockBatteryInfo {
    fn default() -> Self {
        Self {
            status: BatteryStatus::Discharging,
            capacity: 85,
            voltage_now: 3800,
            current_now: Some(-500),
            temperature: Some(35),
            health: BatteryHealth::Good,
        }
    }
}

/// Shared mock state for synchronized access
#[derive(Debug)]
pub struct MockState {
    /// Current brightness (0-255)
    pub brightness: u8,
    /// Current rotation
    pub rotation: Rotation,
    /// Display power state
    pub display_on: bool,
    /// Current volume (0-100)
    pub volume: u8,
    /// Audio muted
    pub muted: bool,
    /// Headphone state
    pub headphones: HeadphoneState,
    /// Battery info
    pub battery: MockBatteryInfo,
    /// CPU governor
    pub governor: CpuGovernor,
    /// Button states
    pub buttons: HashMap<Button, bool>,
    /// Left stick position
    pub left_stick: (i16, i16),
    /// Right stick position
    pub right_stick: (i16, i16),
    /// Pending input events
    pub pending_events: Vec<InputEvent>,
}

impl MockState {
    pub fn new() -> Self {
        let mut buttons = HashMap::new();
        for button in Button::all() {
            buttons.insert(*button, false);
        }

        Self {
            brightness: 180,
            rotation: Rotation::Normal,
            display_on: true,
            volume: 50,
            muted: false,
            headphones: HeadphoneState::Disconnected,
            battery: MockBatteryInfo::default(),
            governor: CpuGovernor::Ondemand,
            buttons,
            left_stick: (0, 0),
            right_stick: (0, 0),
            pending_events: Vec::new(),
        }
    }
}

impl Default for MockState {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock device for testing
pub struct MockDevice {
    profile: DeviceProfile,
    state: Arc<RwLock<MockState>>,
}

impl MockDevice {
    /// Create a new mock device with the given profile
    pub fn new(profile: MockProfile) -> Self {
        Self {
            profile: profile.to_device_profile(),
            state: Arc::new(RwLock::new(MockState::new())),
        }
    }

    /// Create from a TOML profile file
    pub fn from_profile_file(path: &Path) -> Result<Self, DeviceError> {
        let contents = std::fs::read_to_string(path)?;
        let profile: DeviceProfile = toml::from_str(&contents)
            .map_err(|e| DeviceError::InitializationFailed(e.to_string()))?;

        Ok(Self {
            profile,
            state: Arc::new(RwLock::new(MockState::new())),
        })
    }

    /// Create from environment variable or default to Desktop
    pub fn from_env() -> Self {
        let profile = std::env::var("REXOS_MOCK_DEVICE")
            .ok()
            .and_then(|s| MockProfile::from_name(&s))
            .unwrap_or(MockProfile::Desktop);

        Self::new(profile)
    }

    /// Get device profile
    pub fn profile(&self) -> &DeviceProfile {
        &self.profile
    }

    /// Get shared state for manipulation in tests
    pub fn state(&self) -> Arc<RwLock<MockState>> {
        Arc::clone(&self.state)
    }

    /// Check if this is a mock device
    pub fn is_mock(&self) -> bool {
        self.profile.quirks.contains(&"mock".into())
    }

    /// Check if running under QEMU
    pub fn is_qemu(&self) -> bool {
        self.profile.quirks.contains(&"qemu".into())
    }
}

/// Mock display for testing
pub struct MockDisplay {
    config: crate::DisplayConfig,
    state: Arc<RwLock<MockState>>,
}

impl MockDisplay {
    pub fn new(profile: &DeviceProfile, state: Arc<RwLock<MockState>>) -> Self {
        Self {
            config: crate::DisplayConfig {
                width: profile.display.width,
                height: profile.display.height,
                brightness: 180,
                rotation: Rotation::Normal,
                backlight_path: "/mock/backlight".into(),
                max_brightness: 255,
            },
            state,
        }
    }

    pub fn set_brightness(&mut self, level: u8) -> Result<(), DeviceError> {
        self.config.brightness = level;
        if let Ok(mut state) = self.state.write() {
            state.brightness = level;
        }
        tracing::debug!("[MOCK] Brightness set to {}", level);
        Ok(())
    }

    pub fn get_brightness(&self) -> u8 {
        self.state
            .read()
            .map(|s| s.brightness)
            .unwrap_or(self.config.brightness)
    }

    pub fn set_rotation(&mut self, rotation: Rotation) -> Result<(), DeviceError> {
        self.config.rotation = rotation;
        if let Ok(mut state) = self.state.write() {
            state.rotation = rotation;
        }
        tracing::debug!("[MOCK] Rotation set to {} degrees", rotation.degrees());
        Ok(())
    }

    pub fn power_on(&self) -> Result<(), DeviceError> {
        if let Ok(mut state) = self.state.write() {
            state.display_on = true;
        }
        tracing::debug!("[MOCK] Display powered on");
        Ok(())
    }

    pub fn power_off(&self) -> Result<(), DeviceError> {
        if let Ok(mut state) = self.state.write() {
            state.display_on = false;
        }
        tracing::debug!("[MOCK] Display powered off");
        Ok(())
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

/// Mock audio manager for testing
pub struct MockAudio {
    config: AudioConfig,
    state: Arc<RwLock<MockState>>,
}

impl MockAudio {
    pub fn new(profile: &DeviceProfile, state: Arc<RwLock<MockState>>) -> Self {
        let _ = profile; // Used for future device-specific audio profiles
        Self {
            config: AudioConfig::default(),
            state,
        }
    }

    pub fn set_volume(&mut self, volume: u8) -> Result<(), DeviceError> {
        let clamped = volume.min(100);
        self.config.volume = clamped;
        if let Ok(mut state) = self.state.write() {
            state.volume = clamped;
        }
        tracing::debug!("[MOCK] Volume set to {}%", clamped);
        Ok(())
    }

    pub fn get_volume(&self) -> u8 {
        self.state
            .read()
            .map(|s| s.volume)
            .unwrap_or(self.config.volume)
    }

    pub fn set_mute(&mut self, muted: bool) -> Result<(), DeviceError> {
        if let Ok(mut state) = self.state.write() {
            state.muted = muted;
        }
        tracing::debug!("[MOCK] Audio muted: {}", muted);
        Ok(())
    }

    pub fn headphone_state(&self) -> HeadphoneState {
        self.state
            .read()
            .map(|s| s.headphones)
            .unwrap_or(HeadphoneState::Disconnected)
    }
}

/// Mock input manager for testing
pub struct MockInput {
    state: Arc<RwLock<MockState>>,
    deadzone: i16,
}

impl MockInput {
    pub fn new(state: Arc<RwLock<MockState>>) -> Self {
        Self {
            state,
            deadzone: 4096,
        }
    }

    /// Simulate pressing a button
    pub fn press_button(&self, button: Button) {
        if let Ok(mut state) = self.state.write() {
            state.buttons.insert(button, true);
        }
    }

    /// Simulate releasing a button
    pub fn release_button(&self, button: Button) {
        if let Ok(mut state) = self.state.write() {
            state.buttons.insert(button, false);
        }
    }

    /// Simulate moving the left stick
    pub fn set_left_stick(&self, x: i16, y: i16) {
        if let Ok(mut state) = self.state.write() {
            state.left_stick = (x, y);
        }
    }

    /// Simulate moving the right stick
    pub fn set_right_stick(&self, x: i16, y: i16) {
        if let Ok(mut state) = self.state.write() {
            state.right_stick = (x, y);
        }
    }

    /// Get current input state
    pub fn get_state(&self) -> InputState {
        self.state
            .read()
            .map(|s| InputState {
                buttons: s.buttons.clone(),
                left_stick: crate::AnalogStick {
                    x: s.left_stick.0,
                    y: s.left_stick.1,
                },
                right_stick: crate::AnalogStick {
                    x: s.right_stick.0,
                    y: s.right_stick.1,
                },
                l2_analog: 0,
                r2_analog: 0,
            })
            .unwrap_or_default()
    }

    /// Check if button is pressed
    pub fn is_pressed(&self, button: Button) -> bool {
        self.state
            .read()
            .map(|s| *s.buttons.get(&button).unwrap_or(&false))
            .unwrap_or(false)
    }

    pub fn deadzone(&self) -> i16 {
        self.deadzone
    }
}

/// Mock power configuration
#[derive(Debug, Clone)]
pub struct MockPowerConfig {
    pub battery_capacity: u32,
    pub low_battery_threshold: u8,
    pub critical_battery_threshold: u8,
    pub auto_sleep_timeout: u32,
}

/// Mock power manager for testing
pub struct MockPower {
    config: MockPowerConfig,
    state: Arc<RwLock<MockState>>,
}

impl MockPower {
    pub fn new(profile: &DeviceProfile, state: Arc<RwLock<MockState>>) -> Self {
        Self {
            config: MockPowerConfig {
                battery_capacity: profile.battery_capacity,
                low_battery_threshold: 15,
                critical_battery_threshold: 5,
                auto_sleep_timeout: 300,
            },
            state,
        }
    }

    pub fn battery_info(&self) -> MockBatteryInfo {
        self.state
            .read()
            .map(|s| s.battery.clone())
            .unwrap_or_else(|_| MockBatteryInfo {
                status: BatteryStatus::Unknown,
                capacity: 0,
                voltage_now: 0,
                current_now: None,
                temperature: None,
                health: BatteryHealth::Unknown,
            })
    }

    /// Simulate battery level change (for testing)
    pub fn set_battery_capacity(&self, capacity: u8) {
        if let Ok(mut state) = self.state.write() {
            state.battery.capacity = capacity.min(100);
        }
    }

    /// Simulate charging state
    pub fn set_charging(&self, charging: bool) {
        if let Ok(mut state) = self.state.write() {
            state.battery.status = if charging {
                BatteryStatus::Charging
            } else {
                BatteryStatus::Discharging
            };
        }
    }

    pub fn set_governor(&mut self, governor: CpuGovernor) -> Result<(), DeviceError> {
        let _ = self.config.auto_sleep_timeout; // Silence unused warning
        if let Ok(mut state) = self.state.write() {
            state.governor = governor;
        }
        tracing::debug!("[MOCK] CPU governor set to {:?}", governor);
        Ok(())
    }

    pub fn get_governor(&self) -> CpuGovernor {
        self.state
            .read()
            .map(|s| s.governor)
            .unwrap_or(CpuGovernor::Ondemand)
    }
}

/// Complete mock HAL for testing
pub struct MockHal {
    pub device: MockDevice,
    pub display: MockDisplay,
    pub audio: MockAudio,
    pub input: MockInput,
    pub power: MockPower,
}

impl MockHal {
    /// Create a complete mock HAL with the given profile
    pub fn new(profile: MockProfile) -> Self {
        let device = MockDevice::new(profile);
        let state = device.state();

        Self {
            display: MockDisplay::new(device.profile(), Arc::clone(&state)),
            audio: MockAudio::new(device.profile(), Arc::clone(&state)),
            input: MockInput::new(Arc::clone(&state)),
            power: MockPower::new(device.profile(), Arc::clone(&state)),
            device,
        }
    }

    /// Create from environment or default
    pub fn from_env() -> Self {
        let profile = std::env::var("REXOS_MOCK_DEVICE")
            .ok()
            .and_then(|s| MockProfile::from_name(&s))
            .unwrap_or(MockProfile::Desktop);

        Self::new(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_profiles() {
        for profile in MockProfile::all() {
            let device_profile = profile.to_device_profile();
            assert!(!device_profile.name.is_empty());
            assert!(device_profile.quirks.contains(&"mock".into()));
        }
    }

    #[test]
    fn test_mock_device_creation() {
        let device = MockDevice::new(MockProfile::Rg353m);
        assert!(device.is_mock());
        assert!(!device.is_qemu());
        assert_eq!(device.profile().id, "rg353m");
    }

    #[test]
    fn test_mock_qemu_device() {
        let device = MockDevice::new(MockProfile::QemuVirt);
        assert!(device.is_mock());
        assert!(device.is_qemu());
    }

    #[test]
    fn test_mock_display() {
        let device = MockDevice::new(MockProfile::Rg353m);
        let mut display = MockDisplay::new(device.profile(), device.state());

        assert_eq!(display.resolution(), (640, 480));

        display.set_brightness(100).unwrap();
        assert_eq!(display.get_brightness(), 100);

        display.set_rotation(Rotation::Rotate90).unwrap();
        display.power_off().unwrap();
        display.power_on().unwrap();
    }

    #[test]
    fn test_mock_input() {
        let device = MockDevice::new(MockProfile::Rg353m);
        let input = MockInput::new(device.state());

        assert!(!input.is_pressed(Button::A));

        input.press_button(Button::A);
        assert!(input.is_pressed(Button::A));

        input.release_button(Button::A);
        assert!(!input.is_pressed(Button::A));

        input.set_left_stick(10000, -5000);
        let state = input.get_state();
        assert_eq!(state.left_stick.x, 10000);
        assert_eq!(state.left_stick.y, -5000);
    }

    #[test]
    fn test_mock_power() {
        let device = MockDevice::new(MockProfile::Rg353m);
        let power = MockPower::new(device.profile(), device.state());

        let info = power.battery_info();
        assert_eq!(info.capacity, 85);
        assert_eq!(info.status, BatteryStatus::Discharging);

        power.set_battery_capacity(50);
        assert_eq!(power.battery_info().capacity, 50);

        power.set_charging(true);
        assert_eq!(power.battery_info().status, BatteryStatus::Charging);
    }

    #[test]
    fn test_mock_hal_complete() {
        let hal = MockHal::new(MockProfile::Rg353m);

        assert!(hal.device.is_mock());
        assert_eq!(hal.display.resolution(), (640, 480));
        assert!(!hal.input.is_pressed(Button::Start));
        assert_eq!(hal.power.battery_info().capacity, 85);
    }

    #[test]
    fn test_profile_from_name() {
        assert_eq!(MockProfile::from_name("rg353m"), Some(MockProfile::Rg353m));
        assert_eq!(MockProfile::from_name("QEMU"), Some(MockProfile::QemuVirt));
        assert_eq!(MockProfile::from_name("invalid"), None);
    }
}
