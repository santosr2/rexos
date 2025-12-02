//! Emulator management service for RexOS
//!
//! Handles launching RetroArch cores and standalone emulators,
//! based on ArkOS emulator management patterns.

mod launcher;
mod retroarch;
mod standalone;

pub use launcher::{EmulatorLauncher, LaunchConfig, LaunchResult};
pub use retroarch::{RetroArchLauncher, CoreInfo};
pub use standalone::{StandaloneLauncher, EmulatorInfo};

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmulatorError {
    #[error("Core not found: {0}")]
    CoreNotFound(String),

    #[error("ROM not found: {0}")]
    RomNotFound(PathBuf),

    #[error("Launch failed: {0}")]
    LaunchFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Supported game systems
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystem {
    // Nintendo
    Nes,
    Snes,
    N64,
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
    Nds,

    // Sega
    MasterSystem,
    Genesis,
    SegaCd,
    Saturn,
    Dreamcast,
    GameGear,

    // Sony
    Psx,
    Psp,

    // Arcade
    Mame,
    FinalBurnNeo,

    // Computers
    Amiga,
    Dos,

    // Other
    Atari2600,
    Atari7800,
    Lynx,
    NeoGeo,
    NeoGeoPocket,
    PcEngine,
    WonderSwan,

    // Custom/Unknown
    Custom(String),
}

impl GameSystem {
    /// Get system from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "nes" | "fds" => Some(GameSystem::Nes),
            "smc" | "sfc" => Some(GameSystem::Snes),
            "n64" | "z64" | "v64" => Some(GameSystem::N64),
            "gb" => Some(GameSystem::GameBoy),
            "gbc" => Some(GameSystem::GameBoyColor),
            "gba" => Some(GameSystem::GameBoyAdvance),
            "nds" | "ds" => Some(GameSystem::Nds),
            "sms" => Some(GameSystem::MasterSystem),
            "md" | "gen" | "bin" => Some(GameSystem::Genesis),
            "iso" | "cue" | "chd" => None, // Ambiguous
            "cso" | "pbp" => Some(GameSystem::Psp),
            "gg" => Some(GameSystem::GameGear),
            "pce" => Some(GameSystem::PcEngine),
            "ws" | "wsc" => Some(GameSystem::WonderSwan),
            "ngp" | "ngc" => Some(GameSystem::NeoGeoPocket),
            "lnx" => Some(GameSystem::Lynx),
            "a26" => Some(GameSystem::Atari2600),
            "a78" => Some(GameSystem::Atari7800),
            _ => None,
        }
    }

    /// Get system short name (for directory paths)
    pub fn short_name(&self) -> &str {
        match self {
            GameSystem::Nes => "nes",
            GameSystem::Snes => "snes",
            GameSystem::N64 => "n64",
            GameSystem::GameBoy => "gb",
            GameSystem::GameBoyColor => "gbc",
            GameSystem::GameBoyAdvance => "gba",
            GameSystem::Nds => "nds",
            GameSystem::MasterSystem => "sms",
            GameSystem::Genesis => "genesis",
            GameSystem::SegaCd => "segacd",
            GameSystem::Saturn => "saturn",
            GameSystem::Dreamcast => "dreamcast",
            GameSystem::GameGear => "gg",
            GameSystem::Psx => "psx",
            GameSystem::Psp => "psp",
            GameSystem::Mame => "mame",
            GameSystem::FinalBurnNeo => "fbneo",
            GameSystem::Amiga => "amiga",
            GameSystem::Dos => "dos",
            GameSystem::Atari2600 => "atari2600",
            GameSystem::Atari7800 => "atari7800",
            GameSystem::Lynx => "lynx",
            GameSystem::NeoGeo => "neogeo",
            GameSystem::NeoGeoPocket => "ngp",
            GameSystem::PcEngine => "pce",
            GameSystem::WonderSwan => "wonderswan",
            GameSystem::Custom(name) => name,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            GameSystem::Nes => "Nintendo Entertainment System",
            GameSystem::Snes => "Super Nintendo",
            GameSystem::N64 => "Nintendo 64",
            GameSystem::GameBoy => "Game Boy",
            GameSystem::GameBoyColor => "Game Boy Color",
            GameSystem::GameBoyAdvance => "Game Boy Advance",
            GameSystem::Nds => "Nintendo DS",
            GameSystem::MasterSystem => "Sega Master System",
            GameSystem::Genesis => "Sega Genesis",
            GameSystem::SegaCd => "Sega CD",
            GameSystem::Saturn => "Sega Saturn",
            GameSystem::Dreamcast => "Sega Dreamcast",
            GameSystem::GameGear => "Sega Game Gear",
            GameSystem::Psx => "Sony PlayStation",
            GameSystem::Psp => "Sony PSP",
            GameSystem::Mame => "MAME",
            GameSystem::FinalBurnNeo => "Final Burn Neo",
            GameSystem::Amiga => "Commodore Amiga",
            GameSystem::Dos => "DOS",
            GameSystem::Atari2600 => "Atari 2600",
            GameSystem::Atari7800 => "Atari 7800",
            GameSystem::Lynx => "Atari Lynx",
            GameSystem::NeoGeo => "Neo Geo",
            GameSystem::NeoGeoPocket => "Neo Geo Pocket",
            GameSystem::PcEngine => "PC Engine / TurboGrafx-16",
            GameSystem::WonderSwan => "WonderSwan",
            GameSystem::Custom(name) => name,
        }
    }

    /// Get default RetroArch core for this system
    pub fn default_core(&self) -> &str {
        match self {
            GameSystem::Nes => "fceumm",
            GameSystem::Snes => "snes9x",
            GameSystem::N64 => "mupen64plus_next",
            GameSystem::GameBoy | GameSystem::GameBoyColor => "gambatte",
            GameSystem::GameBoyAdvance => "mgba",
            GameSystem::Nds => "desmume",
            GameSystem::MasterSystem | GameSystem::GameGear => "genesis_plus_gx",
            GameSystem::Genesis => "genesis_plus_gx",
            GameSystem::SegaCd => "genesis_plus_gx",
            GameSystem::Saturn => "beetle_saturn",
            GameSystem::Dreamcast => "flycast",
            GameSystem::Psx => "pcsx_rearmed",
            GameSystem::Psp => "ppsspp",
            GameSystem::Mame => "mame",
            GameSystem::FinalBurnNeo => "fbneo",
            GameSystem::Amiga => "puae",
            GameSystem::Dos => "dosbox_pure",
            GameSystem::Atari2600 => "stella",
            GameSystem::Atari7800 => "prosystem",
            GameSystem::Lynx => "handy",
            GameSystem::NeoGeo => "fbneo",
            GameSystem::NeoGeoPocket => "beetle_ngp",
            GameSystem::PcEngine => "beetle_pce_fast",
            GameSystem::WonderSwan => "beetle_wswan",
            GameSystem::Custom(_) => "auto",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_from_extension() {
        assert_eq!(GameSystem::from_extension("gba"), Some(GameSystem::GameBoyAdvance));
        assert_eq!(GameSystem::from_extension("nes"), Some(GameSystem::Nes));
        assert_eq!(GameSystem::from_extension("unknown"), None);
    }

    #[test]
    fn test_system_names() {
        assert_eq!(GameSystem::GameBoyAdvance.short_name(), "gba");
        assert_eq!(GameSystem::GameBoyAdvance.default_core(), "mgba");
    }
}
