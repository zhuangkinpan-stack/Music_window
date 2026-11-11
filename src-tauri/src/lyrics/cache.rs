use std::{
    fs,
    path::PathBuf,
};

use sha2::{Digest, Sha256};

use super::{LyricsError, LyricsResult, TrackQuery};

pub struct LyricsCache {
    root: PathBuf,
}

impl LyricsCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn load(&self, track: &TrackQuery) -> Option<LyricsResult> {
        let path = self.path_for(track);
        let bytes = fs::read(path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    pub fn save(&self, track: &TrackQuery, result: &LyricsResult) -> Result<(), LyricsError> {
        fs::create_dir_all(&self.root)
            .map_err(|error| LyricsError::Io(error.to_string()))?;
        let bytes = serde_json::to_vec(result)
            .map_err(|error| LyricsError::Io(error.to_string()))?;
        fs::write(self.path_for(track), bytes)
            .map_err(|error| LyricsError::Io(error.to_string()))
    }

    fn path_for(&self, track: &TrackQuery) -> PathBuf {
        self.root.join(format!("{}.json", cache_key(track)))
    }
}

fn cache_key(track: &TrackQuery) -> String {
    let metadata = format!(
        "{}\u{1f}{}\u{1f}{}\u{1f}{}",
        track.title.trim().to_lowercase(),
        track.artist.trim().to_lowercase(),
        track.album.trim().to_lowercase(),
        track.duration_ms / 1_000
    );
    let digest = Sha256::digest(metadata.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

