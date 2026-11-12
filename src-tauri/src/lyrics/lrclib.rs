use std::time::Duration;

use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Deserialize;

use super::{LyricsCandidate, LyricsError, LyricsFetcher, TrackQuery};

pub struct LrclibFetcher {
    client: reqwest::Client,
    base_url: String,
}

impl LrclibFetcher {
    pub fn new() -> Result<Self, LyricsError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .user_agent("Lumi/0.2 (Windows desktop music controller)")
            .build()
            .map_err(|error| LyricsError::Network(error.to_string()))?;
        Ok(Self {
            client,
            base_url: "https://lrclib.net".to_owned(),
        })
    }

    async fn request_json<T: for<'de> Deserialize<'de>>(
        &self,
        url: reqwest::Url,
    ) -> Result<Option<T>, LyricsError> {
        let response = self
            .client
            .get(url)
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

    fn exact_url(&self, track: &TrackQuery) -> Result<reqwest::Url, LyricsError> {
        let mut url = reqwest::Url::parse(&format!("{}/api/get", self.base_url))
            .map_err(|error| LyricsError::Parse(error.to_string()))?;
        let mut query = url.query_pairs_mut();
        query.append_pair("artist_name", &track.artist);
        query.append_pair("track_name", &track.title);
        if !track.album.trim().is_empty() {
            query.append_pair("album_name", &track.album);
        }
        if track.duration_ms > 0 {
            query.append_pair("duration", &(track.duration_ms / 1_000).to_string());
        }
        drop(query);
        Ok(url)
    }

    fn search_url(&self, track: &TrackQuery) -> Result<reqwest::Url, LyricsError> {
        let mut url = reqwest::Url::parse(&format!("{}/api/search", self.base_url))
            .map_err(|error| LyricsError::Parse(error.to_string()))?;
        let mut query = url.query_pairs_mut();
        query.append_pair("artist_name", &track.artist);
        query.append_pair("track_name", &track.title);
        drop(query);
        Ok(url)
    }

    fn q_search_url(&self, track: &TrackQuery) -> Result<reqwest::Url, LyricsError> {
        let mut url = reqwest::Url::parse(&format!("{}/api/search", self.base_url))
            .map_err(|error| LyricsError::Parse(error.to_string()))?;
        url.query_pairs_mut().append_pair(
            "q",
            &format!("{} {}", track.title.trim(), track.artist.trim()),
        );
        Ok(url)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibRecord {
    track_name: String,
    artist_name: String,
    album_name: Option<String>,
    duration: Option<f64>,
    synced_lyrics: Option<String>,
    #[serde(default)]
    instrumental: bool,
}

impl From<LrclibRecord> for LyricsCandidate {
    fn from(record: LrclibRecord) -> Self {
        Self {
            track_name: record.track_name,
            artist_name: record.artist_name,
            album_name: record.album_name.unwrap_or_default(),
            duration_ms: record.duration.unwrap_or_default().round() as i64 * 1_000,
            synced_lyrics: (!record.instrumental)
                .then_some(record.synced_lyrics)
                .flatten(),
        }
    }
}

#[async_trait]
impl LyricsFetcher for LrclibFetcher {
    async fn lookup(&self, track: &TrackQuery) -> Result<Vec<LyricsCandidate>, LyricsError> {
        let mut candidates: Vec<LyricsCandidate> = Vec::new();
        if let Some(record) = self
            .request_json::<LrclibRecord>(self.exact_url(track)?)
            .await?
        {
            candidates.push(record.into());
        }

        if let Some(records) = self
            .request_json::<Vec<LrclibRecord>>(self.search_url(track)?)
            .await?
        {
            candidates.extend(records.into_iter().map(Into::into));
        }
        if let Some(records) = self
            .request_json::<Vec<LrclibRecord>>(self.q_search_url(track)?)
            .await?
        {
            candidates.extend(records.into_iter().map(Into::into));
        }
        candidates.sort_by(|left, right| {
            (&left.track_name, &left.artist_name, left.duration_ms).cmp(&(
                &right.track_name,
                &right.artist_name,
                right.duration_ms,
            ))
        });
        candidates.dedup_by(|left, right| {
            left.track_name == right.track_name
                && left.artist_name == right.artist_name
                && left.duration_ms == right.duration_ms
        });
        Ok(candidates)
    }
}

impl Default for LrclibFetcher {
    fn default() -> Self {
        Self::new().expect("LRCLIB client configuration is valid")
    }
}
