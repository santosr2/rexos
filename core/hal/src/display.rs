//! Display management

use crate::DeviceError;

pub struct Display {
    config: DisplayConfig,
}

#[derive(Debug, Clone)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub brightness: u8, // 0-255
    pub rotation: Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rotation {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl Display {
    pub fn new(config: DisplayConfig) -> Result<Self, DeviceError> {
        Ok(Self { config })
    }
    
    pub fn set_brightness(&mut self, level: u8) -> Result<(), DeviceError> {
        tracing::debug!("Setting brightness to {}", level);
        self.config.brightness = level;
        // TODO: Write to hardware
        Ok(())
    }
    
    pub fn get_brightness(&self) -> u8 {
        self.config.brightness
    }
    
    pub fn resolution(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_brightness() {
        let config = DisplayConfig {
            width: 640,
            height: 480,
            brightness: 128,
            rotation: Rotation::Normal,
        };
        
        let mut display = Display::new(config).unwrap();
        assert_eq!(display.get_brightness(), 128);
        
        display.set_brightness(200).unwrap();
        assert_eq!(display.get_brightness(), 200);
    }
}
