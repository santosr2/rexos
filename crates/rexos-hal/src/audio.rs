//! Audio system management
//!
//! Handles audio output via ALSA and headphone detection.

use crate::DeviceError;
use std::fs;
use std::process::Command;

/// Audio configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub volume: u8,
    pub sample_rate: u32,
    pub muted: bool,
    pub alsa_card: String,
    pub mixer_control: String,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            volume: 70,
            sample_rate: 48000,
            muted: false,
            alsa_card: "default".to_string(),
            mixer_control: "Playback".to_string(),
        }
    }
}

/// Headphone connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadphoneState {
    Connected,
    Disconnected,
    Unknown,
}

/// Audio output profile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioProfile {
    Speaker,
    Headphones,
    Hdmi,
}

/// Audio manager
pub struct AudioManager {
    config: AudioConfig,
    previous_volume: u8,
}

impl AudioManager {
    /// Create a new audio manager
    pub fn new(config: AudioConfig) -> Result<Self, DeviceError> {
        let mut manager = Self {
            config,
            previous_volume: 70,
        };

        // Apply initial volume
        let volume = manager.config.volume;
        manager.set_volume(volume)?;

        Ok(manager)
    }

    /// Set volume (0-100)
    pub fn set_volume(&mut self, volume: u8) -> Result<(), DeviceError> {
        let volume = volume.min(100);
        self.config.volume = volume;

        // Use amixer to set volume
        let result = Command::new("amixer")
            .args([
                "-c", &self.config.alsa_card,
                "sset", &self.config.mixer_control,
                &format!("{}%", volume),
            ])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    tracing::debug!("Volume set to {}%", volume);
                } else {
                    // Fallback: try with 'Master' control
                    let _ = Command::new("amixer")
                        .args(["sset", "Master", &format!("{}%", volume)])
                        .output();
                }
            }
            Err(e) => {
                tracing::warn!("Failed to set volume via amixer: {}", e);
            }
        }

        Ok(())
    }

    /// Get current volume (0-100)
    pub fn get_volume(&self) -> u8 {
        self.config.volume
    }

    /// Increase volume by step
    pub fn volume_up(&mut self, step: u8) -> Result<(), DeviceError> {
        let new_volume = self.config.volume.saturating_add(step).min(100);
        self.set_volume(new_volume)
    }

    /// Decrease volume by step
    pub fn volume_down(&mut self, step: u8) -> Result<(), DeviceError> {
        let new_volume = self.config.volume.saturating_sub(step);
        self.set_volume(new_volume)
    }

    /// Mute audio
    pub fn mute(&mut self) -> Result<(), DeviceError> {
        if !self.config.muted {
            self.previous_volume = self.config.volume;
            self.config.muted = true;

            let _ = Command::new("amixer")
                .args(["-c", &self.config.alsa_card, "sset", &self.config.mixer_control, "mute"])
                .output();

            tracing::info!("Audio muted");
        }
        Ok(())
    }

    /// Unmute audio
    pub fn unmute(&mut self) -> Result<(), DeviceError> {
        if self.config.muted {
            self.config.muted = false;

            let _ = Command::new("amixer")
                .args(["-c", &self.config.alsa_card, "sset", &self.config.mixer_control, "unmute"])
                .output();

            self.set_volume(self.previous_volume)?;
            tracing::info!("Audio unmuted");
        }
        Ok(())
    }

    /// Toggle mute
    pub fn toggle_mute(&mut self) -> Result<(), DeviceError> {
        if self.config.muted {
            self.unmute()
        } else {
            self.mute()
        }
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.config.muted
    }

    /// Get headphone connection state
    pub fn headphone_state(&self) -> HeadphoneState {
        // Check common headphone detection paths
        let paths = [
            "/sys/class/switch/h2w/state",
            "/sys/devices/platform/sound/jack",
            "/sys/class/extcon/extcon0/state",
        ];

        for path in &paths {
            if let Ok(contents) = fs::read_to_string(path) {
                let state = contents.trim();
                if state == "1" || state.contains("HEADPHONE") {
                    return HeadphoneState::Connected;
                } else if state == "0" || state.is_empty() {
                    return HeadphoneState::Disconnected;
                }
            }
        }

        HeadphoneState::Unknown
    }

    /// Check if headphones are connected
    pub fn is_headphones_connected(&self) -> bool {
        self.headphone_state() == HeadphoneState::Connected
    }

    /// Get current audio profile based on detected output
    pub fn current_profile(&self) -> AudioProfile {
        if self.is_headphones_connected() {
            AudioProfile::Headphones
        } else {
            AudioProfile::Speaker
        }
    }

    /// Set sample rate
    pub fn set_sample_rate(&mut self, rate: u32) -> Result<(), DeviceError> {
        self.config.sample_rate = rate;
        // Sample rate is typically set per-application via ALSA config
        tracing::debug!("Sample rate set to {}", rate);
        Ok(())
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Get audio configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// List available ALSA cards
    pub fn list_cards() -> Vec<String> {
        let mut cards = Vec::new();

        // Read from /proc/asound/cards
        let Ok(contents) = fs::read_to_string("/proc/asound/cards") else {
            return cards;
        };

        for line in contents.lines() {
            // Use if-let chain (Rust 2024) to reduce nesting
            if let Some(idx) = line.find('[')
                && let Some(end) = line.find(']')
            {
                let name = line[idx + 1..end].trim();
                if !name.is_empty() {
                    cards.push(name.to_string());
                }
            }
        }

        cards
    }

    /// Play a simple beep (for system feedback)
    pub fn beep(&self) -> Result<(), DeviceError> {
        // Use speaker-test for a quick beep, or aplay with a wav file
        let _ = Command::new("speaker-test")
            .args(["-t", "sine", "-f", "440", "-l", "1", "-p", "100"])
            .output();
        Ok(())
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new(AudioConfig::default()).unwrap_or_else(|_| Self {
            config: AudioConfig::default(),
            previous_volume: 70,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.volume, 70);
        assert_eq!(config.sample_rate, 48000);
        assert!(!config.muted);
    }

    #[test]
    fn test_volume_clamping() {
        let mut manager = AudioManager::default();
        manager.config.volume = 100;
        let _ = manager.volume_up(20);
        assert_eq!(manager.config.volume, 100);
    }
}
