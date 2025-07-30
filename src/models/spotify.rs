use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SpotifyErrorResponse {
    pub error: SpotifyError,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyError {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SpotifyTrackItem {
    pub id: String,
    pub name: String,
    pub duration_ms: u32,
    pub explicit: bool,
    pub popularity: u32,
    pub preview_url: Option<String>,
    #[serde(rename = "type")]
    pub r#type: String,
    pub external_urls: ExternalUrls,
    pub artists: Vec<Artist>,
    pub album: Album,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub release_date: String,
    pub total_tracks: u32,
    #[serde(rename = "type")]
    pub r#type: String,
    pub external_urls: ExternalUrls,
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub external_urls: ExternalUrls,
}

#[derive(Debug, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}
