//! Hardware Abstraction Layer (HAL)
//!
//! This module provides a hardware abstraction layer for RexOS-supported handheld devices,
//! allowing the rest of the system to interact with hardware through a unified interface.
//!
//! # Supported Devices
//!
//! - Anbernic RG353M/V/VS (RK3566)
//! - Anbernic RG35XX series
//! - More devices to be added
//!
//! # Example
//!
//! ```no_run
//! use rexos_hal::Device;
//!
//! fn main() -> anyhow::Result<()> {
//!     // Auto-detect device
//!     let device = Device::detect()?;
//!     println!("Detected: {}", device.profile().name);
//!     
//!     // Get device capabilities
//!     let profile = device.profile();
//!     println!("Resolution: {}x{}", profile.display.width, profile.display.height);
//!     Ok(())
//! }
//! ```

pub mod device;
pub mod display;
pub mod input;
pub mod audio;
pub mod power;

pub use device::{Device, DeviceProfile, DeviceError, SystemInfo, DisplaySpec};
pub use display::{Display, DisplayConfig, Rotation, BacklightInfo};
pub use input::{InputManager, Button, AnalogStick, InputState, InputDevice, InputEvent};
pub use audio::{AudioManager, AudioConfig, HeadphoneState, AudioProfile};
pub use power::{PowerManager, PowerConfig, BatteryInfo, BatteryStatus, BatteryHealth, CpuGovernor};

/// HAL Result type
pub type Result<T> = std::result::Result<T, DeviceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hal_imports() {
        // Simple smoke test to ensure all modules can be imported
        let _ = std::mem::size_of::<Device>();
    }
}
