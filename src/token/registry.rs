use crate::token::spotify::SpotifyTokenManager;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct TokenRegistry {
    pub spotify: Arc<Mutex<SpotifyTokenManager>>,

}

impl TokenRegistry {
    pub fn new() -> Self {
        Self {
            spotify: Arc::new(Mutex::new(SpotifyTokenManager::new())),
        }
    }
}
