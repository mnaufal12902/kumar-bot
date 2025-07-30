use dotenv::dotenv;
use tracing_subscriber::EnvFilter;

mod api;
mod bot;
mod commands;
mod handler;
mod models;
mod token;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    if let Err(e) = bot::start().await {
        tracing::error!("Bot error: {:?}", e);
    }
}
