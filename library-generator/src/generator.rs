use crate::game::{GameLibrary, SteamGame};
use crate::library::{Album, Artist, Library};
use crate::track::Track;
use anyhow::Result;
use serde::Serialize;
use slug::slugify;
use std::fs;
use tera::Tera;

// Context struct for track page template
#[derive(Serialize)]
struct TrackContext {
    title: String,
    template: String,
    track: String,
    artist: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<String>,
    format: String,
    bitrate: String,
    length: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comments: Option<String>,
    search_content: String,
    url: String,
}

// Context structs for artist page template
#[derive(Serialize)]
struct ArtistContext {
    title: String,
    template: String,
    artist: String,
    albums: Vec<AlbumSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tracks: Vec<TrackSummary>,
}

#[derive(Serialize)]
struct AlbumSummary {
    title: String,
    year: String,
    tracks: usize,
}

#[derive(Serialize)]
struct TrackSummary {
    title: String,
    length: String,
    year: String,
}

// Context structs for album page template
#[derive(Serialize)]
struct AlbumContext {
    title: String,
    template: String,
    album: String,
    artist: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    year: String,
    genre: String,
    tracktotal: u32,
    tracks: Vec<TrackInAlbum>,
}

#[derive(Serialize)]
struct TrackInAlbum {
    title: String,
    length: String,
}

// Context structs for index page template
#[derive(Serialize)]
struct IndexContext {
    title: String,
    sort_by: String,
    template: String,
    artist_count: usize,
    album_count: usize,
    track_count: usize,
    artists: Vec<ArtistLink>,
}

#[derive(Serialize)]
struct ArtistLink {
    name: String,
    slug: String,
}

// Context struct for game page template
#[derive(Serialize)]
struct GameContext {
    title: String,
    template: String,
    game: String,
    appid: u64,
    playtime_hours: String,
    last_played: String,
    search_content: String,
    url: String,
}

// Context structs for games index page template
#[derive(Serialize)]
struct GamesIndexContext {
    title: String,
    sort_by: String,
    template: String,
    game_count: usize,
    total_hours: String,
    games: Vec<GameLink>,
}

#[derive(Serialize)]
struct GameLink {
    name: String,
    slug: String,
    playtime_hours: String,
}

pub struct Generator {
    output_dir: String,
    tera: Tera,
}

// Custom filter for TOML string escaping
fn escape_toml_filter(
    value: &tera::Value,
    _args: &std::collections::HashMap<String, tera::Value>,
) -> tera::Result<tera::Value> {
    if let Some(s) = value.as_str() {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r");
        Ok(tera::Value::String(escaped))
    } else {
        Ok(value.clone())
    }
}

impl Generator {
    pub fn new(output_dir: String) -> Result<Self> {
        // Initialize Tera with templates
        let mut tera = Tera::new("content-templates/**/*.tera")
            .map_err(|e| anyhow::anyhow!("Failed to load templates: {}", e))?;

        // Register custom filter for TOML escaping
        tera.register_filter("escape_toml", escape_toml_filter);

        // Validate required templates exist
        let required = vec![
            "track.md.tera",
            "artist.md.tera",
            "album.md.tera",
            "index.md.tera",
            "game.md.tera",
            "games_index.md.tera",
        ];
        for template in required {
            if !tera.get_template_names().any(|n| n == template) {
                return Err(anyhow::anyhow!(
                    "Required template '{}' not found",
                    template
                ));
            }
        }

        Ok(Self { output_dir, tera })
    }

    pub async fn generate(&self, library: &Library) -> Result<()> {
        // Create output directories
        fs::create_dir_all(&self.output_dir)?;
        fs::create_dir_all(format!("{}/artists", self.output_dir))?;
        fs::create_dir_all(format!("{}/albums", self.output_dir))?;
        fs::create_dir_all(format!("{}/tracks", self.output_dir))?;

        // Generate index page
        self.generate_index(library).await?;

        // Generate section indexes
        self.generate_artists_section_index().await?;
        self.generate_albums_section_index().await?;
        self.generate_tracks_section_index().await?;

        // Generate artist pages
        for (artist_name, artist) in library {
            self.generate_artist_page(artist_name, artist).await?;
        }

        // Generate album pages
        for artist in library.values() {
            for album in &artist.albums {
                self.generate_album_page(album).await?;
            }
        }

        // Generate individual track pages
        for artist in library.values() {
            for album in &artist.albums {
                for track in &album.tracks {
                    self.generate_track_page(track).await?;
                }
            }
            for track in &artist.tracks {
                self.generate_track_page(track).await?;
            }
        }

        println!("Generated content in {}", self.output_dir);
        Ok(())
    }

    async fn generate_index(&self, library: &Library) -> Result<()> {
        // Calculate statistics
        let artist_count = library.len();
        let album_count: usize = library.values().map(|a| a.albums.len()).sum();
        let track_count: usize = library
            .values()
            .map(|a| a.albums.iter().map(|al| al.tracks.len()).sum::<usize>() + a.tracks.len())
            .sum();

        // Build sorted artist list
        let mut artist_names: Vec<&String> = library.keys().collect();
        artist_names.sort();
        let artists: Vec<ArtistLink> = artist_names
            .iter()
            .map(|&name| ArtistLink {
                name: name.clone(),
                slug: slugify(name),
            })
            .collect();

        // Create context
        let context = IndexContext {
            title: "Music Library".to_string(),
            sort_by: "title".to_string(),
            template: "index.html".to_string(),
            artist_count,
            album_count,
            track_count,
            artists,
        };

        // Render template
        let content = self
            .tera
            .render("index.md.tera", &tera::Context::from_serialize(&context)?)
            .map_err(|e| anyhow::anyhow!("Failed to render index: {}", e))?;

        // Write file
        let path = format!("{}/_index.md", self.output_dir);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;

        Ok(())
    }

    async fn generate_artist_page(&self, artist_name: &str, artist: &Artist) -> Result<()> {
        let slug = slugify(artist_name);

        // Build album summaries
        let albums: Vec<AlbumSummary> = artist
            .albums
            .iter()
            .map(|album| AlbumSummary {
                title: album.title.clone(),
                year: album.year.clone(),
                tracks: album.track_count(),
            })
            .collect();

        // Build standalone track summaries
        let tracks: Vec<TrackSummary> = artist
            .tracks
            .iter()
            .map(|track| TrackSummary {
                title: track.title.clone(),
                length: track.length.clone(),
                year: track.year.clone(),
            })
            .collect();

        // Create context
        let context = ArtistContext {
            title: artist_name.to_string(),
            template: "artist.html".to_string(),
            artist: artist_name.to_string(),
            albums,
            tracks,
        };

        // Render template
        let content = self
            .tera
            .render("artist.md.tera", &tera::Context::from_serialize(&context)?)
            .map_err(|e| anyhow::anyhow!("Failed to render artist '{}': {}", artist_name, e))?;

        // Write file
        let path = format!("{}/artists/{}.md", self.output_dir, slug);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;

        Ok(())
    }

    async fn generate_album_page(&self, album: &Album) -> Result<()> {
        let slug = slugify(&album.title);

        // Build track list
        let tracks: Vec<TrackInAlbum> = album
            .tracks
            .iter()
            .map(|track| TrackInAlbum {
                title: track.title.clone(),
                length: track.length.clone(),
            })
            .collect();

        // Create context
        let context = AlbumContext {
            title: album.title.clone(),
            template: "album.html".to_string(),
            album: album.title.clone(),
            artist: album.artist.clone(),
            year: album.year.clone(),
            genre: album.genre.clone(),
            tracktotal: album.tracktotal,
            tracks,
        };

        // Render template
        let content = self
            .tera
            .render("album.md.tera", &tera::Context::from_serialize(&context)?)
            .map_err(|e| anyhow::anyhow!("Failed to render album '{}': {}", album.title, e))?;

        // Write file
        let path = format!("{}/albums/{}.md", self.output_dir, slug);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;

        Ok(())
    }

    async fn generate_track_page(&self, track: &Track) -> Result<()> {
        let slug = slugify(&track.title);

        // Build search content
        let mut search_parts = Vec::new();
        search_parts.push(track.title.clone());
        search_parts.push(track.artist.clone());
        if track.has_album() {
            search_parts.push(track.album.clone());
        }
        if !track.genre.is_empty() {
            search_parts.push(track.genre.clone());
        }
        let mut search_content = search_parts.join(" ");
        if search_content.len() > 500 {
            search_content.truncate(500);
        }

        // Create context
        let context = TrackContext {
            title: track.title.clone(),
            template: "track.html".to_string(),
            track: track.title.clone(),
            artist: track.artist.clone(),
            album: if track.has_album() {
                Some(track.album.clone())
            } else {
                None
            },
            year: if !track.year.is_empty() {
                Some(track.year.clone())
            } else {
                None
            },
            format: track.format.clone(),
            bitrate: track.bitrate.clone(),
            length: track.length.clone(),
            genre: if !track.genre.is_empty() {
                Some(track.genre.clone())
            } else {
                None
            },
            comments: if !track.comments.is_empty() {
                Some(track.comments.replace('\n', " ").replace('\r', " "))
            } else {
                None
            },
            search_content,
            url: format!("/tracks/{}", slug),
        };

        // Render template
        let content = self
            .tera
            .render("track.md.tera", &tera::Context::from_serialize(&context)?)
            .map_err(|e| anyhow::anyhow!("Failed to render track '{}': {}", track.title, e))?;

        // Write file
        let path = format!("{}/tracks/{}.md", self.output_dir, slug);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;

        Ok(())
    }

    pub async fn generate_games(&self, library: &GameLibrary) -> Result<()> {
        // Create games directory
        fs::create_dir_all(format!("{}/games", self.output_dir))?;

        // Generate games section index
        self.generate_games_index(library).await?;

        // Generate individual game pages
        for game in library {
            if !game.is_empty() {
                self.generate_game_page(game).await?;
            }
        }

        println!("Generated games in {}/games", self.output_dir);
        Ok(())
    }

    async fn generate_games_index(&self, library: &GameLibrary) -> Result<()> {
        let game_count = library.len();
        let total_playtime: u64 = library.iter().map(|g| g.playtime_forever).sum();
        let total_hours = total_playtime as f64 / 60.0;

        // Sort games by playtime (descending)
        let mut sorted_games = library.clone();
        sorted_games.sort_by(|a, b| b.playtime_forever.cmp(&a.playtime_forever));

        let games: Vec<GameLink> = sorted_games
            .iter()
            .map(|game| GameLink {
                name: game.name.clone(),
                slug: slugify(&game.name),
                playtime_hours: format!("{:.1}h", game.playtime_hours()),
            })
            .collect();

        let context = GamesIndexContext {
            title: "Games".to_string(),
            sort_by: "title".to_string(),
            template: "games_index.html".to_string(),
            game_count,
            total_hours: format!("{:.1}", total_hours),
            games,
        };

        let content = self
            .tera
            .render(
                "games_index.md.tera",
                &tera::Context::from_serialize(&context)?,
            )
            .map_err(|e| anyhow::anyhow!("Failed to render games index: {}", e))?;

        let path = format!("{}/games/_index.md", self.output_dir);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;

        Ok(())
    }

    async fn generate_game_page(&self, game: &SteamGame) -> Result<()> {
        let slug = slugify(&game.name);

        // Build search content
        let search_content = game.name.clone();
        let playtime_hours = format!("{:.1}", game.playtime_hours());

        // Create context
        let context = GameContext {
            title: game.name.clone(),
            template: "game.html".to_string(),
            game: game.name.clone(),
            appid: game.appid,
            playtime_hours,
            last_played: game.last_played_date(),
            search_content,
            url: format!("/games/{}", slug),
        };

        // Render template
        let content = self
            .tera
            .render("game.md.tera", &tera::Context::from_serialize(&context)?)
            .map_err(|e| anyhow::anyhow!("Failed to render game '{}': {}", game.name, e))?;

        // Write file
        let path = format!("{}/games/{}.md", self.output_dir, slug);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;

        Ok(())
    }

    async fn generate_artists_section_index(&self) -> Result<()> {
        let content = r#"+++
title = "Artists"
sort_by = "title"
template = "section.html"
+++
"#;
        let path = format!("{}/artists/_index.md", self.output_dir);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;
        Ok(())
    }

    async fn generate_albums_section_index(&self) -> Result<()> {
        let content = r#"+++
title = "Albums"
sort_by = "title"
template = "section.html"
+++
"#;
        let path = format!("{}/albums/_index.md", self.output_dir);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;
        Ok(())
    }

    async fn generate_tracks_section_index(&self) -> Result<()> {
        let content = r#"+++
title = "Tracks"
sort_by = "title"
template = "section.html"
+++
"#;
        let path = format!("{}/tracks/_index.md", self.output_dir);
        fs::write(&path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file '{}': {}", path, e))?;
        Ok(())
    }
}
