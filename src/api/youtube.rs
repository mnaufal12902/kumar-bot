use std::env;

use anyhow::Ok;
use reqwest::Client;

use crate::models::youtube::{ApiYoutubeResponse, YoutubeErrorResponse, YoutubeSearchResult};

pub async fn search_youtube(query: &str) -> Result<Vec<YoutubeSearchResult>, anyhow::Error> {
    let client = Client::new();
    let api_key = env::var("YOUTUBE_API_KEY").expect("Missing Youtube API key, please configure your env");

    let url = "https://www.googleapis.com/youtube/v3/search";

    let res = client
        .get(url)
        .query(&[
            ("part", "snippet"),
            ("maxResults", "5"),
            ("q", query),
            ("type", "video"),
            ("key", &api_key),
        ])
        .send()
        .await?;

    if res.status().is_success() {
        let results = res.json::<ApiYoutubeResponse>().await?;
        let items = results.items.into_iter().map(YoutubeSearchResult::from).collect();
        Ok(items)
    } else {
        let err = res.json::<YoutubeErrorResponse>().await?;
        Err(anyhow::anyhow!("{} {}", err.error.code, err.error.message))
    }
}