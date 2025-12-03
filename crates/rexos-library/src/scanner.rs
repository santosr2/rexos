//! ROM scanning functionality

use crate::{Game, LibraryError};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Result of a ROM scan
#[derive(Debug, Default)]
pub struct ScanResult {
    pub games_found: usize,
    pub games_added: usize,
    pub games_updated: usize,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// ROM scanner configuration
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// File extensions to scan
    pub extensions: HashSet<String>,

    /// Directories to skip
    pub skip_dirs: HashSet<String>,

    /// Scan subdirectories
    pub recursive: bool,

    /// Skip hidden files/directories
    pub skip_hidden: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        let mut extensions = HashSet::new();
        // Common ROM extensions
        for ext in &[
            "nes", "fds", "smc", "sfc", "n64", "z64", "v64", "gb", "gbc", "gba", "nds", "sms",
            "gg", "md", "gen", "bin", "32x", "pce", "sgx", "iso", "cso", "chd", "pbp", "cue",
            "a26", "a78", "lnx", "ngp", "ngc", "ws", "wsc", "zip", "7z",
        ] {
            extensions.insert(ext.to_string());
        }

        let mut skip_dirs = HashSet::new();
        skip_dirs.insert("bios".to_string());
        skip_dirs.insert("saves".to_string());
        skip_dirs.insert("states".to_string());
        skip_dirs.insert("screenshots".to_string());
        skip_dirs.insert(".rexos".to_string());

        Self {
            extensions,
            skip_dirs,
            recursive: true,
            skip_hidden: true,
        }
    }
}

/// ROM scanner
pub struct RomScanner {
    config: ScanConfig,
}

impl Default for RomScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl RomScanner {
    /// Create a new scanner with default config
    pub fn new() -> Self {
        Self {
            config: ScanConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: ScanConfig) -> Self {
        Self { config }
    }

    /// Scan a directory for ROMs
    pub fn scan(&self, path: &Path, system: &str) -> Result<Vec<Game>, LibraryError> {
        let mut games = Vec::new();
        self.scan_dir(path, system, &mut games)?;
        Ok(games)
    }

    /// Recursively scan a directory
    fn scan_dir(
        &self,
        path: &Path,
        system: &str,
        games: &mut Vec<Game>,
    ) -> Result<(), LibraryError> {
        if !path.exists() || !path.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files/directories
            if self.config.skip_hidden && name.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                // Skip configured directories
                if self.config.skip_dirs.contains(&name.to_lowercase()) {
                    continue;
                }

                // Recurse into subdirectories
                if self.config.recursive {
                    self.scan_dir(&entry_path, system, games)?;
                }
            } else if entry_path.is_file() {
                // Check extension
                if let Some(ext) = entry_path.extension().and_then(|e| e.to_str())
                    && self.config.extensions.contains(&ext.to_lowercase())
                    && let Some(game) = self.create_game(&entry_path, system)
                {
                    games.push(game);
                }
            }
        }

        Ok(())
    }

    /// Create a Game from a ROM file
    fn create_game(&self, path: &Path, system: &str) -> Option<Game> {
        let name = path.file_stem()?.to_string_lossy().to_string();

        // Clean up name (remove region codes, etc.)
        let clean_name = Self::clean_game_name(&name);

        Some(Game {
            id: 0,
            path: path.to_string_lossy().to_string(),
            system: system.to_string(),
            name: clean_name,
            description: None,
            release_date: None,
            developer: None,
            publisher: None,
            genre: None,
            players: None,
            rating: None,
            favorite: false,
            hidden: false,
        })
    }

    /// Clean up a game name (remove region codes, etc.)
    fn clean_game_name(name: &str) -> String {
        let mut clean = name.to_string();

        // Remove common patterns in parentheses/brackets
        let patterns = [
            // Regions
            "(USA)", "(Europe)", "(Japan)", "(World)", "(U)", "(E)", "(J)", "(W)", "(En)", "(Fr)",
            "(De)", "(Es)", "(It)", // Versions
            "(Rev 1)", "(Rev 2)", "(Rev A)", "(Rev B)", "(v1.0)", "(v1.1)", "(v1.2)",
            // Tags
            "(Unl)", "(Proto)", "(Beta)", "(Demo)", "(Sample)", "[!]", "[a]", "[b]", "[h]", "[o]",
            "[t]",
        ];

        for pattern in &patterns {
            clean = clean.replace(pattern, "");
        }

        // Remove anything in square brackets
        while let Some(start) = clean.find('[') {
            if let Some(end) = clean.find(']') {
                if end > start {
                    clean = format!("{}{}", &clean[..start], &clean[end + 1..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Clean up whitespace
        clean = clean
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        clean
    }

    /// Scan all systems in a roms directory
    pub fn scan_all(&self, roms_dir: &Path) -> Result<Vec<(String, Vec<Game>)>, LibraryError> {
        let mut results = Vec::new();

        if !roms_dir.exists() {
            return Ok(results);
        }

        for entry in fs::read_dir(roms_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let system = entry.file_name().to_string_lossy().to_string();

                // Skip special directories
                if self.config.skip_dirs.contains(&system.to_lowercase()) {
                    continue;
                }

                let games = self.scan(&path, &system)?;
                if !games.is_empty() {
                    results.push((system, games));
                }
            }
        }

        Ok(results)
    }

    /// Get file info (size, hash, etc.)
    pub fn get_file_info(path: &Path) -> Option<FileInfo> {
        let metadata = fs::metadata(path).ok()?;

        Some(FileInfo {
            size: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

/// File information
#[derive(Debug)]
pub struct FileInfo {
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_game_name() {
        assert_eq!(
            RomScanner::clean_game_name("Super Mario World (USA)"),
            "Super Mario World"
        );
        assert_eq!(RomScanner::clean_game_name("Zelda (Europe) [!]"), "Zelda");
        assert_eq!(
            RomScanner::clean_game_name("Pokemon Red (U) (Rev 1)"),
            "Pokemon Red"
        );
    }

    #[test]
    fn test_scan_config_default() {
        let config = ScanConfig::default();
        assert!(config.extensions.contains("gba"));
        assert!(config.extensions.contains("nes"));
        assert!(config.skip_dirs.contains("bios"));
    }
}
