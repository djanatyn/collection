use crate::library::{Album, Artist, Library};
use crate::track::Track;
use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub struct Parser {
    artists: HashMap<String, Artist>,
    albums: HashMap<String, Album>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            artists: HashMap::new(),
            albums: HashMap::new(),
        }
    }

    pub async fn parse_file(&mut self, file_path: &str) -> Result<Library> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let tracks: Vec<Track> = serde_json::from_reader(reader)?;

        println!("Parsing {} tracks...", tracks.len());

        for track in tracks {
            if track.is_empty() {
                continue;
            }

            self.process_track(track);
        }

        Ok(self.artists.clone())
    }

    fn process_track(&mut self, track: Track) {
        let artist_name = if track.albumartist.is_empty() {
            &track.artist
        } else {
            &track.albumartist
        };

        if artist_name.is_empty() {
            return;
        }

        let artist = self
            .artists
            .entry(artist_name.clone())
            .or_insert_with(|| Artist::new(artist_name.clone()));

        if track.has_album() {
            let album_key = format!("{}-{}", track.album_id, track.album);
            let album = self.albums.entry(album_key.clone()).or_insert_with(|| {
                let mut album = Album::new(
                    track.album_id.clone(),
                    track.album.clone(),
                    artist_name.clone(),
                );
                album.year = track.year.clone();
                album.genre = track.genre.clone();
                if let Ok(total) = track.tracktotal.parse::<u32>() {
                    album.tracktotal = total;
                }
                album
            });

            album.add_track(track.clone());

            // Always update the artist's album reference to get latest track list
            if let Some(artist_album) = artist.albums.iter_mut().find(|a| a.id == track.album_id) {
                *artist_album = album.clone();
            } else {
                artist.add_album(album.clone());
            }
        } else {
            artist.add_track(track.clone());
        }
    }
}
