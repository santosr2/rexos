//! Input management
//!
//! Handles gamepad input via evdev (Linux event devices).
//! Supports both GPIO buttons and USB/Bluetooth controllers.

use crate::DeviceError;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Gamepad buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    L1,
    L2,
    R1,
    R2,
    L3, // Left stick click
    R3, // Right stick click
    Start,
    Select,
    Home,
}

impl Button {
    /// Get all standard buttons
    pub fn all() -> &'static [Button] {
        &[
            Button::Up, Button::Down, Button::Left, Button::Right,
            Button::A, Button::B, Button::X, Button::Y,
            Button::L1, Button::L2, Button::R1, Button::R2,
            Button::L3, Button::R3,
            Button::Start, Button::Select, Button::Home,
        ]
    }

    /// Get button name
    pub fn name(&self) -> &'static str {
        match self {
            Button::Up => "up",
            Button::Down => "down",
            Button::Left => "left",
            Button::Right => "right",
            Button::A => "a",
            Button::B => "b",
            Button::X => "x",
            Button::Y => "y",
            Button::L1 => "l1",
            Button::L2 => "l2",
            Button::R1 => "r1",
            Button::R2 => "r2",
            Button::L3 => "l3",
            Button::R3 => "r3",
            Button::Start => "start",
            Button::Select => "select",
            Button::Home => "home",
        }
    }
}

/// Analog stick state
#[derive(Debug, Clone, Copy, Default)]
pub struct AnalogStick {
    pub x: i16, // -32768 to 32767
    pub y: i16,
}

impl AnalogStick {
    /// Check if the stick is within deadzone
    pub fn is_neutral(&self, deadzone: i16) -> bool {
        self.x.abs() < deadzone && self.y.abs() < deadzone
    }

    /// Get normalized values (0.0 to 1.0)
    pub fn normalized(&self) -> (f32, f32) {
        (
            self.x as f32 / 32767.0,
            self.y as f32 / 32767.0,
        )
    }
}

/// Input event types (from linux/input-event-codes.h)
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Syn = 0x00,
    Key = 0x01,
    Rel = 0x02,
    Abs = 0x03,
}

/// Raw input event from evdev
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InputEvent {
    pub tv_sec: i64,
    pub tv_usec: i64,
    pub event_type: u16,
    pub code: u16,
    pub value: i32,
}

/// Input device information
#[derive(Debug, Clone)]
pub struct InputDevice {
    pub path: PathBuf,
    pub name: String,
    pub is_gamepad: bool,
    pub has_analog: bool,
}

/// State of all inputs
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub buttons: HashMap<Button, bool>,
    pub left_stick: AnalogStick,
    pub right_stick: AnalogStick,
    pub l2_analog: i16, // Analog trigger
    pub r2_analog: i16,
}

/// Manages input devices
pub struct InputManager {
    devices: Vec<InputDevice>,
    device_files: Vec<File>,
    state: InputState,
    deadzone: i16,
    button_map: HashMap<u16, Button>,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Result<Self, DeviceError> {
        let mut manager = Self {
            devices: Vec::new(),
            device_files: Vec::new(),
            state: InputState::default(),
            deadzone: 4096,
            button_map: Self::default_button_map(),
        };

        // Initialize button states
        for button in Button::all() {
            manager.state.buttons.insert(*button, false);
        }

        // Scan for input devices
        manager.scan_devices()?;

        Ok(manager)
    }

    /// Create with custom deadzone
    pub fn with_deadzone(deadzone: i16) -> Result<Self, DeviceError> {
        let mut manager = Self::new()?;
        manager.deadzone = deadzone;
        Ok(manager)
    }

    /// Scan for available input devices
    pub fn scan_devices(&mut self) -> Result<(), DeviceError> {
        self.devices.clear();
        self.device_files.clear();

        let input_dir = Path::new("/dev/input");
        if !input_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(input_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Only look at event devices
            if !name.starts_with("event") {
                continue;
            }

            // Use if-let chain (Rust 2024) to reduce nesting
            if let Ok(device) = self.probe_device(&path)
                && device.is_gamepad
                && let Ok(file) = File::open(&path)
            {
                tracing::info!("Found gamepad: {} at {}", device.name, path.display());
                self.device_files.push(file);
                self.devices.push(device);
            }
        }

        Ok(())
    }

    /// Probe a device to determine its capabilities
    fn probe_device(&self, path: &Path) -> Result<InputDevice, DeviceError> {
        // Read device name from sysfs
        let sysfs_name = path.file_name().unwrap_or_default().to_string_lossy();
        let sysfs_path = format!("/sys/class/input/{}/device/name", sysfs_name);

        let name = fs::read_to_string(&sysfs_path)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        // Check capabilities from sysfs
        let caps_path = format!("/sys/class/input/{}/device/capabilities/key", sysfs_name);
        let has_keys = fs::read_to_string(&caps_path).is_ok();

        let abs_path = format!("/sys/class/input/{}/device/capabilities/abs", sysfs_name);
        let has_analog = fs::read_to_string(&abs_path)
            .map(|s| !s.trim().is_empty() && s.trim() != "0")
            .unwrap_or(false);

        // Heuristics for gamepad detection
        let is_gamepad = has_keys && (
            name.to_lowercase().contains("gamepad") ||
            name.to_lowercase().contains("joystick") ||
            name.to_lowercase().contains("controller") ||
            name.to_lowercase().contains("gpio") ||
            name.to_lowercase().contains("odroidgo") ||
            name.to_lowercase().contains("anbernic") ||
            name.to_lowercase().contains("rg351") ||
            name.to_lowercase().contains("rg353")
        );

        Ok(InputDevice {
            path: path.to_path_buf(),
            name,
            is_gamepad,
            has_analog,
        })
    }

    /// Default button mapping for common Linux key codes
    fn default_button_map() -> HashMap<u16, Button> {
        let mut map = HashMap::new();

        // Standard gamepad buttons (BTN_* from input-event-codes.h)
        map.insert(304, Button::A);      // BTN_SOUTH / BTN_A
        map.insert(305, Button::B);      // BTN_EAST / BTN_B
        map.insert(307, Button::X);      // BTN_NORTH / BTN_X
        map.insert(308, Button::Y);      // BTN_WEST / BTN_Y
        map.insert(310, Button::L1);     // BTN_TL
        map.insert(311, Button::R1);     // BTN_TR
        map.insert(312, Button::L2);     // BTN_TL2
        map.insert(313, Button::R2);     // BTN_TR2
        map.insert(314, Button::Select); // BTN_SELECT
        map.insert(315, Button::Start);  // BTN_START
        map.insert(316, Button::Home);   // BTN_MODE
        map.insert(317, Button::L3);     // BTN_THUMBL
        map.insert(318, Button::R3);     // BTN_THUMBR

        // D-pad as buttons
        map.insert(544, Button::Up);     // BTN_DPAD_UP
        map.insert(545, Button::Down);   // BTN_DPAD_DOWN
        map.insert(546, Button::Left);   // BTN_DPAD_LEFT
        map.insert(547, Button::Right);  // BTN_DPAD_RIGHT

        map
    }

    /// Poll for input events (non-blocking)
    pub fn poll(&mut self) -> Result<Vec<InputEvent>, DeviceError> {
        let mut events = Vec::new();

        for file in &mut self.device_files {
            let mut buffer = [0u8; std::mem::size_of::<InputEvent>()];

            // Use poll or select in production; here we just try reading
            loop {
                match file.read(&mut buffer) {
                    Ok(size) if size == buffer.len() => {
                        // SAFETY: InputEvent is repr(C) and the buffer is correctly sized
                        let event: InputEvent = unsafe {
                            std::ptr::read(buffer.as_ptr() as *const InputEvent)
                        };
                        events.push(event);
                    }
                    _ => break,
                }
            }

            // Reset file position for next poll
            let _ = file.seek(SeekFrom::End(0));
        }

        // Process events after collecting them (avoids borrow issue)
        for event in &events {
            self.process_event(event);
        }

        Ok(events)
    }

    /// Process a raw input event
    fn process_event(&mut self, event: &InputEvent) {
        match event.event_type {
            // Key/Button event
            0x01 => {
                if let Some(&button) = self.button_map.get(&event.code) {
                    self.state.buttons.insert(button, event.value != 0);
                }
            }
            // Absolute axis event
            0x03 => {
                match event.code {
                    // Left stick
                    0x00 => self.state.left_stick.x = event.value as i16,  // ABS_X
                    0x01 => self.state.left_stick.y = event.value as i16,  // ABS_Y
                    // Right stick
                    0x03 => self.state.right_stick.x = event.value as i16, // ABS_RX
                    0x04 => self.state.right_stick.y = event.value as i16, // ABS_RY
                    // Triggers
                    0x02 => self.state.l2_analog = event.value as i16,     // ABS_Z
                    0x05 => self.state.r2_analog = event.value as i16,     // ABS_RZ
                    // D-pad as axes (HAT)
                    0x10 => { // ABS_HAT0X
                        self.state.buttons.insert(Button::Left, event.value < 0);
                        self.state.buttons.insert(Button::Right, event.value > 0);
                    }
                    0x11 => { // ABS_HAT0Y
                        self.state.buttons.insert(Button::Up, event.value < 0);
                        self.state.buttons.insert(Button::Down, event.value > 0);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Get current input state
    pub fn state(&self) -> &InputState {
        &self.state
    }

    /// Check if a button is pressed
    pub fn is_pressed(&self, button: Button) -> bool {
        *self.state.buttons.get(&button).unwrap_or(&false)
    }

    /// Check if a button combination is pressed
    pub fn is_combo_pressed(&self, buttons: &[Button]) -> bool {
        buttons.iter().all(|b| self.is_pressed(*b))
    }

    /// Get left analog stick state
    pub fn left_stick(&self) -> AnalogStick {
        self.state.left_stick
    }

    /// Get right analog stick state
    pub fn right_stick(&self) -> AnalogStick {
        self.state.right_stick
    }

    /// Get deadzone setting
    pub fn deadzone(&self) -> i16 {
        self.deadzone
    }

    /// Set deadzone
    pub fn set_deadzone(&mut self, deadzone: i16) {
        self.deadzone = deadzone;
    }

    /// Get list of detected devices
    pub fn devices(&self) -> &[InputDevice] {
        &self.devices
    }

    /// Set custom button mapping
    pub fn set_button_map(&mut self, map: HashMap<u16, Button>) {
        self.button_map = map;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            devices: Vec::new(),
            device_files: Vec::new(),
            state: InputState::default(),
            deadzone: 4096,
            button_map: Self::default_button_map(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_names() {
        assert_eq!(Button::A.name(), "a");
        assert_eq!(Button::Start.name(), "start");
    }

    #[test]
    fn test_analog_stick_neutral() {
        let stick = AnalogStick { x: 100, y: -50 };
        assert!(stick.is_neutral(4096));

        let stick = AnalogStick { x: 10000, y: 0 };
        assert!(!stick.is_neutral(4096));
    }

    #[test]
    fn test_analog_stick_normalized() {
        let stick = AnalogStick { x: 32767, y: -32768 };
        let (nx, ny) = stick.normalized();
        assert!((nx - 1.0).abs() < 0.01);
        assert!((ny + 1.0).abs() < 0.01);
    }
}
