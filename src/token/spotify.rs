use std::time::{Duration, Instant};
use base64::{engine::general_purpose, Engine};
use reqwest::Client;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct SpotifyToken {
    pub access_token: String,
    pub expired_at: Instant,
}

impl SpotifyToken {
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expired_at
    }
}

#[derive(Debug)]
pub struct SpotifyTokenManager {
    token: Option<SpotifyToken>,
}

impl SpotifyTokenManager {
    pub fn new() -> Self {
        Self { token: None }
    }

    pub async fn get_token(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        match &self.token {
            Some(token) if !token.is_expired() => {
                Ok(token.access_token.clone())
            }
            _ => {
                let new_token = Self::fetch_new_token().await?;
                let access_token = new_token.access_token.clone();
                self.token = Some(new_token);
                Ok(access_token)
            }
        }
    }

    async fn fetch_new_token() -> Result<SpotifyToken, Box<dyn std::error::Error>> {
        let client_id = env::var("SPOTIFY_CLIENT_ID")?;
        let secret_id = env::var("SPOTIFY_SECRET_ID")?;
        let credentials = format!("{}:{}", client_id, secret_id);
        let encoded = general_purpose::STANDARD.encode(credentials);

        let client = Client::new();
        let res = client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", encoded))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body("grant_type=client_credentials")
            .send()
            .await?
            .error_for_status()?
            .json::<SpotifyTokenResponse>()
            .await?;

        Ok(SpotifyToken {
            access_token: res.access_token,
            expired_at: Instant::now() + Duration::from_secs(res.expires_in),
        })
    }
}

#[derive(Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
    expires_in: u64,
}
