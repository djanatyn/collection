use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub albumartist: String,
    pub year: String,
    pub genre: String,
    pub length: String,
    pub track: String,
    pub tracktotal: String,
    pub disc: String,
    pub disctotal: String,
    pub bitrate: String,
    pub format: String,
    pub path: String,
    pub added: String,
    pub comments: String,
    pub bpm: String,
    pub composer: String,
    pub label: String,
    pub country: String,
    pub albumtype: String,
    pub mb_trackid: String,
    pub mb_albumid: String,
    pub mb_artistid: String,
    pub album_id: String,
    // Add more fields as needed
}

impl Track {
    pub fn is_empty(&self) -> bool {
        self.title.is_empty() && self.artist.is_empty()
    }

    pub fn has_album(&self) -> bool {
        !self.album.is_empty()
    }

    pub fn track_number(&self) -> u32 {
        self.track.parse().unwrap_or(0)
    }
}
