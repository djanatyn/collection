mod game;
mod generator;
mod library;
mod parser;
mod steam;
mod track;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "library-generator")]
#[command(about = "Generate static site content from library exports")]
struct Cli {
    /// Path to the music JSON export file
    #[arg(short = 'm', long)]
    music_input: Option<PathBuf>,

    /// Fetch Steam library (requires STEAM_API_KEY and STEAM_ID env vars)
    #[arg(short = 's', long)]
    steam: bool,

    /// Clear Steam cache before fetching
    #[arg(long)]
    clear_steam_cache: bool,

    /// Output directory for generated content
    #[arg(short, long, default_value = "content")]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Library Generator");

    // Initialize generator
    let generator = generator::Generator::new(cli.output.to_str().unwrap().to_string())?;

    // Generate music library if input provided
    if let Some(music_path) = cli.music_input {
        println!("Music Input: {:?}", music_path);
        let mut parser = parser::Parser::new();
        let library = parser.parse_file(music_path.to_str().unwrap()).await?;
        generator.generate(&library).await?;
    }

    // Generate Steam library if requested
    if cli.steam {
        if cli.clear_steam_cache {
            steam::SteamClient::clear_cache()?;
        }

        let api_key =
            std::env::var("STEAM_API_KEY").expect("STEAM_API_KEY environment variable not set");
        let steam_id = std::env::var("STEAM_ID").expect("STEAM_ID environment variable not set");

        let client = steam::SteamClient::new(api_key, steam_id);
        let games = client.fetch_library().await?;
        generator.generate_games(&games).await?;
    }

    println!("Done!");
    Ok(())
}
