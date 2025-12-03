//! Game metadata handling

use serde::{Deserialize, Serialize};

/// Game metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<i32>,
    pub rating: Option<f32>,
    pub region: Option<String>,
    pub box_art_url: Option<String>,
    pub screenshot_url: Option<String>,
}

/// Metadata source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSource {
    /// Local gamelist.xml
    Local,
    /// ScreenScraper API
    ScreenScraper,
    /// TheGamesDB API
    TheGamesDb,
    /// Manual entry
    Manual,
}

impl GameMetadata {
    /// Create empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if metadata is mostly empty
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.developer.is_none()
            && self.rating.is_none()
    }

    /// Merge with another metadata (non-empty fields take precedence)
    pub fn merge(&mut self, other: &GameMetadata) {
        if self.name.is_none() {
            self.name = other.name.clone();
        }
        if self.description.is_none() {
            self.description = other.description.clone();
        }
        if self.release_date.is_none() {
            self.release_date = other.release_date.clone();
        }
        if self.developer.is_none() {
            self.developer = other.developer.clone();
        }
        if self.publisher.is_none() {
            self.publisher = other.publisher.clone();
        }
        if self.genre.is_none() {
            self.genre = other.genre.clone();
        }
        if self.players.is_none() {
            self.players = other.players;
        }
        if self.rating.is_none() {
            self.rating = other.rating;
        }
        if self.region.is_none() {
            self.region = other.region.clone();
        }
        if self.box_art_url.is_none() {
            self.box_art_url = other.box_art_url.clone();
        }
        if self.screenshot_url.is_none() {
            self.screenshot_url = other.screenshot_url.clone();
        }
    }
}

/// Parse gamelist.xml format (EmulationStation compatible)
#[allow(dead_code)]
pub fn parse_gamelist_xml(xml: &str) -> Vec<(String, GameMetadata)> {
    let mut games = Vec::new();

    // Simple XML parsing (production would use quick-xml or roxmltree)
    // This is a basic implementation for the structure

    let mut current_path = String::new();
    let mut current_metadata = GameMetadata::new();
    let mut in_game = false;

    for line in xml.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("<game>") {
            in_game = true;
            current_metadata = GameMetadata::new();
        } else if trimmed.starts_with("</game>") {
            if in_game && !current_path.is_empty() {
                games.push((current_path.clone(), current_metadata.clone()));
            }
            in_game = false;
            current_path.clear();
        } else if in_game {
            if let Some(value) = extract_xml_value(trimmed, "path") {
                current_path = value;
            } else if let Some(value) = extract_xml_value(trimmed, "name") {
                current_metadata.name = Some(value);
            } else if let Some(value) = extract_xml_value(trimmed, "desc") {
                current_metadata.description = Some(value);
            } else if let Some(value) = extract_xml_value(trimmed, "releasedate") {
                current_metadata.release_date = Some(value);
            } else if let Some(value) = extract_xml_value(trimmed, "developer") {
                current_metadata.developer = Some(value);
            } else if let Some(value) = extract_xml_value(trimmed, "publisher") {
                current_metadata.publisher = Some(value);
            } else if let Some(value) = extract_xml_value(trimmed, "genre") {
                current_metadata.genre = Some(value);
            } else if let Some(value) = extract_xml_value(trimmed, "players") {
                current_metadata.players = value.parse().ok();
            } else if let Some(value) = extract_xml_value(trimmed, "rating") {
                current_metadata.rating = value.parse().ok();
            } else if let Some(value) = extract_xml_value(trimmed, "image") {
                current_metadata.box_art_url = Some(value);
            }
        }
    }

    games
}

/// Extract value from simple XML tag
#[allow(dead_code)]
fn extract_xml_value(line: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    if line.starts_with(&open_tag) && line.ends_with(&close_tag) {
        let start = open_tag.len();
        let end = line.len() - close_tag.len();
        if end > start {
            return Some(line[start..end].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_merge() {
        let mut meta1 = GameMetadata::new();
        meta1.name = Some("Game 1".to_string());

        let mut meta2 = GameMetadata::new();
        meta2.name = Some("Game 2".to_string());
        meta2.developer = Some("Dev".to_string());

        meta1.merge(&meta2);

        assert_eq!(meta1.name, Some("Game 1".to_string())); // Original kept
        assert_eq!(meta1.developer, Some("Dev".to_string())); // Merged
    }

    #[test]
    fn test_extract_xml_value() {
        assert_eq!(
            extract_xml_value("<name>Test Game</name>", "name"),
            Some("Test Game".to_string())
        );
        assert_eq!(extract_xml_value("<other>value</other>", "name"), None);
    }

    #[test]
    fn test_parse_gamelist() {
        let xml = r#"
<gameList>
    <game>
        <path>./mario.gba</path>
        <name>Super Mario</name>
        <desc>A great game</desc>
    </game>
</gameList>
"#;

        let games = parse_gamelist_xml(xml);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].0, "./mario.gba");
        assert_eq!(games[0].1.name, Some("Super Mario".to_string()));
    }
}
