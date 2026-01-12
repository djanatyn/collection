use serde::{Deserialize, Serialize};

// Steam API response structure
#[derive(Debug, Clone, Deserialize)]
pub struct SteamLibraryResponse {
    pub response: SteamGamesResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SteamGamesResponse {
    pub game_count: u32,
    pub games: Vec<SteamGame>,
}

// Individual game from Steam API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SteamGame {
    pub appid: u64,
    pub name: String,
    pub playtime_forever: u64, // Minutes
    #[serde(default)]
    pub img_icon_url: String,
    pub rtime_last_played: u64, // Unix timestamp
    #[serde(default)]
    pub playtime_linux_forever: u64,
    #[serde(default)]
    pub playtime_deck_forever: u64,
}

impl SteamGame {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    pub fn playtime_hours(&self) -> f64 {
        self.playtime_forever as f64 / 60.0
    }

    pub fn last_played_date(&self) -> String {
        if self.rtime_last_played == 0 {
            "Never".to_string()
        } else {
            // Convert Unix timestamp to readable date
            use chrono::{DateTime, Utc};
            let datetime = DateTime::<Utc>::from_timestamp(self.rtime_last_played as i64, 0);
            match datetime {
                Some(dt) => dt.format("%Y-%m-%d").to_string(),
                None => "Unknown".to_string(),
            }
        }
    }
}

pub type GameLibrary = Vec<SteamGame>;
