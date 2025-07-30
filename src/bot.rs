use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

use reqwest::Client as HttpClient;
use serenity::model::gateway::GatewayIntents;
use serenity::prelude::TypeMapKey;
use serenity::Client;
use songbird::SerenityInit;

use crate::commands::music::queue::BotMusicState;
use crate::handler::Handler;

pub struct HttpKey;
pub struct MusicStateKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

impl TypeMapKey for MusicStateKey {
    type Value = Arc<Mutex<BotMusicState>>;
}

pub async fn start() -> serenity::Result<()> {
    // Login with a bot token from environtment
    let token = env::var("BOT_TOKEN").expect("Missing bot token, please configure you env");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .type_map_insert::<MusicStateKey>(Arc::new(Mutex::new(BotMusicState::new())))
        .await?;

    client.start().await
}
