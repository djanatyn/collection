use crate::game::{GameLibrary, SteamLibraryResponse};
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;

const STEAM_API_URL: &str = "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1";
const CACHE_FILE: &str = "steam-library.json";

pub struct SteamClient {
    api_key: String,
    steam_id: String,
}

impl SteamClient {
    pub fn new(api_key: String, steam_id: String) -> Self {
        Self { api_key, steam_id }
    }

    pub async fn fetch_library(&self) -> Result<GameLibrary> {
        // Check if cache exists
        if Path::new(CACHE_FILE).exists() {
            println!("Loading Steam library from cache: {}", CACHE_FILE);
            return self.load_from_cache();
        }

        println!("Fetching Steam library from API...");
        let library = self.fetch_from_api()?;

        // Save to cache
        self.save_to_cache(&library)?;

        Ok(library)
    }

    fn fetch_from_api(&self) -> Result<GameLibrary> {
        let url = format!(
            "{}?key={}&steamid={}&include_appinfo=true&include_played_free_games=false",
            STEAM_API_URL, self.api_key, self.steam_id
        );

        let response = ureq::get(&url).call()?;
        let steam_response: SteamLibraryResponse = response.into_json()?;

        println!(
            "Fetched {} games from Steam API",
            steam_response.response.game_count
        );

        Ok(steam_response.response.games)
    }

    fn load_from_cache(&self) -> Result<GameLibrary> {
        let file = File::open(CACHE_FILE)?;
        let reader = BufReader::new(file);
        let games: GameLibrary = serde_json::from_reader(reader)?;
        println!("Loaded {} games from cache", games.len());
        Ok(games)
    }

    fn save_to_cache(&self, library: &GameLibrary) -> Result<()> {
        let json = serde_json::to_string_pretty(library)?;
        let mut file = File::create(CACHE_FILE)?;
        file.write_all(json.as_bytes())?;
        println!("Saved Steam library to {}", CACHE_FILE);
        Ok(())
    }

    pub fn clear_cache() -> Result<()> {
        if Path::new(CACHE_FILE).exists() {
            std::fs::remove_file(CACHE_FILE)?;
            println!("Cleared Steam library cache");
        }
        Ok(())
    }
}
