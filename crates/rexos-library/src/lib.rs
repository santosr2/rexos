//! Game library management service for RexOS
//!
//! Handles ROM scanning, metadata storage, and game collection management.
//! Based on ArkOS library patterns with SQLite storage.

mod database;
mod metadata;
mod scanner;

pub use database::{Game, GameDatabase, GameStats};
pub use metadata::{GameMetadata, MetadataSource};
pub use scanner::{RomScanner, ScanResult};

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Scan error: {0}")]
    ScanError(String),

    #[error("Game not found: {0}")]
    GameNotFound(i64),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Collection types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Collection {
    /// All games
    All,
    /// Favorites
    Favorites,
    /// Recently played
    RecentlyPlayed,
    /// By system
    System(String),
    /// Custom collection
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_variants() {
        let col = Collection::System("gba".to_string());
        assert!(matches!(col, Collection::System(_)));
    }
}
