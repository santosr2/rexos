//! Audio system management

use crate::DeviceError;

pub struct AudioManager {
    config: AudioConfig,
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub volume: u8, // 0-100
    pub sample_rate: u32,
}

impl AudioManager {
    pub fn new(config: AudioConfig) -> Result<Self, DeviceError> {
        Ok(Self { config })
    }
    
    pub fn set_volume(&mut self, volume: u8) -> Result<(), DeviceError> {
        self.config.volume = volume.min(100);
        tracing::debug!("Volume set to {}", self.config.volume);
        // TODO: Apply to hardware
        Ok(())
    }
    
    pub fn get_volume(&self) -> u8 {
        self.config.volume
    }
}
