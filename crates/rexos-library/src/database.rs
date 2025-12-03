//! Game database using SQLite

use crate::LibraryError;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

/// A game in the library
#[derive(Debug, Clone)]
pub struct Game {
    pub id: i64,
    pub path: String,
    pub system: String,
    pub name: String,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<i32>,
    pub rating: Option<f32>,
    pub favorite: bool,
    pub hidden: bool,
}

/// Game statistics
#[derive(Debug, Clone, Default)]
pub struct GameStats {
    pub last_played: Option<String>,
    pub play_count: i32,
    pub play_time_seconds: i64,
}

/// Game database manager
pub struct GameDatabase {
    conn: Connection,
}

impl GameDatabase {
    /// Open or create a database
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LibraryError> {
        let conn = Connection::open(path)?;

        let db = Self { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// Create an in-memory database (for testing)
    pub fn in_memory() -> Result<Self, LibraryError> {
        let conn = Connection::open_in_memory()?;

        let db = Self { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<(), LibraryError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                system TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                release_date TEXT,
                developer TEXT,
                publisher TEXT,
                genre TEXT,
                players INTEGER,
                rating REAL,
                favorite INTEGER DEFAULT 0,
                hidden INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS game_stats (
                game_id INTEGER PRIMARY KEY,
                last_played TEXT,
                play_count INTEGER DEFAULT 0,
                play_time_seconds INTEGER DEFAULT 0,
                FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS collection_games (
                collection_id INTEGER,
                game_id INTEGER,
                added_at TEXT DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (collection_id, game_id),
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_games_system ON games(system);
            CREATE INDEX IF NOT EXISTS idx_games_name ON games(name);
            CREATE INDEX IF NOT EXISTS idx_games_favorite ON games(favorite);
            CREATE INDEX IF NOT EXISTS idx_game_stats_last_played ON game_stats(last_played);
        "#,
        )?;

        Ok(())
    }

    /// Add a game to the database
    pub fn add_game(&self, game: &Game) -> Result<i64, LibraryError> {
        self.conn.execute(
            r#"INSERT OR REPLACE INTO games
               (path, system, name, description, release_date, developer,
                publisher, genre, players, rating, favorite, hidden, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, CURRENT_TIMESTAMP)"#,
            params![
                game.path,
                game.system,
                game.name,
                game.description,
                game.release_date,
                game.developer,
                game.publisher,
                game.genre,
                game.players,
                game.rating,
                game.favorite,
                game.hidden,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get a game by ID
    pub fn get_game(&self, id: i64) -> Result<Option<Game>, LibraryError> {
        let game = self
            .conn
            .query_row("SELECT * FROM games WHERE id = ?1", params![id], |row| {
                Self::row_to_game(row)
            })
            .optional()?;

        Ok(game)
    }

    /// Get a game by path
    pub fn get_game_by_path(&self, path: &str) -> Result<Option<Game>, LibraryError> {
        let game = self
            .conn
            .query_row(
                "SELECT * FROM games WHERE path = ?1",
                params![path],
                Self::row_to_game,
            )
            .optional()?;

        Ok(game)
    }

    /// Get all games
    pub fn get_all_games(&self) -> Result<Vec<Game>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM games WHERE hidden = 0 ORDER BY name")?;

        let games = stmt
            .query_map([], Self::row_to_game)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(games)
    }

    /// Get games by system
    pub fn get_games_by_system(&self, system: &str) -> Result<Vec<Game>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM games WHERE system = ?1 AND hidden = 0 ORDER BY name")?;

        let games = stmt
            .query_map(params![system], Self::row_to_game)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(games)
    }

    /// Get favorite games
    pub fn get_favorites(&self) -> Result<Vec<Game>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM games WHERE favorite = 1 AND hidden = 0 ORDER BY name")?;

        let games = stmt
            .query_map([], Self::row_to_game)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(games)
    }

    /// Get recently played games
    pub fn get_recently_played(&self, limit: usize) -> Result<Vec<Game>, LibraryError> {
        let mut stmt = self.conn.prepare(
            r#"SELECT g.* FROM games g
               JOIN game_stats s ON g.id = s.game_id
               WHERE g.hidden = 0 AND s.last_played IS NOT NULL
               ORDER BY s.last_played DESC
               LIMIT ?1"#,
        )?;

        let games = stmt
            .query_map(params![limit as i64], Self::row_to_game)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(games)
    }

    /// Search games by name
    pub fn search_games(&self, query: &str) -> Result<Vec<Game>, LibraryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM games WHERE name LIKE ?1 AND hidden = 0 ORDER BY name")?;

        let pattern = format!("%{}%", query);
        let games = stmt
            .query_map(params![pattern], Self::row_to_game)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(games)
    }

    /// Set game as favorite
    pub fn set_favorite(&self, id: i64, favorite: bool) -> Result<(), LibraryError> {
        self.conn.execute(
            "UPDATE games SET favorite = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![favorite, id],
        )?;
        Ok(())
    }

    /// Set game as hidden
    pub fn set_hidden(&self, id: i64, hidden: bool) -> Result<(), LibraryError> {
        self.conn.execute(
            "UPDATE games SET hidden = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![hidden, id],
        )?;
        Ok(())
    }

    /// Delete a game
    pub fn delete_game(&self, id: i64) -> Result<(), LibraryError> {
        self.conn
            .execute("DELETE FROM games WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Update game stats (when played)
    pub fn update_play_stats(&self, game_id: i64, play_time: i64) -> Result<(), LibraryError> {
        self.conn.execute(
            r#"INSERT INTO game_stats (game_id, last_played, play_count, play_time_seconds)
               VALUES (?1, CURRENT_TIMESTAMP, 1, ?2)
               ON CONFLICT(game_id) DO UPDATE SET
                   last_played = CURRENT_TIMESTAMP,
                   play_count = play_count + 1,
                   play_time_seconds = play_time_seconds + ?2"#,
            params![game_id, play_time],
        )?;
        Ok(())
    }

    /// Get game stats
    pub fn get_stats(&self, game_id: i64) -> Result<GameStats, LibraryError> {
        let stats = self.conn.query_row(
            "SELECT last_played, play_count, play_time_seconds FROM game_stats WHERE game_id = ?1",
            params![game_id],
            |row| Ok(GameStats {
                last_played: row.get(0)?,
                play_count: row.get(1)?,
                play_time_seconds: row.get(2)?,
            }),
        ).optional()?.unwrap_or_default();

        Ok(stats)
    }

    /// Get total game count
    pub fn game_count(&self) -> Result<i64, LibraryError> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM games WHERE hidden = 0", [], |row| {
                    row.get(0)
                })?;
        Ok(count)
    }

    /// Get game count by system
    pub fn game_count_by_system(&self, system: &str) -> Result<i64, LibraryError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM games WHERE system = ?1 AND hidden = 0",
            params![system],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get list of systems with game counts
    pub fn get_systems(&self) -> Result<Vec<(String, i64)>, LibraryError> {
        let mut stmt = self.conn.prepare(
            "SELECT system, COUNT(*) FROM games WHERE hidden = 0 GROUP BY system ORDER BY system",
        )?;

        let systems = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(systems)
    }

    /// Convert a row to a Game
    fn row_to_game(row: &rusqlite::Row) -> rusqlite::Result<Game> {
        Ok(Game {
            id: row.get("id")?,
            path: row.get("path")?,
            system: row.get("system")?,
            name: row.get("name")?,
            description: row.get("description")?,
            release_date: row.get("release_date")?,
            developer: row.get("developer")?,
            publisher: row.get("publisher")?,
            genre: row.get("genre")?,
            players: row.get("players")?,
            rating: row.get("rating")?,
            favorite: row.get("favorite")?,
            hidden: row.get("hidden")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = GameDatabase::in_memory().unwrap();
        assert_eq!(db.game_count().unwrap(), 0);
    }

    #[test]
    fn test_add_and_get_game() {
        let db = GameDatabase::in_memory().unwrap();

        let game = Game {
            id: 0,
            path: "/roms/gba/test.gba".to_string(),
            system: "gba".to_string(),
            name: "Test Game".to_string(),
            description: None,
            release_date: None,
            developer: None,
            publisher: None,
            genre: None,
            players: None,
            rating: None,
            favorite: false,
            hidden: false,
        };

        let id = db.add_game(&game).unwrap();
        let retrieved = db.get_game(id).unwrap().unwrap();

        assert_eq!(retrieved.name, "Test Game");
        assert_eq!(retrieved.system, "gba");
    }

    #[test]
    fn test_search_games() {
        let db = GameDatabase::in_memory().unwrap();

        let game = Game {
            id: 0,
            path: "/roms/gba/mario.gba".to_string(),
            system: "gba".to_string(),
            name: "Super Mario Advance".to_string(),
            description: None,
            release_date: None,
            developer: None,
            publisher: None,
            genre: None,
            players: None,
            rating: None,
            favorite: false,
            hidden: false,
        };

        db.add_game(&game).unwrap();

        let results = db.search_games("mario").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].name.contains("Mario"));
    }
}
