//! Hotkey configuration
//!
//! Based on ArkOS hotkey patterns (Select + button combinations)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Actions that can be triggered by hotkeys
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    /// Exit current game/emulator
    Exit,
    /// Save state to current slot
    SaveState,
    /// Load state from current slot
    LoadState,
    /// Toggle fast forward
    FastForward,
    /// Rewind (if supported)
    Rewind,
    /// Take screenshot
    Screenshot,
    /// Toggle pause
    Pause,
    /// Open RetroArch menu
    Menu,
    /// Increase save state slot
    NextSlot,
    /// Decrease save state slot
    PrevSlot,
    /// Increase volume
    VolumeUp,
    /// Decrease volume
    VolumeDown,
    /// Increase brightness
    BrightnessUp,
    /// Decrease brightness
    BrightnessDown,
    /// Toggle FPS display
    ShowFps,
    /// Reset game
    Reset,
    /// Toggle turbo mode
    Turbo,
}

/// A hotkey definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    /// Modifier button (usually Select)
    pub modifier: String,
    /// Action button
    pub button: String,
}

impl Hotkey {
    pub fn new(modifier: &str, button: &str) -> Self {
        Self {
            modifier: modifier.to_string(),
            button: button.to_string(),
        }
    }

    /// Format as human-readable string
    pub fn to_string_pretty(&self) -> String {
        format!("{} + {}", self.modifier, self.button)
    }
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// The modifier button for all hotkeys
    #[serde(default = "default_modifier")]
    pub modifier: String,

    /// Enabled hotkeys
    #[serde(default = "default_hotkeys")]
    pub hotkeys: HashMap<HotkeyAction, String>,

    /// Enable hotkeys globally
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_modifier() -> String {
    "Select".to_string()
}

fn default_true() -> bool {
    true
}

fn default_hotkeys() -> HashMap<HotkeyAction, String> {
    let mut map = HashMap::new();
    // Default ArkOS-style hotkeys
    map.insert(HotkeyAction::Exit, "Start".to_string());
    map.insert(HotkeyAction::SaveState, "R1".to_string());
    map.insert(HotkeyAction::LoadState, "L1".to_string());
    map.insert(HotkeyAction::FastForward, "R2".to_string());
    map.insert(HotkeyAction::Screenshot, "L2".to_string());
    map.insert(HotkeyAction::Menu, "X".to_string());
    map.insert(HotkeyAction::Reset, "B".to_string());
    map.insert(HotkeyAction::NextSlot, "Right".to_string());
    map.insert(HotkeyAction::PrevSlot, "Left".to_string());
    map.insert(HotkeyAction::VolumeUp, "Up".to_string());
    map.insert(HotkeyAction::VolumeDown, "Down".to_string());
    map
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifier: default_modifier(),
            hotkeys: default_hotkeys(),
            enabled: true,
        }
    }
}

impl HotkeyConfig {
    /// Get the hotkey for an action
    pub fn get_hotkey(&self, action: &HotkeyAction) -> Option<Hotkey> {
        self.hotkeys.get(action).map(|button| Hotkey {
            modifier: self.modifier.clone(),
            button: button.clone(),
        })
    }

    /// Set a hotkey for an action
    pub fn set_hotkey(&mut self, action: HotkeyAction, button: String) {
        self.hotkeys.insert(action, button);
    }

    /// Remove a hotkey
    pub fn remove_hotkey(&mut self, action: &HotkeyAction) {
        self.hotkeys.remove(action);
    }

    /// Get all configured hotkeys
    pub fn all_hotkeys(&self) -> Vec<(HotkeyAction, Hotkey)> {
        self.hotkeys
            .iter()
            .map(|(action, button)| {
                (
                    action.clone(),
                    Hotkey {
                        modifier: self.modifier.clone(),
                        button: button.clone(),
                    },
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_hotkeys() {
        let config = HotkeyConfig::default();
        assert!(config.enabled);
        assert_eq!(config.modifier, "Select");

        let exit = config.get_hotkey(&HotkeyAction::Exit);
        assert!(exit.is_some());
        assert_eq!(exit.unwrap().button, "Start");
    }

    #[test]
    fn test_hotkey_pretty_string() {
        let hotkey = Hotkey::new("Select", "Start");
        assert_eq!(hotkey.to_string_pretty(), "Select + Start");
    }
}
