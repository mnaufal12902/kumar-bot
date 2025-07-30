use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiYoutubeResponse {
    pub items: Vec<ApiItem>,
}

#[derive(Debug, Deserialize)]
pub struct YoutubeErrorResponse {
    pub error: YoutubeError
}

#[derive(Debug, Deserialize)]
pub struct YoutubeError {
    pub code: i32,
    pub message: String,
    pub errors: Vec<ErrorMessage>,
    pub status: String //TODO: Should be ENUM
}

#[derive(Debug, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
    pub domain: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiItem {
    pub id: ApiId,
    pub snippet: Snippet,
}

#[derive(Debug, Deserialize)]
pub struct ApiId {
    #[serde(rename = "videoId")]
    pub video_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Snippet {
    pub title: String,

    #[serde(rename = "channelTitle")]
    pub channel_title: String,
}

#[derive(Debug)]
pub struct YoutubeSearchResult {
    pub title: String,
    pub artist: String,
    pub video_id: String,
    pub source: &'static str,
}

impl From<ApiItem> for YoutubeSearchResult {
    fn from(item: ApiItem) -> Self {
        Self {
            title: item.snippet.title,
            artist: item.snippet.channel_title,
            video_id: item.id.video_id,
            source: "youtube",
        }
    }
}
