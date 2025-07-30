mod message; 

use serenity::all::{ActivityData, ActivityType, OnlineStatus};
use serenity::async_trait;
use serenity::model::{channel::Message, gateway::Ready};
use serenity::prelude::*;
use message::handle_message;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        handle_message(ctx, msg).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_presence(
            Some(ActivityData {
                name: "on Sleeping".to_string(),
                kind: ActivityType::Playing,
                state: None,
                url: None,
            }),
            OnlineStatus::Idle,
        );
        
        tracing::info!("{} is online!", ready.user.name);
    }
}
