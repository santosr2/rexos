//! Input management

use crate::DeviceError;

pub struct InputManager {
    // TODO: Add input device handles
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Start,
    Select,
}

#[derive(Debug, Clone, Copy)]
pub struct AnalogStick {
    pub x: i16, // -32768 to 32767
    pub y: i16,
}

impl InputManager {
    pub fn new() -> Result<Self, DeviceError> {
        Ok(Self {})
    }
    
    // TODO: Implement input polling
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
