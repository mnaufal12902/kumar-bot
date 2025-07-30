use crate::models::spotify::{SpotifyErrorResponse, SpotifyTrackItem};
use crate::token::registry::TokenRegistry;
use anyhow::{Error, Result};
use reqwest::Client;

pub async fn get_track_by_id(
    track_id: String,
    registry: TokenRegistry,
) -> Result<SpotifyTrackItem, Error> {
    let token = {
        let mut guard = registry.spotify.lock().await;
        match guard.get_token().await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to get token: {:?}", e);
                return Err(anyhow::anyhow!(e.to_string()));
            }
        }
    };

    let url = format!("https://api.spotify.com/v1/tracks/{}", track_id);

    let client = Client::new();
    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if res.status().is_success() {
        let track = res.json::<SpotifyTrackItem>().await?;
        Ok(track)
    } else {
        let err = res.json::<SpotifyErrorResponse>().await?;
        Err(anyhow::anyhow!("Spotify error {}: {}", err.error.status, err.error.message))
    }
}
