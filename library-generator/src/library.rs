use crate::track::Track;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub year: String,
    pub tracktotal: u32,
    pub disctotal: u32,
    pub genre: String,
    pub tracks: Vec<Track>,
}

impl Album {
    pub fn new(id: String, title: String, artist: String) -> Self {
        Self {
            id,
            title,
            artist,
            year: String::new(),
            tracktotal: 0,
            disctotal: 1,
            genre: String::new(),
            tracks: Vec::new(),
        }
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.tracks
            .sort_by_key(|t| (t.disc.parse::<u32>().unwrap_or(1), t.track_number()));
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Artist {
    pub name: String,
    pub albums: Vec<Album>,
    pub tracks: Vec<Track>, // For tracks without albums
}

impl Artist {
    pub fn new(name: String) -> Self {
        Self {
            name,
            albums: Vec::new(),
            tracks: Vec::new(),
        }
    }

    pub fn add_album(&mut self, album: Album) {
        self.albums.push(album);
        self.albums
            .sort_by(|a, b| a.year.cmp(&b.year).then(a.title.cmp(&b.title)));
    }

    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.tracks.sort_by_key(|t| t.title.clone());
    }
}

pub type Library = HashMap<String, Artist>;
