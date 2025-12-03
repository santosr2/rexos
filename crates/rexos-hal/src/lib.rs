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

pub mod audio;
pub mod device;
pub mod display;
pub mod input;
pub mod power;

pub use audio::{AudioConfig, AudioManager, AudioProfile, HeadphoneState};
pub use device::{Device, DeviceError, DeviceProfile, DisplaySpec, SystemInfo};
pub use display::{BacklightInfo, Display, DisplayConfig, Rotation};
pub use input::{AnalogStick, Button, InputDevice, InputEvent, InputManager, InputState};
pub use power::{
    BatteryHealth, BatteryInfo, BatteryStatus, CpuGovernor, PowerConfig, PowerManager,
};

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
