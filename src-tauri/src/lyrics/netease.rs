use std::time::Duration;

use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;

use super::{
    artists_match, normalize_text, title_match_score, LyricsCandidate, LyricsError, LyricsFetcher,
    TrackQuery,
};

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct NeteaseSearchResponse {
    pub result: Option<NeteaseSearchResult>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct NeteaseSearchResult {
    #[serde(default)]
    pub songs: Vec<NeteaseSong>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct NeteaseSong {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub duration: i64,
    #[serde(default)]
    pub artists: Vec<NeteaseArtist>,
    pub album: Option<NeteaseAlbum>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct NeteaseArtist {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct NeteaseAlbum {
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct NeteaseLyricResponse {
    lrc: Option<NeteaseLyricText>,
    tlyric: Option<NeteaseLyricText>,
}

#[derive(Debug, Deserialize)]
struct NeteaseLyricText {
    #[serde(default)]
    lyric: String,
}

pub(crate) fn netease_search_url(title: &str, artist: &str) -> reqwest::Url {
    let mut url = reqwest::Url::parse("https://music.163.com/api/search/get/web")
        .expect("valid Netease search URL");
    url.query_pairs_mut()
        .append_pair("s", &format!("{} {}", title.trim(), artist.trim()))
        .append_pair("type", "1")
        .append_pair("limit", "10");
    url
}

pub(crate) fn netease_lyric_url(song_id: &str) -> reqwest::Url {
    reqwest::Url::parse(&format!(
        "https://music.163.com/api/song/lyric?id={song_id}&lv=-1&kv=-1&tv=-1"
    ))
    .expect("valid Netease lyric URL")
}

pub(crate) fn select_netease_song<'a>(
    track: &TrackQuery,
    response: &'a NeteaseSearchResponse,
) -> Option<&'a NeteaseSong> {
    let artist = normalize_text(&track.artist);
    response
        .result
        .as_ref()?
        .songs
        .iter()
        .filter_map(|song| {
            let song_artist = song
                .artists
                .iter()
                .map(|artist| artist.name.as_str())
                .collect::<Vec<_>>()
                .join("/");
            let score =
                title_match_score(&normalize_text(&track.title), &normalize_text(&song.name))?;
            artists_match(&artist, &normalize_text(&song_artist)).then_some((score, song))
        })
        .min_by_key(|(score, song)| (*score, (song.duration - track.duration_ms).abs()))
        .map(|(_, song)| song)
}

pub struct NeteaseLyricsFetcher {
    client: reqwest::Client,
}

impl NeteaseLyricsFetcher {
    pub fn new() -> Result<Self, LyricsError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .user_agent("Lumi/0.2 (Windows desktop music controller)")
            .build()
            .map_err(|error| LyricsError::Network(error.to_string()))?;
        Ok(Self { client })
    }

    async fn request_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: reqwest::Url,
    ) -> Result<Option<T>, LyricsError> {
        let response = self
            .client
            .get(url)
            .header("Referer", "https://music.163.com/")
            .send()
            .await
            .map_err(|error| LyricsError::Network(error.to_string()))?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            return Err(LyricsError::Http(response.status().as_u16()));
        }
        response
            .json()
            .await
            .map(Some)
            .map_err(|error| LyricsError::Parse(error.to_string()))
    }
}

#[async_trait]
impl LyricsFetcher for NeteaseLyricsFetcher {
    async fn lookup(&self, track: &TrackQuery) -> Result<Vec<LyricsCandidate>, LyricsError> {
        let Some(search) = self
            .request_json::<NeteaseSearchResponse>(netease_search_url(&track.title, &track.artist))
            .await?
        else {
            return Ok(Vec::new());
        };
        let Some(song) = select_netease_song(track, &search).cloned() else {
            return Ok(Vec::new());
        };
        let Some(lyrics) = self
            .request_json::<NeteaseLyricResponse>(netease_lyric_url(&song.id.to_string()))
            .await?
        else {
            return Ok(Vec::new());
        };
        let synced = lyrics
            .lrc
            .map(|value| value.lyric)
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                lyrics
                    .tlyric
                    .map(|value| value.lyric)
                    .filter(|value| !value.trim().is_empty())
            });
        if synced.is_none() {
            return Ok(Vec::new());
        }
        Ok(vec![LyricsCandidate {
            track_name: song.name,
            artist_name: song
                .artists
                .iter()
                .map(|artist| artist.name.clone())
                .collect::<Vec<_>>()
                .join("/"),
            album_name: song.album.map(|album| album.name).unwrap_or_default(),
            duration_ms: song.duration,
            synced_lyrics: synced,
        }])
    }
}
